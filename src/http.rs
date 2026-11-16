//! Minimal HTTPS client built on the Windows WinHTTP API.
//! Using the OS TLS stack keeps the binary tiny and makes cross-compiling
//! to Windows from any host painless (no OpenSSL to build).

use windows::core::PCWSTR;
use windows::Win32::Networking::WinHttp::{
    WinHttpCloseHandle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest, WinHttpQueryHeaders,
    WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest, HINTERNET,
    WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
};
use windows::Win32::WinHttp::{WINHTTP_HEADER_NAME_BY_INDEX, WINHTTP_QUERY_FLAG_NUMBER};

pub struct Url {
    pub host: String,
    pub port: u16,
    pub path_and_query: String,
    pub secure: bool,
}

pub fn parse_url(url: &str) -> Result<Url, String> {
    let (scheme, rest) = url
        .split_once("://")
        .ok_or_else(|| format!("bad url (no scheme): {url}"))?;
    let secure = match scheme {
        "https" => true,
        "http" => false,
        _ => return Err(format!("unsupported scheme: {scheme}")),
    };
    let (hostport, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match hostport.rsplit_once(':') {
        Some((h, p)) => (
            h.to_string(),
            p.parse::<u16>().map_err(|_| format!("bad port in {url}"))?,
        ),
        None => (hostport.to_string(), if secure { 443u16 } else { 80u16 }),
    };
    Ok(Url {
        host,
        port,
        path_and_query: path.to_string(),
        secure,
    })
}

struct Session(HINTERNET);

impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            let _ = WinHttpCloseHandle(self.0);
        }
    }
}

/// Perform a request and return the response body as text.
pub fn request(
    method: &str,
    url: &str,
    body: Option<&[u8]>,
    headers: &[(String, String)],
) -> Result<String, String> {
    let u = parse_url(url)?;

    unsafe {
        let agent: Vec<u16> = "zTranz/0.1\0".encode_utf16().collect();
        let h = WinHttpOpen(
            PCWSTR(agent.as_ptr()),
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            PCWSTR::null(),
            PCWSTR::null(),
            0,
        );
        let h = match h {
            Ok(h) => Session(h),
            Err(e) => return Err(format!("WinHttpOpen failed: {e}")),
        };

        let host: Vec<u16> = format!("{}\0", u.host).encode_utf16().collect();
        let conn = WinHttpConnect(h.0, &host, u.port, 0)
            .map_err(|e| format!("WinHttpConnect failed: {e}"))?;
        let conn = Session(conn);

        let path: Vec<u16> = format!("{}\0", u.path_and_query).encode_utf16().collect();
        let verb: Vec<u16> = format!("{}\0", method).encode_utf16().collect();
        let req = WinHttpOpenRequest(
            conn.0,
            PCWSTR(verb.as_ptr()),
            PCWSTR(path.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            PCWSTR::null(),
            0,
            0,
        )
        .map_err(|e| format!("WinHttpOpenRequest failed: {e}"))?;
        let req = Session(req);

        // Build the extra header block.
        let mut hdr = String::new();
        for (k, v) in headers {
            hdr.push_str(k);
            hdr.push_str(": ");
            hdr.push_str(v);
            hdr.push_str("\r\n");
        }
        let hdr_w: Vec<u16> = if hdr.is_empty() {
            Vec::new()
        } else {
            format!("{}\0", hdr).encode_utf16().collect()
        };
        let hdr_ptr = if hdr_w.is_empty() {
            PCWSTR::null()
        } else {
            PCWSTR(hdr_w.as_ptr())
        };

        let opt_len = body.map(|b| b.len() as u32).unwrap_or(0);
        let opt_ptr = body.map(|b| b.as_ptr()).unwrap_or(std::ptr::null());

        WinHttpSendRequest(
            req.0,
            hdr_ptr,
            hdr_w.len() as u32,
            Some(opt_ptr),
            opt_len,
            0,
            0,
        )
        .map_err(|e| format!("send failed: {e}"))?;

        WinHttpReceiveResponse(req.0, None).map_err(|e| format!("no response: {e}"))?;

        // Status code (best effort).
        let mut status: u32 = 0;
        let _ = WinHttpQueryHeaders(
            req.0,
            (WINHTTP_QUERY_FLAG_NUMBER | 0x0000000F) as u32,
            WINHTTP_HEADER_NAME_BY_INDEX,
            Some(&mut status as *mut u32 as *mut _),
            std::mem::size_of::<u32>() as u32,
            Some(std::ptr::null_mut()),
            Some(std::ptr::null_mut()),
        );
        // 0x0F = WINHTTP_QUERY_STATUS_CODE
        let status = if status == 0 { 200 } else { status };

        let mut buf = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            let mut read: u32 = 0;
            let ok =
                WinHttpReadData(req.0, chunk.as_mut_ptr(), chunk.len() as u32, &mut read).is_ok();
            if !ok || read == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..read as usize]);
            if buf.len() > 8 * 1024 * 1024 {
                break;
            }
        }

        let text = String::from_utf8_lossy(&buf).to_string();
        if status >= 400 {
            return Err(format!(
                "HTTP {} from {}: {}",
                status,
                u.host,
                truncate(&text, 400)
            ));
        }
        Ok(text)
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n).collect::<String>() + "..."
    }
}

pub fn get(url: &str) -> Result<String, String> {
    request("GET", url, None, &[])
}

pub fn post_json(url: &str, body: &str, headers: &[(String, String)]) -> Result<String, String> {
    let mut h = vec![(
        "Content-Type".to_string(),
        "application/json; charset=utf-8".to_string(),
    )];
    h.extend_from_slice(headers);
    request("POST", url, Some(body.as_bytes()), &h)
}
