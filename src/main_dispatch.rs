//! Bridges UI events (tray, hotkeys, overlay) to the worker pipeline.
//! All translation work happens on a background thread so the UI never
//! blocks — the popup appears as soon as the result is ready.

use std::sync::atomic::Ordering;

use crate::config::Config;
use crate::popup::PopupContent;
use crate::state;

/// Global handle to the overlay, set by main().
pub static mut OVERLAY: Option<crate::overlay::Overlay> = None;

/// Set when the app should quit.
pub static QUIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The region (in virtual-screen coordinates) chosen in the overlay.
static SELECTED_REGION: std::sync::Mutex<Option<(i32, i32, i32, i32)>> =
    std::sync::Mutex::new(None);

/// Called by the overlay when the user finished dragging a region.
pub fn on_region_selected(region: (i32, i32, i32, i32)) {
    if let Ok(mut g) = SELECTED_REGION.lock() {
        *g = Some(region);
    }
    state::set_status("Capturing region…");
    run_capture_job(region);
}

/// Called by the overlay when the drag was too small (i.e. cancelled).
pub fn on_region_cancelled() {
    state::set_status("zTranz — ready");
    state::HIDE_POPUP.store(true, Ordering::SeqCst);
}

/// Triggered by the tray menu / hotkey: translate the current selection or
/// the clipboard contents.
pub fn on_translate_requested() {
    let cfg = Config::load();
    let text = crate::clipboard::get_selection()
        .or_else(crate::clipboard::get_text)
        .unwrap_or_default();

    if text.trim().is_empty() {
        state::publish(
            PopupContent {
                source: String::new(),
                translated: String::new(),
                status: "Nothing to translate — select text or copy it first.".into(),
            },
            "zTranz — nothing selected",
        );
        return;
    }
    run_translate_job(cfg, text, "selection");
}

/// Triggered by the tray menu / hotkey: open the region-select overlay.
pub fn on_capture_requested() {
    let geom = crate::capture::virtual_screen();
    // Copy the overlay out atomically (static mut cannot be shared-referenced).
    unsafe {
        if let Some(o) = std::ptr::read(&raw const OVERLAY) {
            o.show(&geom);
            state::set_status("Drag to select a region…");
        }
    }
}

pub fn on_quit_requested() {
    QUIT.store(true, Ordering::SeqCst);
    unsafe {
        windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
    }
}

// ---------------------------------------------------------------- jobs

fn run_capture_job(region: (i32, i32, i32, i32)) {
    let cfg = Config::load();
    let req = state::next_request_id();

    std::thread::Builder::new()
        .name("ztranz-capture".into())
        .spawn(move || {
            let raw = match crate::capture::grab_virtual_screen() {
                Ok(r) => r,
                Err(e) => {
                    fail(req, &format!("Screen capture failed: {e}"));
                    return;
                }
            };

            let tmp = crate::util::temp_dir().join(format!("ztranz-{}.png", req));
            if let Err(e) = crate::capture::crop_to_png(&raw, region, &tmp) {
                fail(req, &format!("Crop failed: {e}"));
                return;
            }

            state::set_status("Recognising text…");
            let text = match crate::ocr::ocr_png(&tmp) {
                Ok(t) => t,
                Err(e) => {
                    let _ = std::fs::remove_file(&tmp);
                    fail(
                        req,
                        &format!("{e}\n\nTip: Windows OCR needs the language pack for the text's language\n(Settings → Time & language → Language → Language options → Basic typing/Optical character recognition)."),
                    );
                    return;
                }
            };
            let _ = std::fs::remove_file(&tmp);

            state::set_status("Translating…");
            let mut c = cfg.clone();
            if c.source_lang == "auto" {
                c.source_lang = crate::translate::detect_lang(&text);
            }
            finish(req, text, c, "screenshot");
        })
        .ok();
}

fn run_translate_job(cfg: Config, text: String, origin: &str) {
    let req = state::next_request_id();
    let origin = origin.to_string();
    std::thread::Builder::new()
        .name("ztranz-translate".into())
        .spawn(move || {
            state::set_status("Translating…");
            let mut c = cfg.clone();
            if c.source_lang == "auto" {
                c.source_lang = crate::translate::detect_lang(&text);
            }
            finish(req, text, c, &origin);
        })
        .ok();
}

fn finish(req: u64, text: String, cfg: Config, origin: &str) {
    // If a newer request arrived while we were working, drop this result.
    if state::REQUEST_ID.load(Ordering::SeqCst) != req {
        return;
    }
    match crate::translate::translate(&text, &cfg) {
        Ok(t) => {
            let status = format!(
                "{} → {}  ·  {}  ·  {}",
                cfg.source_lang, cfg.target_lang, cfg.provider, origin
            );
            state::publish(
                PopupContent {
                    source: text,
                    translated: t,
                    status,
                },
                "zTranz — ready",
            );
        }
        Err(e) => fail(req, &e),
    }
}

fn fail(req: u64, message: &str) {
    if state::REQUEST_ID.load(Ordering::SeqCst) != req {
        return;
    }
    crate::util::log(&format!("error: {message}"));
    state::publish(
        PopupContent {
            source: String::new(),
            translated: String::new(),
            status: message.to_string(),
        },
        "zTranz — error",
    );
}
