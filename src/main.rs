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
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, MOD_SHIFT, MOD_WIN};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, PeekMessageW, TranslateMessage, MSG, MWMO_INPUTAVAILABLE, PM_REMOVE,
    WM_HOTKEY,
};

const HK_CAPTURE: i32 = 1; // Win+Shift+C → region select
const HK_TRANSLATE: i32 = 2; // Win+Shift+V → translate selection/clipboard
const HK_QUIT: i32 = 3; // Win+Shift+Q → quit

fn main() {
    trace::init();
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
                let _ = TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
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

            // Wait up to 30 ms for a message, then loop.
            let _ = windows::Win32::UI::WindowsAndMessaging::MsgWaitForMultipleObjectsEx(
                None,
                30,
                windows::Win32::UI::WindowsAndMessaging::QS_ALLINPUT,
                MWMO_INPUTAVAILABLE,
            );
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

/// Register the global hotkeys. A single hotkey that is already claimed by
/// another application (or a system service) must not prevent zTranz from
/// starting — the tray menu and right-click actions still work — so we log a
/// warning and skip the conflicting key rather than aborting the whole app.
fn register_hotkeys(hwnd: windows::Win32::Foundation::HWND) -> Result<(), String> {
    unsafe {
        let (name, id, vk) = ("Win+Shift+C", HK_CAPTURE, 0x43);
        reg_one(hwnd, id, vk, name);
        let (name, id, vk) = ("Win+Shift+V", HK_TRANSLATE, 0x56);
        reg_one(hwnd, id, vk, name);
        let (name, id, vk) = ("Win+Shift+Q", HK_QUIT, 0x51);
        reg_one(hwnd, id, vk, name);
    }
    Ok(())
}

unsafe fn reg_one(hwnd: windows::Win32::Foundation::HWND, id: i32, vk: u32, name: &str) {
    if let Err(e) = RegisterHotKey(Some(hwnd), id, MOD_WIN | MOD_SHIFT, vk) {
        crate::util::log(&format!(
            "hotkey {name} unavailable ({e}) — it may be in use by another app; \
             use the tray menu instead"
        ));
    }
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
