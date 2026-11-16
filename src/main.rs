mod capture;
mod clipboard;
mod config;
mod http;
mod main_dispatch;
mod ocr;
mod overlay;
mod popup;
mod state;
mod trace;
mod translate;
mod tray;
mod util;

use std::sync::atomic::Ordering;
use windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PeekMessageW, TranslateMessage, PM_REMOVE, WM_HOTKEY,
};

// Global hotkeys. Win+Shift so they rarely collide with apps.
const MOD_WIN: u32 = 0x0004;
const MOD_SHIFT: u32 = 0x0002;
const MOD_ALT: u32 = 0x0001;

const HK_CAPTURE: i32 = 1; // Win+Shift+C → region select
const HK_TRANSLATE: i32 = 2; // Win+Shift+V → translate selection/clipboard
const HK_QUIT: i32 = 3; // Win+Shift+Q → quit

fn main() {
    util::log("=== zTranz starting ===");
    if let Err(e) = run() {
        util::log(&format!("FATAL: {e}"));
        eprintln!("zTranz: {e}");
        // Show the error in a message box so a GUI-only launch is visible.
        show_error(&e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    // The UI thread owns: tray icon, popup window, overlay window, hotkeys.
    let tray = tray::Tray::new()?;
    let popup = popup::Popup::new()?;
    let ov = overlay::Overlay::new()?;

    unsafe {
        main_dispatch::OVERLAY = Some(ov);
    }

    register_hotkeys(tray.hwnd())?;

    // Run the message loop.
    unsafe {
        loop {
            // Drain queued window messages (tray callbacks, hotkeys).
            loop {
                let mut msg = MSG::default();
                if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).0 == 0 {
                    break;
                }
                if msg.message == WM_HOTKEY {
                    handle_hotkey(msg.wParam.0 as i32);
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // Apply pending UI state changes produced by the workers.
            if state::SHOW_POPUP.swap(false, Ordering::SeqCst) {
                if let Some(c) = state::take_content() {
                    popup.show(c);
                }
            }
            if state::HIDE_POPUP.swap(false, Ordering::SeqCst) {
                popup.hide();
            }
            if state::STATUS_DIRTY.swap(false, Ordering::SeqCst) {
                tray.set_tooltip(&state::current_status());
            }

            if main_dispatch::QUIT.load(Ordering::SeqCst) {
                break;
            }

            // Sleep briefly rather than spin.
            let mut msg = MSG::default();
            unsafe {
                // Wait up to 30 ms for a message, then loop.
                let _ = windows::Win32::UI::WindowsAndMessaging::MsgWaitForMultipleObjectsEx(
                    None,
                    0,
                    false,
                    30,
                    windows::Win32::Foundation::INFINITE,
                    windows::Win32::UI::WindowsAndMessaging::QS_ALLINPUT,
                );
            }
            let _ = &mut msg;
        }
    }

    tray.quit();
    util::log("=== zTranz stopped ===");
    Ok(())
}

fn handle_hotkey(id: i32) {
    match id {
        HK_CAPTURE => main_dispatch::on_capture_requested(),
        HK_TRANSLATE => main_dispatch::on_translate_requested(),
        HK_QUIT => main_dispatch::on_quit_requested(),
        _ => {}
    }
}

fn register_hotkeys(hwnd: windows::Win32::Foundation::HWND) -> Result<(), String> {
    unsafe {
        // Win+Shift+C
        if let Err(e) = RegisterHotKey(Some(hwnd), HK_CAPTURE, MOD_WIN | MOD_SHIFT, 0x43) {
            return Err(format!("Could not register Win+Shift+C ({e})"));
        }
        // Win+Shift+V
        if let Err(e) = RegisterHotKey(Some(hwnd), HK_TRANSLATE, MOD_WIN | MOD_SHIFT, 0x56) {
            return Err(format!("Could not register Win+Shift+V ({e})"));
        }
        // Win+Shift+Q
        if let Err(e) = RegisterHotKey(Some(hwnd), HK_QUIT, MOD_WIN | MOD_SHIFT, 0x51) {
            return Err(format!("Could not register Win+Shift+Q ({e})"));
        }
    }
    Ok(())
}

fn show_error(msg: &str) {
    use windows::core::PCWSTR;
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
            None,
            PCWSTR(util::wide(msg).as_ptr()),
            PCWSTR(util::wide("zTranz — error").as_ptr()),
            windows::Win32::UI::WindowsAndMessaging::MB_OK
                | windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR,
        );
    }
}
