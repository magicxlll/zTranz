//! Small helpers: wide strings, percent-encoding, config file, logging.

use std::path::PathBuf;

/// UTF-16 NUL-terminated buffer for Win32 `PCWSTR` arguments.
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn wide_of(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// Percent-encode for URL query strings (UTF-8).
pub fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// `%APPDATA%\zTranz\config.json` (fallback: `%USERPROFILE%\.ztranz`).
pub fn config_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join(".ztranz")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("zTranz").join("config.json")
}

/// Where we drop the temporary screenshot for OCR.
pub fn temp_dir() -> PathBuf {
    std::env::var_os("TEMP")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("TMP").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn log(msg: &str) {
    let line = format!("[zTranz] {}\n", msg);
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(config_path().with_file_name("ztranz.log"))
    {
        let _ = f.write_all(line.as_bytes());
    }
    eprint!("{}", line);
}
