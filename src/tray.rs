//! System tray icon with a right-click menu.

use std::cell::RefCell;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_MODIFY, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, PostMessageW, RegisterClassW,
    SetForegroundWindow, WNDCLASSW, WS_EX_TOOLWINDOW, WS_POPUP,
};

pub const WM_TRAY: u32 = 0x8000 + 1;

const ID_TRANSLATE: u16 = 1001;
const ID_CAPTURE: u16 = 1002;
const ID_QUIT: u16 = 1003;
const ID_STATUS: u16 = 1004;

pub struct Tray {
    hwnd: HWND,
    _icon: windows::Win32::UI::WindowsAndMessaging::HICON,
}

impl Tray {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let class_name = w!("zTranzTrayClass");
            let hinst = GetModuleHandleW(None).map_err(|e| format!("GetModuleHandle: {e}"))?;

            let wc = WNDCLASSW {
                lpfnWndProc: Some(tray_proc),
                hInstance: hinst.into(),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };
            let _ = RegisterClassW(&wc);

            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW,
                class_name,
                w!("zTranz Tray"),
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
            .map_err(|e| format!("tray window: {e}"))?;

            // A small generated icon so we don't depend on an .ico resource.
            let icon = make_icon();

            let mut nid = NOTIFYICONDATAW {
                hWnd: hwnd,
                uID: 1,
                uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
                uCallbackMessage: WM_TRAY,
                hIcon: icon,
                ..Default::default()
            };
            copy_wide(&mut nid.szTip, "zTranz — ready");
            copy_wide(&mut nid.szInfo, "zTranz — ready");

            if Shell_NotifyIconW(NIM_ADD, &nid).as_bool() == false {
                return Err("Shell_NotifyIcon failed".into());
            }

            Ok(Tray { hwnd, _icon: icon })
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn set_tooltip(&self, text: &str) {
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                hWnd: self.hwnd,
                uID: 1,
                uFlags: NIF_TIP,
                ..Default::default()
            };
            copy_wide(&mut nid.szTip, text);
            let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }

    pub fn quit(&self) {
        unsafe {
            let nid = NOTIFYICONDATAW {
                hWnd: self.hwnd,
                uID: 1,
                ..Default::default()
            };
            let _ = Shell_NotifyIconW(windows::Win32::UI::Shell::NIM_DELETE, &nid);
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

unsafe fn copy_wide(dst: &mut [u16], s: &str) {
    let enc: Vec<u16> = s.encode_utf16().collect();
    let n = enc.len().min(dst.len().saturating_sub(1));
    dst[..n].copy_from_slice(&enc[..n]);
    for c in dst[n..].iter_mut() {
        *c = 0;
    }
}

/// Generate a simple 16x16 icon (a globe-ish glyph) at runtime so the
/// binary needs no external resource file.
unsafe fn make_icon() -> windows::Win32::UI::WindowsAndMessaging::HICON {
    use windows::Win32::Graphics::Gdi::{
        CreateBitmap, CreateDIBSection, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        DIB_RGB_COLORS,
    };
    use windows::Win32::UI::WindowsAndMessaging::CreateIconIndirect;

    let size = 32i32;
    let bi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size,
            biHeight: -size * 2,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..Default::default()
        },
        bmiColors: [0],
    };

    let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
    let dc = match screen_dc {
        Ok(d) => d,
        Err(_) => return windows::Win32::UI::WindowsAndMessaging::HICON(std::ptr::null_mut()),
    };
    let mut bits: *mut u8 = std::ptr::null_mut();
    let hbm = match CreateDIBSection(
        dc,
        &bi,
        DIB_RGB_COLORS,
        &mut bits as *mut *mut u8 as *mut _,
        None,
        0,
    ) {
        Ok(b) => b,
        Err(_) => return windows::Win32::UI::WindowsAndMessaging::HICON(std::ptr::null_mut()),
    };
    let old = SelectObject(dc, hbm);

    // Draw: blue circle on transparent background, with a lighter "T".
    let n = (size * size) as usize;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - size as f32 / 2.0;
            let dy = y as f32 - size as f32 / 2.0;
            let d = (dx * dx + dy * dy).sqrt();
            let i = (y as usize * size as usize + x as usize) * 4;
            if d < size as f32 / 2.0 - 1.0 {
                let (mut r, mut g, mut b) = (46u8, 134u8, 222u8);
                // A vertical + horizontal bar forming a "T".
                let in_t =
                    (x > 7 && x < 25 && y > 9 && y < 13) || (x > 14 && x < 18 && y > 9 && y < 23);
                if in_t {
                    r = 255;
                    g = 255;
                    b = 255;
                }
                bits.add(i).write(r);
                bits.add(i + 1).write(g);
                bits.add(i + 2).write(b);
                bits.add(i + 3).write(255);
            } else {
                for k in 0..4 {
                    bits.add(i + k).write(0);
                }
            }
        }
    }
    // Fill the AND mask row (the bitmap is 2× height: colour + mask).
    for k in 0..(n * 4) {
        bits.add(n * 4 + k).write(0);
    }

    let icon_info = windows::Win32::UI::WindowsAndMessaging::ICONINFO {
        fIcon: true,
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: CreateBitmap(size, size, 1, 1, None),
        hbmColor: hbm,
    };
    let hicon = CreateIconIndirect(&icon_info);

    SelectObject(dc, old);
    let _ = windows::Win32::Graphics::Gdi::DeleteObject(Some(&hbm));
    let _ = windows::Win32::Graphics::Gdi::ReleaseDC(None, dc);
    hicon
}

unsafe extern "system" fn tray_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAY => {
            match lparam.0 as u32 {
                0x0202 => {
                    // WM_LBUTTONUP → translate selection/clipboard
                    crate::main_dispatch::on_translate_requested();
                }
                0x0205 => {
                    // WM_RBUTTONUP → context menu
                    show_menu(hwnd);
                }
                _ => {}
            }
            LRESULT(0)
        }
        0x0111 => {
            // WM_COMMAND
            let id = (wparam.0 & 0xFFFF) as u16;
            match id {
                ID_TRANSLATE => crate::main_dispatch::on_translate_requested(),
                ID_CAPTURE => crate::main_dispatch::on_capture_requested(),
                ID_QUIT => {
                    crate::main_dispatch::on_quit_requested();
                }
                _ => {}
            }
            LRESULT(0)
        }
        0x0002 => {
            // WM_DESTROY
            windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn show_menu(hwnd: HWND) {
    use windows::Win32::UI::WindowsAndMessaging::{
        CreatePopupMenu, GetCursorPos, SetForegroundWindow, TrackPopupMenu, MF_STRING,
        TPM_RETURNCMD, TPM_RIGHTBUTTON,
    };
    let menu = match CreatePopupMenu() {
        Ok(m) => m,
        Err(_) => return,
    };
    let _ = windows::Win32::UI::WindowsAndMessaging::AppendMenuW(
        menu,
        MF_STRING,
        ID_TRANSLATE as usize,
        windows::core::PCWSTR(w!("Translate selection / clipboard").as_ptr()),
    );
    let _ = windows::Win32::UI::WindowsAndMessaging::AppendMenuW(
        menu,
        MF_STRING,
        ID_CAPTURE as usize,
        windows::core::PCWSTR(w!("Capture screen region...").as_ptr()),
    );
    let _ =
        windows::Win32::UI::WindowsAndMessaging::AppendMenuW(menu, MF_STRING, 0, PCWSTR::null());
    let _ = windows::Win32::UI::WindowsAndMessaging::AppendMenuW(
        menu,
        MF_STRING,
        ID_QUIT as usize,
        windows::core::PCWSTR(w!("Quit zTranz").as_ptr()),
    );

    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);
    let _ = SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(
        menu,
        TPM_RIGHTBUTTON | TPM_RETURNCMD,
        pt.x,
        pt.y,
        0,
        hwnd,
        None,
    );
    let _ = DestroyWindow(menu);

    match cmd.0 as u16 {
        ID_TRANSLATE => crate::main_dispatch::on_translate_requested(),
        ID_CAPTURE => crate::main_dispatch::on_capture_requested(),
        ID_QUIT => crate::main_dispatch::on_quit_requested(),
        _ => {}
    }
}
