//! OCR of a PNG screenshot via the built-in Windows.Media.Ocr engine
//! (offline, no extra runtime). Must be called from a worker thread.
//!
//! Pipeline (windows crate 0.62.2): open PNG file -> IRandomAccessStream
//! -> BitmapDecoder -> SoftwareBitmap -> OcrEngine::RecognizeAsync.
//! All WinRT async operations are blocked with `windows_future::join()`.

use std::path::Path;

use windows::core::HSTRING;
use windows::Graphics::Imaging::BitmapDecoder;
use windows::Media::Ocr::OcrEngine;
use windows::Storage::{FileAccessMode, StorageFile};

/// Recognize text in a PNG file. Returns the full recognized text.
pub fn ocr_png(path: &Path) -> Result<String, String> {
    // UIAutomation/WinRT COM: make sure the worker thread has an MTA apartment.
    // (Harmless no-op if already initialized.)
    let _ = unsafe {
        windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_MULTITHREADED,
        )
    };

    let path_str = path
        .to_str()
        .ok_or_else(|| format!("screenshot path not valid UTF-8: {path:?}"))?;

    let engine = OcrEngine::TryCreateFromUserProfileLanguages().map_err(|e| {
        format!("no OCR engine available (install a Windows OCR language pack): {e}")
    })?;

    // Open the PNG file directly as an IRandomAccessStream (in windows 0.62.2
    // the sync write path IOutputStream::WriteBytes was removed, so we avoid
    // copying bytes into an in-memory stream).
    let file_h = HSTRING::from(path_str);
    let file = StorageFile::GetFileFromPathAsync(&file_h)
        .and_then(|op| op.join())
        .map_err(|e| format!("cannot open screenshot '{path:?}': {e}"))?;
    let stream = file
        .OpenAsync(FileAccessMode::Read)
        .and_then(|op| op.join())
        .map_err(|e| format!("cannot read screenshot '{path:?}': {e}"))?;

    // Decode PNG -> SoftwareBitmap.
    let decoder = BitmapDecoder::CreateAsync(&stream)
        .and_then(|op| op.join())
        .map_err(|e| format!("cannot open screenshot '{path:?}' as image: {e}"))?;
    let frame = decoder
        .GetFrameAsync(0)
        .and_then(|op| op.join())
        .map_err(|e| format!("cannot decode PNG: {e}"))?;
    let bitmap = frame
        .GetSoftwareBitmapAsync()
        .and_then(|op| op.join())
        .map_err(|e| format!("cannot convert decoded image to SoftwareBitmap: {e}"))?;

    let result = engine
        .RecognizeAsync(&bitmap)
        .and_then(|op| op.join())
        .map_err(|e| format!("OCR failed (install the matching Windows OCR language pack): {e}"))?;

    let text = result
        .Text()
        .map(|h| h.to_string_lossy())
        .map_err(|e| format!("OCR result unreadable: {e}"))?;
    if text.trim().is_empty() {
        Err("no text found in the selected region".into())
    } else {
        Ok(text)
    }
}
