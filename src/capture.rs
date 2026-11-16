//! Screen capture helpers: raw BGRA grab of the virtual screen and
//! PNG export of a cropped region (used for OCR).

use std::path::Path;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
    SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;

#[derive(Clone, Copy, Debug)]
pub struct ScreenGeometry {
    /// Virtual screen origin (may be negative on multi-monitor).
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn virtual_screen() -> ScreenGeometry {
    unsafe {
        ScreenGeometry {
            x: GetSystemMetrics(76).try_into().unwrap_or(0),  // SM_XVIRTUALSCREEN
            y: GetSystemMetrics(77).try_into().unwrap_or(0),  // SM_YVIRTUALSCREEN
            width: GetSystemMetrics(78).try_into().unwrap_or(1920), // SM_CXVIRTUALSCREEN
            height: GetSystemMetrics(79).try_into().unwrap_or(1080), // SM_CYVIRTUALSCREEN
        }
    }
}

pub struct RawScreen {
    pub geom: ScreenGeometry,
    /// BGRA, top-down, `geom.width * geom.height * 4` bytes.
    pub bgra: Vec<u8>,
}

/// Grab the whole virtual screen into memory.
pub fn grab_virtual_screen() -> Result<RawScreen, String> {
    let geom = virtual_screen();
    if geom.width <= 0 || geom.height <= 0 {
        return Err("could not determine screen size".into());
    }
    let (w, h) = (geom.width, geom.height);

    unsafe {
        let src = GetDC(None).ok_or("GetDC failed")?;
        let mem = CreateCompatibleDC(&src).ok_or("CreateCompatibleDC failed")?;

        let bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -(h as i32), // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [0],
        };

        let mut bits: *mut u8 = std::ptr::null_mut();
        let hbm = CreateDIBSection(
            mem,
            &bi,
            DIB_RGB_COLORS,
            &mut bits as *mut *mut u8 as *mut _,
            None,
            0,
        )
        .ok_or("CreateDIBSection failed")?;

        let old = SelectObject(mem, hbm);
        BitBlt(mem, 0, 0, w, h, Some(&src), geom.x, geom.y, SRCCOPY);

        let total = (w as usize) * (h as usize) * 4;
        let bgra = if bits.is_null() {
            Vec::new()
        } else {
            std::slice::from_raw_parts(bits, total).to_vec()
        };

        SelectObject(mem, old);
        DeleteObject(Some(&hbm));
        DeleteDC(mem);
        ReleaseDC(None, src);

        if bgra.is_empty() {
            return Err("screen capture returned no pixels".into());
        }
        Ok(RawScreen { geom, bgra })
    }
}

/// Crop a region (virtual-screen coordinates) out of a raw screen grab and
/// write it to `out` as a PNG.
pub fn crop_to_png(src: &RawScreen, region: (i32, i32, i32, i32), out: &Path) -> Result<(), String> {
    let (rx, ry, rw, rh) = region;
    let sw = src.geom.width;
    let sh = src.geom.height;

    // Translate virtual-screen coords into buffer coords.
    let x0 = (rx - src.geom.x).clamp(0, sw - 1);
    let y0 = (ry - src.geom.y).clamp(0, sh - 1);
    let x1 = ((rx + rw) - src.geom.x).clamp(x0 + 1, sw);
    let y1 = ((ry + rh) - src.geom.y).clamp(y0 + 1, sh);

    let cw = (x1 - x0) as u32;
    let ch = (y1 - y0) as u32;
    if cw == 0 || ch == 0 {
        return Err("empty capture region".into());
    }

    let stride = sw as usize * 4;
    let mut rgba = Vec::with_capacity((cw as usize) * (ch as usize) * 4);
    for y in y0..y1 {
        let row_start = y as usize * stride + x0 as usize * 4;
        let row = &src.bgra[row_start..row_start + cw as usize * 4];
        for px in row.chunks_exact(4) {
            rgba.push(px[2]); // R
            rgba.push(px[1]); // G
            rgba.push(px[0]); // B
            rgba.push(px[3]); // A
        }
    }

    let img = image::RgbaImage::from_raw(cw, ch, &rgba).ok_or("bad pixel buffer")?;
    let file = std::io::BufWriter::new(
        std::fs::File::create(out).map_err(|e| format!("create {}: {}", out.display(), e))?,
    );
    img.write_to(file).map_err(|e| format!("encode png: {e}"))?;
    Ok(())
}
