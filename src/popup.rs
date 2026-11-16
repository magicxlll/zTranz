//! The result popup: a frameless, always-on-top window drawn with GDI.
//! Pure Win32 → no WebView2 runtime needed, tiny binary, instant show.

use std::cell::RefCell;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, GetClientRect,
    GetTextExtentPoint32W, SetBkColor, SetTextColor, TextOutW, COLOR_WINDOW as GDI_COLOR_WINDOW,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, PostMessageW, RegisterClassW, SetForegroundWindow,
    SetWindowPos, ShowWindow, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    HWND_TOPMOST, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_SHOW,
};

use crate::util::wide;

pub struct PopupContent {
    pub source: String,
    pub translated: String,
    pub status: String,
}

thread_local! {
    static CONTENT: RefCell<PopupContent> = RefCell::new(PopupContent {
        source: String::new(),
        translated: String::new(),
        status: String::new(),
    });
}

pub fn set_content(c: PopupContent) {
    CONTENT.with(|x| *x.borrow_mut() = c);
}

pub struct Popup {
    hwnd: HWND,
}

impl Popup {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let class_name = w!("zTranzPopupClass");
            let hinst = GetModuleHandleW(None).map_err(|e| format!("GetModuleHandle: {e}"))?;

            let wc = windows::Win32::UI::WindowsAndMessaging::WNDCLASSW {
                lpfnWndProc: Some(wnd_proc),
                hInstance: hinst.into(),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                hbrBackground: CreateSolidBrush(GDI_COLOR_WINDOW),
                ..Default::default()
            };
            // Ignore "already registered" errors.
            let _ = RegisterClassW(&wc);

            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class_name,
                w!("zTranz"),
                WS_POPUP,
                100,
                100,
                560,
                360,
                None,
                None,
                Some(hinst.into()),
                None,
            )
            .map_err(|e| format!("CreateWindowEx: {e}"))?;

            Ok(Popup { hwnd })
        }
    }

    pub fn show(&self, content: PopupContent) {
        set_content(content);
        unsafe {
            let mut pt = POINT { x: 0, y: 0 };
            let _ = windows::Win32::UI::Input::KeyboardAndMouse::GetCursorPos(&mut pt);

            let (w, h) = (560i32, 360i32);
            let x = (pt.x - w / 2).max(8);
            let y = (pt.y + 24).max(8);

            let _ = SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                x,
                y,
                w,
                h,
                SWP_SHOWWINDOW | SWP_NOACTIVATE,
            );
            let _ = ShowWindow(self.hwnd, SW_SHOW);
            let _ = SetForegroundWindow(self.hwnd);
            let _ = windows::Win32::UI::WindowsAndMessaging::RedrawWindow(
                self.hwnd,
                None,
                None,
                windows::Win32::UI::WindowsAndMessaging::REDRAW_WINDOW_FLAGS(0x0001 | 0x0004 | 0x0080),
            );
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

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        0x0002 => {
            // WM_DESTROY
            windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
            LRESULT(0)
        }
        0x0011 => {
            // WM_COMMAND
            match (wparam.0 & 0xFFFF) as u16 {
                1 => {
                    // Copy translated text
                    let t = CONTENT.with(|c| c.borrow().translated.clone());
                    crate::clipboard::set_text(&t);
                    let _ = PostMessageW(hwnd, 0x0010, WPARAM(0), LPARAM(0));
                }
                2 => {
                    let _ = PostMessageW(hwnd, 0x0010, WPARAM(0), LPARAM(0));
                }
                _ => {}
            }
            LRESULT(0)
        }
        0x000F => {
            // WM_PAINT
            let mut ps = windows::Win32::Graphics::Gdi::PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            let mut rc = RECT::default();
            let _ = GetClientRect(hwnd, &mut rc);

            let brush = CreateSolidBrush(GDI_COLOR_WINDOW);
            let _ = FillRect(hdc, &rc, brush);
            let _ = DeleteObject(Some(&brush));

            draw_content(hdc, &rc);
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        0x0100 => {
            // WM_KEYDOWN — Esc closes
            if wparam.0 == 0x1B {
                let _ = PostMessageW(hwnd, 0x0010, WPARAM(0), LPARAM(0));
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn draw_content(hdc: windows::Win32::Graphics::Gdi::HDC, rc: &RECT) {
    let content = CONTENT.with(|c| c.borrow().clone());

    let _ = SetBkColor(hdc, GDI_COLOR_WINDOW);
    let _ = SetTextColor(hdc, COLORREF(0x001A1A1A));

    let left = 16;
    let width = (rc.right - rc.left - 32).max(10);
    let mut y = 14;

    // Title
    let title = wide("zTranz");
    let _ = TextOutW(hdc, left, y, &title);
    y += 24;

    // Divider
    let pen = CreateSolidBrush(COLORREF(0x00E0E0E0));
    let line = RECT {
        left,
        top: y,
        right: rc.right - 16,
        bottom: y + 1,
    };
    let _ = FillRect(hdc, &line, pen);
    let _ = DeleteObject(Some(&pen));
    y += 12;

    // Source text (grey)
    y = draw_wrapped(hdc, left, y, width, &content.source, 0x00808080, 4);
    y += 6;

    // Translated text (dark)
    y = draw_wrapped(hdc, left, y, width, &content.translated, 0x001A1A1A, 8);

    // Status bar
    if !content.status.is_empty() {
        let status = RECT {
            left,
            top: rc.bottom - 30,
            right: rc.right - 16,
            bottom: rc.bottom - 14,
        };
        let brush = CreateSolidBrush(COLORREF(0x00F2F2F2));
        let _ = FillRect(hdc, &status, brush);
        let _ = DeleteObject(Some(&brush));

        let _ = SetTextColor(hdc, COLORREF(0x00404040));
        let mut buf: Vec<u16> = content.status.encode_utf16().chain(std::iter::once(0)).collect();
        let _ = TextOutW(hdc, left + 8, rc.bottom - 27, &mut buf);
    }
}

unsafe fn draw_wrapped(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    x: i32,
    y: i32,
    max_width: i32,
    text: &str,
    color: u32,
    max_lines: i32,
) -> i32 {
    let _ = SetTextColor(hdc, COLORREF(color));
    let mut y = y;
    let mut lines = 0i32;

    for para in text.split('\n') {
        let words: Vec<&str> = para.split_whitespace().collect();
        if words.is_empty() {
            if lines < max_lines {
                y += 16;
                lines += 1;
            }
            continue;
        }
        let mut line = String::new();
        for word in words {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", line, word)
            };
            if text_width(hdc, &candidate) > max_width && !line.is_empty() {
                if lines < max_lines {
                    let mut buf: Vec<u16> = line.encode_utf16().chain(std::iter::once(0)).collect();
                    let _ = TextOutW(hdc, x, y, &mut buf);
                    y += 16;
                    lines += 1;
                }
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
        if lines < max_lines {
            let mut buf: Vec<u16> = line.encode_utf16().chain(std::iter::once(0)).collect();
            let _ = TextOutW(hdc, x, y, &mut buf);
            y += 16;
            lines += 1;
        }
        if lines >= max_lines {
            break;
        }
    }
    y
}

unsafe fn text_width(hdc: windows::Win32::Graphics::Gdi::HDC, s: &str) -> i32 {
    let mut buf: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
    let mut size = SIZE::default();
    let _ = GetTextExtentPoint32W(hdc, &mut buf, &mut size);
    size.cx
}
