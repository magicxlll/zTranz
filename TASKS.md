# zTranz Tasks

Legend: `[ ]` not started, `[-]` in progress, `[x]` verified complete, `[?]` blocked/unknown.

## Baseline

- [ ] Confirm repository root and initialize/attach Git metadata if needed.
- [ ] Add/verify GitHub remote: `https://github.com/magicxlll/zTranz.git`.
- [ ] Identify a Windows build host/toolchain and record the command.
- [ ] Run and record `cargo fmt --check`, `cargo check`, and release build.

## Core translation

- [ ] Audit selection acquisition and clipboard fallback.
- [ ] Audit worker dispatch and result ordering.
- [ ] Validate Google provider request, response, and failures.
- [ ] Validate OpenAI-compatible provider request, response, and failures.
- [ ] Confirm language and style configuration behavior.

## OCR flow

- [ ] Test overlay on one monitor and multiple monitors.
- [ ] Test DPI scaling and negative virtual-screen coordinates.
- [ ] Test cancellation and invalid/empty selection.
- [ ] Test OCR language-pack errors and empty OCR result.
- [ ] Route OCR text through the shared translation pipeline.

## UX, safety, release

- [ ] Review popup long-text rendering and copy/hide behavior.
- [ ] Ensure API keys never enter logs or normal UI output.
- [ ] Verify config persistence and invalid-config recovery.
- [ ] Add focused tests/fixtures for provider parsing and config merge where practical.
- [ ] Manual smoke test all acceptance criteria on Windows.
- [ ] Update `PROJECT_MEMORY.md` after every debug session.
- [ ] Commit and push a verified backup after each debug session when authorized.

## Explicitly excluded

- [ ] Do not implement screen recording, video replay, or `ScreenCapReplay` workflows.
