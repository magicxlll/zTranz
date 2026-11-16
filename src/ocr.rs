//! OCR of a PNG screenshot via the built-in Windows.Media.Ocr engine
//! (offline, no extra runtime). Must be called from a worker thread.

use std::path::Path;
use windows::Foundation::Future;
use windows::Media::Ocr::OcrEngine;
use windows::Storage::StorageFile;

/// Recognize text in a PNG file. Returns the full recognized text.
pub fn ocr_png(path: &Path) -> Result<String, String> {
    let engine = OcrEngine::TryCreate().map_err(|e| format!("no OCR engine available: {e}"))?;

    let file = StorageFile::OpenFileAsync(&path.to_string_lossy())
        .join()
        .map_err(|e| format!("cannot open screenshot '{path}': {e}"))?;

    let result = engine
        .RecognizeAsyncWithLanguage(&file, None)
        .join()
        .map_err(|e| format!("OCR failed (install the matching Windows OCR language pack): {e}"))?;

    let text = result.Text().into();
    if text.is_empty() {
        Err("no text found in the selected region".into())
    } else {
        Ok(text)
    }
}
