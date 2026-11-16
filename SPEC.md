# zTranz Specification

## Goal

zTranz is a Windows desktop utility that translates text anywhere with minimal interruption. It runs in the system tray and supports both directly selectable text and text visible only in pixels.

## In scope

1. **Selection/clipboard translation** — `Win+Shift+V` reads selected text when possible, otherwise clipboard text, translates it, and shows the result.
2. **Screen-region OCR translation** — `Win+Shift+C` opens a drag-selection overlay, captures the selected region, runs Windows OCR, translates recognized text, and shows the result.
3. **Providers** — Google translation and an OpenAI-compatible LLM provider, with configurable endpoint, model, API key, and style.
4. **Presentation** — tray status plus a popup/overlay result that can display source, translation, and errors.
5. **Configuration** — source language, target language, provider, LLM options, style, auto-language flag, and remembered capture region.
6. **Offline boundaries** — OCR uses the local Windows OCR engine; network translation is only used by the selected translation provider.

## Out of scope

Screen recording, video replay, annotation/review workflows, timeline tools, arbitrary document editing, and unrelated `ScreenCapReplay` functionality.

## Functional acceptance criteria

- App starts without a console and remains available through the tray.
- Hotkeys do not block the UI thread while clipboard, OCR, or network work runs.
- Selection/clipboard translation reports useful errors for empty input, unavailable clipboard, provider failure, and malformed response.
- OCR flow handles cancellation, empty OCR output, missing language pack, capture failure, and provider failure.
- Popup remains readable, can be hidden, and does not silently lose the latest result.
- Configuration survives restart and does not expose the LLM key in normal UI/log output.
- Build/check can be reproduced on the documented Windows Rust toolchain.

## Non-functional constraints

- Windows-only implementation using the existing Rust/Windows APIs.
- Keep the UI responsive; worker threads own slow operations.
- Avoid adding dependencies unless they directly support this goal.
- Never claim a build or runtime test passed unless it was actually run.
