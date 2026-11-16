//! Read the current clipboard text, and try to capture the user's
//! text selection in the foreground app via UI Automation.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::DataExchange::{
    CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard, CF_UNICODETEXT,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock, HGLOBAL};

/// Get the clipboard's current text, if any.
pub fn get_text() -> Option<String> {
    unsafe {
        if OpenClipboard(None).is_err() {
            return None;
        }
        let _guard = ClipboardGuard;
        if IsClipboardFormatAvailable(CF_UNICODETEXT).is_err() {
            return None;
        }
        let h = GetClipboardData(CF_UNICODETEXT).ok()?;
        let h = HGLOBAL(h.0);
        let size = GlobalSize(h) as usize;
        let ptr = GlobalLock(h) as *const u16;
        if ptr.is_null() || size == 0 {
            return None;
        }
        // The buffer is UTF-16 with a NUL terminator.
        let max = size / 2;
        let slice = std::slice::from_raw_parts(ptr, max);
        let len = slice.iter().position(|&c| c == 0).unwrap_or(max);
        let text = String::from_utf16_lossy(&slice[..len]);
        let _ = GlobalUnlock(h);
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
        let _ = windows::Win32::System::DataExchange::EmptyClipboard();
        let mut wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = wide.len() * 2;
        let hg = windows::Win32::System::Memory::GlobalAlloc(
            windows::Win32::System::Memory::GLOBAL_ALLOC_FLAGS(0x0042), // GMEM_MOVEABLE | GMEM_ZEROINIT
            bytes,
        );
        let hg = match hg {
            Ok(h) => h,
            Err(_) => return false,
        };
        let dst = GlobalLock(hg) as *mut u16;
        if dst.is_null() {
            return false;
        }
        std::ptr::copy_nonoverlapping(wide.as_mut_ptr(), dst, wide.len());
        let _ = GlobalUnlock(hg);
        let ok = windows::Win32::System::DataExchange::SetClipboardData(
            CF_UNICODETEXT,
            Some(HANDLE(hg.0)),
        )
        .is_ok();
        ok
    }
}

/// Try to read the text the user currently has selected in the foreground
/// application using UI Automation. Returns None when the focused control
/// does not expose a selectable text pattern (common in games, video
/// players, or apps that block accessibility).
pub fn get_selection() -> Option<String> {
    unsafe {
        use windows::Win32::UI::Accessibility::{
            IUIAutomationElement, UIA_IsTextPatternAvailablePropertyId, UIA_TextPatternId,
        };
        use windows::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId,
        };

        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        let auto = IUIAutomationElement::RootElement().ok()?;
        let cond = auto
            .CreatePropertyCondition(
                UIA_IsTextPatternAvailablePropertyId,
                windows::core::VARIANT::from(true),
            )
            .ok()?;
        let _ = &cond;
        let el = auto.ElementFromHandle(hwnd).ok()?;
        let _ = &el;

        // Walk down from the focused element to find a text pattern with a selection.
        if let Ok(focused) = auto.GetFocusedElement() {
            if let Some(t) = selection_from(&focused) {
                return Some(t);
            }
            let mut walker = focused.ControlViewWalker().ok()?;
            let mut child = walker.GetFirstChildElement().ok()?;
            let mut guard = 0;
            while child.is_some() && guard < 64 {
                if let Some(t) = selection_from(&child.unwrap()) {
                    return Some(t);
                }
                child = walker.GetNextSiblingElement().ok()?;
                guard += 1;
            }
        }
        None
    }
}

unsafe fn selection_from(
    el: &windows::Win32::UI::Accessibility::IUIAutomationElement,
) -> Option<String> {
    use windows::Win32::UI::Accessibility::{IUIAutomationTextPattern, UIA_TextPatternId};
    let pat = el.GetCurrentPattern(UIA_TextPatternId).ok()?;
    let tp: IUIAutomationTextPattern = pat.cast()?;
    let sel = tp.GetSelection().ok()?;
    if sel.Length() == 0 {
        return None;
    }
    let mut out = String::new();
    for i in 0..sel.Length() {
        let r = sel.GetElement(i).ok()?;
        let t = r.CurrentName().ok()?;
        out.push_str(&t);
    }
    let out = out.trim().to_string();
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Utility for building wide strings from OsString-ish values.
pub fn to_wide(s: &OsString) -> Vec<u16> {
    s.clone().encode_wide().chain(std::iter::once(0)).collect()
}
