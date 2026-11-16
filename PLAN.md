# zTranz Implementation Plan

## Phase 0 — Baseline and guardrails

- Verify the working tree and GitHub remote `https://github.com/magicxlll/zTranz.git`.
- Establish a clean baseline commit after reviewing existing files.
- Document the Windows Rust toolchain and a reproducible build/check command.
- Treat `PROJECT_MEMORY.md`, `SPEC.md`, `PLAN.md`, and `TASKS.md` as the source of project context.

## Phase 1 — Compile and core-flow audit

- Run `cargo fmt --check`, `cargo check`, and a Windows-target release build where available.
- Inspect module boundaries: hotkeys, dispatch, clipboard, capture, OCR, translation, popup, tray, state, and config.
- Fix compile errors and obvious thread/UI ownership problems without changing scope.
- Verify every acceptance criterion has an implementation path or a tracked task.

## Phase 2 — Selection and clipboard translation

- Make selected-text acquisition deterministic and preserve clipboard contents when temporarily using it.
- Validate empty input, language values, provider selection, HTTP status, response shape, and timeout/error messages.
- Ensure worker results are delivered once and stale results cannot overwrite a newer request.
- Test Google and OpenAI-compatible request/response paths with controlled fixtures where possible.

## Phase 3 — OCR region translation

- Verify virtual-screen coordinates, DPI/multi-monitor behavior, selection cancellation, PNG capture, and cleanup.
- Verify Windows OCR language handling and empty-result behavior.
- Pass OCR output through the same translation pipeline as clipboard text.
- Keep the overlay and popup owned by the UI thread; keep OCR/network work off it.

## Phase 4 — UX and configuration hardening

- Review popup sizing, long text, Unicode, copy action, hide/close behavior, and error visibility.
- Validate configuration migration, invalid values, permissions, atomic-ish writes, and secret handling.
- Improve tray status so translating/success/failure states are truthful.
- Add logging that is useful for diagnosis but never logs API keys.

## Phase 5 — Release verification

- Run formatting, check, release build, and available tests on Windows.
- Perform a manual smoke matrix for both hotkeys, providers, OCR, popup, restart persistence, and quit.
- Build/package the Windows executable and record exact artifact/version information.
- Update project memory with results, remaining risks, and next action.
- Commit and push the reviewed source to the GitHub remote when credentials and policy permit; verify the remote result.

## Scope checkpoint

Before every phase and code change, ask: does this directly improve translating selectable or on-screen text on Windows? If not, stop and add an explicit requirement before proceeding.
