//! Debug helpers: attach a console for stdout/stderr when launched from a
//! terminal, so `zTranz.exe` run from cmd shows its log output.

use std::sync::Once;

static ONCE: Once = Once::new();

pub fn init() {
    ONCE.call_once(|| {
        // Attach to the parent console if we have one (not a GUI subsystem
        // process without a console).
        unsafe {
            let _ = windows::Win32::System::Console::AttachConsole(
                windows::Win32::System::Console::ATTACH_PARENT_PROCESS,
            );
        }
    });
}
