//! Read the current clipboard text, and try to capture the user's
//! text selection in the foreground app via UI Automation.

use windows::Win32::Foundation::{HANDLE, HGLOBAL};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GLOBAL_ALLOC_FLAGS,
};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationTextPattern,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

/// Get the clipboard's current text, if any.
pub fn get_text() -> Option<String> {
    unsafe {
        if OpenClipboard(None).is_err() {
            return None;
        }
        let _guard = ClipboardGuard;
        if IsClipboardFormatAvailable(CF_UNICODETEXT.0 as u32).is_err() {
            return None;
        }
        let h = GetClipboardData(CF_UNICODETEXT.0 as u32).ok()?;
        let hg = HGLOBAL(h.0);
        let size = GlobalSize(hg) as usize;
        let ptr = GlobalLock(hg) as *const u16;
        if ptr.is_null() || size == 0 {
            return None;
        }
        // The buffer is UTF-16 with a NUL terminator.
        let max = size / 2;
        let slice = std::slice::from_raw_parts(ptr, max);
        let len = slice.iter().position(|&c| c == 0).unwrap_or(max);
        let text = String::from_utf16_lossy(&slice[..len]);
        let _ = GlobalUnlock(hg);
        if text.trim().is_empty() {
            None
        } else {
            Some(text)
        }
    }
}

struct ClipboardGuard;

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

/// Replace the clipboard contents with `text`.
pub fn set_text(text: &str) -> bool {
    unsafe {
        if OpenClipboard(None).is_err() {
            return false;
        }
        let _guard = ClipboardGuard;
        let _ = EmptyClipboard();
        let mut wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = wide.len() * 2;
        let flags = GLOBAL_ALLOC_FLAGS(0x0042); // GMEM_MOVEABLE | GMEM_ZEROINIT
        let hg = match GlobalAlloc(flags, bytes) {
            Ok(h) => h,
            Err(_) => return false,
        };
        let dst = GlobalLock(hg) as *mut u16;
        if dst.is_null() {
            return false;
        }
        std::ptr::copy_nonoverlapping(wide.as_mut_ptr(), dst, wide.len());
        let _ = GlobalUnlock(hg);
        let ok = SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(hg.0))).is_ok();
        ok
    }
}

/// Try to read the text the user currently has selected in the foreground
/// application using UI Automation. Returns None when the focused control
/// does not expose a selectable text pattern (common in games, video
/// players, or apps that block accessibility).
pub fn get_selection() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid = 0u32;
        let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));

        let auto: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL).ok()?;
        let root = auto.GetRootElement().ok()?;
        let _ = &root;

        // Walk down from the focused element to find a text pattern with a selection.
        if let Ok(focused) = auto.GetFocusedElement() {
            if let Some(t) = selection_from(&focused) {
                return Some(t);
            }
            let walker = auto.ControlViewWalker().ok()?;
            let mut child = walker.GetFirstChildElement(&focused).ok()?;
            let mut guard = 0;
            loop {
                if let Some(t) = selection_from(&child) {
                    return Some(t);
                }
                child = match walker.GetNextSiblingElement(&child).ok() {
                    Some(c) => c,
                    None => break,
                };
                guard += 1;
                if guard >= 64 {
                    break;
                }
            }
        }
        None
    }
}

unsafe fn selection_from(el: &IUIAutomationElement) -> Option<String> {
    use windows::Win32::UI::Accessibility::UIA_TextPatternId;
    let pattern = el
        .GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId)
        .ok()?;
    let sel = pattern.GetSelection().ok()?;
    let len = sel.Length().ok()?;
    if len == 0 {
        return None;
    }
    let mut out = String::new();
    for i in 0..len {
        let r = sel.GetElement(i).ok()?;
        // Take up to 32k chars per range — far more than any real selection.
        let t = r.GetText(32767).ok()?;
        out.push_str(&String::from_utf16_lossy(t.as_ref()));
    }
    let out = out.trim().to_string();
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}
