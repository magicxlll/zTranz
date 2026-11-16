//! Drag-to-select region overlay: a transparent, always-on-top,
//! click-capturing window covering the whole virtual screen.
//!
//! Drawn with GDI into a 32-bit DIB section and blitted with
//! UpdateLayeredWindow (per-pixel alpha): the area outside the selection is
//! dimmed, the selection stays clear, and an animated border marks the crop.

use std::cell::RefCell;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetClientRect, GetDC,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, PostMessageW, RegisterClassW, SetWindowPos, ShowWindow,
    UpdateLayeredWindow, HWND_TOPMOST, SWP_NOZORDER, SWP_SHOWWINDOW, SW_SHOW, ULW_ALPHA,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

const WM_MOUSEMOVE: u32 = 0x0200;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_LBUTTONUP: u32 = 0x0202;

#[derive(Default, Clone, Copy)]
struct OverlayState {
    start: (i32, i32),
    cur: (i32, i32),
    dragging: bool,
}

thread_local! {
    static STATE: RefCell<OverlayState> = const { RefCell::new(OverlayState { dragging: false, ..Default::default() }) };
    static INSTANCE_HWND: RefCell<HWND> = const { RefCell::new(HWND(std::ptr::null_mut())) };
}

pub struct Overlay {
    hwnd: HWND,
}

impl Overlay {
    /// Create the overlay window (hidden). Call on the UI thread.
    pub fn new() -> Result<Self, String> {
        unsafe {
            let class_name = w!("zTranzOverlayClass");
            let hinst = GetModuleHandleW(None).map_err(|e| format!("GetModuleHandle: {e}"))?;

            let wc = windows::Win32::UI::WindowsAndMessaging::WNDCLASSW {
                lpfnWndProc: Some(overlay_proc),
                hInstance: hinst.into(),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };
            let _ = RegisterClassW(&wc);

            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                class_name,
                w!("zTranz Overlay"),
                WS_POPUP,
                0,
                0,
                1,
                1,
                None,
                None,
                Some(hinst.into()),
                None,
            )
            .map_err(|e| format!("CreateWindowEx overlay: {e}"))?;

            INSTANCE_HWND.with(|x| *x.borrow_mut() = hwnd);
            Ok(Overlay { hwnd })
        }
    }

    /// Show the overlay over the whole virtual screen.
    pub fn show(&self, geom: &crate::capture::ScreenGeometry) {
        STATE.with(|s| {
            *s.borrow_mut() = OverlayState {
                dragging: false,
                ..Default::default()
            };
        });
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                geom.x,
                geom.y,
                geom.width,
                geom.height,
                SWP_SHOWWINDOW | SWP_NOZORDER,
            );
            let _ = ShowWindow(self.hwnd, SW_SHOW);
            paint(self.hwnd);
        }
    }

    pub fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, windows::Win32::UI::WindowsAndMessaging::SW_HIDE);
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }
}

unsafe extern "system" fn overlay_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_LBUTTONDOWN => {
            let (x, y) = lparam_point(lparam);
            STATE.with(|s| {
                let mut st = s.borrow_mut();
                st.start = (x, y);
                st.cur = (x, y);
                st.dragging = true;
            });
            paint(hwnd);
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let (x, y) = lparam_point(lparam);
            let tracking = STATE.with(|s| s.borrow().dragging);
            if tracking {
                STATE.with(|s| s.borrow_mut().cur = (x, y));
                paint(hwnd);
            }
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let (x, y) = lparam_point(lparam);
            STATE.with(|s| {
                let mut st = s.borrow_mut();
                st.cur = (x, y);
                st.dragging = false;
            });

            let sel = selection();
            let geom = crate::capture::virtual_screen();
            let region = (geom.x + sel.0, geom.y + sel.1, sel.2, sel.3);

            if sel.2 >= 8 && sel.3 >= 8 {
                crate::main_dispatch::on_region_selected(region);
            } else {
                crate::main_dispatch::on_region_cancelled();
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn lparam_point(l: LPARAM) -> (i32, i32) {
    (
        (l.0 & 0xFFFF) as u16 as i32,
        ((l.0 >> 16) & 0xFFFF) as u16 as i32,
    )
}

/// Current selection in client coordinates.
fn selection() -> (i32, i32, i32, i32) {
    STATE.with(|s| {
        let st = s.borrow();
        let (ax, ay) = st.start;
        let (bx, by) = st.cur;
        (
            ax.min(bx),
            ay.min(by),
            ax.abs_diff(bx),
            ay.abs_diff(by),
        )
    })
}

/// Repaint the overlay bitmap.
fn paint(hwnd: HWND) {
    unsafe {
        let mut rc = windows::Win32::Foundation::RECT::default();
        let _ = GetClientRect(hwnd, &mut rc);
        let (w, h) = (rc.right, rc.bottom);
        if w <= 1 || h <= 1 {
            return;
        }

        let bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..Default::default()
            },
            bmiColors: [0],
        };

        let pixels = build_overlay_pixels(w, h, selection());

        let hdc = match CreateCompatibleDC(None) {
            Ok(dc) => dc,
            Err(_) => return,
        };
        let mut bits: *mut u8 = std::ptr::null_mut();
        let hbm = match CreateDIBSection(
            hdc,
            &bi,
            DIB_RGB_COLORS,
            &mut bits as *mut *mut u8 as *mut _,
            None,
            0,
        ) {
            Ok(b) => b,
            Err(_) => {
                DeleteDC(hdc);
                return;
            }
        };
        let old = SelectObject(hdc, hbm);
        if !bits.is_null() {
            std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits, pixels.len());
        }

        let pos = POINT { x: 0, y: 0 };
        let size = SIZE { cx: w, cy: h };
        let _ = UpdateLayeredWindow(
            hwnd,
            None,
            None,
            &size,
            hdc,
            &pos,
            COLORREF(0xFF000000),
            None,
            ULW_ALPHA,
        );
        let scr = GetDC(None);
        if let Ok(dc) = scr {
            let _ = ReleaseDC(None, dc);
        }
        SelectObject(hdc, old);
        DeleteObject(Some(&hbm));
        DeleteDC(hdc);
    }
}

/// Build the ARGB buffer: dim outside the selection, clear inside,
/// animated border around the crop.
fn build_overlay_pixels(width: i32, height: i32, sel: (i32, i32, i32, i32)) -> Vec<u8> {
    let total = (width as usize) * (height as usize) * 4;
    let mut buf = vec![0u8; total];
    let (sx, sy, sw, sh) = sel;

    const DIM: u8 = 110; // ~43% black outside the crop

    for row in 0..height as usize {
        for col in 0..width as usize {
            let i = (row * (width as usize) + col) * 4;
            let cx = col as i32;
            let cy = row as i32;

            let in_sel = sw > 0
                && sh > 0
                && cx >= sx
                && cx < sx + sw
                && cy >= sy
                && cy < sy + sh;

            let near_edge = sw > 0
                && sh > 0
                && cx >= sx - 1
                && cx <= sx + sw
                && cy >= sy - 1
                && cy <= sy + sh;
            let on_edge = near_edge
                && (cx == sx - 1 || cx == sx + sw || cy == sy - 1 || cy == sy + sh);

            if on_edge {
                // Marching ants.
                if (row % 12 < 6) ^ (col % 12 < 6) {
                    buf[i] = 255;
                    buf[i + 1] = 255;
                    buf[i + 2] = 255;
                    buf[i + 3] = 255;
                } else {
                    buf[i] = 20;
                    buf[i + 1] = 20;
                    buf[i + 2] = 20;
                    buf[i + 3] = 255;
                }
            } else if in_sel {
                buf[i + 3] = 0; // fully transparent inside the crop
            } else {
                buf[i + 3] = DIM;
            }
        }
    }
    buf
}
