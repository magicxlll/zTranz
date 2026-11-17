//! Shared state between the UI thread and the worker threads.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use crate::popup::PopupContent;

/// Set by the worker when the popup should be shown/hidden.
pub static SHOW_POPUP: AtomicBool = AtomicBool::new(false);
pub static HIDE_POPUP: AtomicBool = AtomicBool::new(false);
/// Bumped whenever the tray tooltip should be refreshed.
pub static STATUS_DIRTY: AtomicBool = AtomicBool::new(false);

/// The popup content to display.
pub static CONTENT: Mutex<Option<PopupContent>> = Mutex::new(None);

/// The last status string for the tray tooltip.
///
/// Initialized empty; `set_status` populates it. (`String::from` is not a
/// const fn, so it cannot be used in a `static` initializer.)
pub static STATUS: Mutex<String> = Mutex::new(String::new());

/// Monotonic counter used to know the newest request wins.
pub static REQUEST_ID: AtomicU64 = AtomicU64::new(0);

pub fn next_request_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::SeqCst) + 1
}

pub fn publish(content: PopupContent, status: &str) {
    if let Ok(mut c) = CONTENT.lock() {
        *c = Some(content);
    }
    set_status(status);
    SHOW_POPUP.store(true, Ordering::SeqCst);
}

pub fn set_status(s: &str) {
    if let Ok(mut st) = STATUS.lock() {
        *st = s.to_string();
    }
    STATUS_DIRTY.store(true, Ordering::SeqCst);
}

pub fn take_content() -> Option<PopupContent> {
    CONTENT.lock().ok().and_then(|mut c| c.take())
}

pub fn current_status() -> String {
    STATUS.lock().map(|s| s.clone()).unwrap_or_default()
}
