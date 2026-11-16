# ScreenCap Replay — native macOS screen-capture & replay tool

## Context

The user asked for a small macOS tool that records the screen and can replay it
with scrubbing/variable speed and annotation. The repo (zTranz) is an unrelated
**Windows Rust** translation utility, so this ships as a **standalone Swift
project** at `/Volumes/ZMac1TB/Workspace/StudioProjects/ScreenCapReplay` —
a sibling of zTranz, touching nothing in that repo. The environment has no
Rust toolchain but a full Swift 6.4 / Xcode toolchain on macOS 27 arm64, so the
implementation uses **Swift + ScreenCaptureKit + AVFoundation + SpriteKit +
SwiftUI** — zero external dependencies, compiles offline in seconds.

All four features confirmed with the user:
1. Record + replay + scrub + speed presets
2. Annotations (timestamped markers + notes, JSON import/export)
3. Input event log (global keys/mouse during capture, replayed as indicators)
4. PNG frame-sequence export

## Architecture (Swift Package, executable target `ScreenCapReplay`)

Location: `/Volumes/ZMac1TB/Workspace/StudioProjects/ScreenCapReplay` (sibling of
zTranz; the zTranz repo is untouched).

```
ScreenCapReplay/
├── Package.swift                     # executable target, no external deps
└── Sources/ScreenCapReplay/
    ├── main.swift                    # NSApplication bootstrap, app menu, window
    ├── AppModel.swift                # @MainActor @Observable: mode, status, capture settings
    ├── CaptureService.swift          # SCStream wrapper, frame delegate, permission checks
    ├── FrameStore.swift              # frames + timestamps; PNG-sequence export
    ├── ReplayEngine.swift            # SpriteKit scene + playback clock + speed
    ├── ReplayScene.swift             # SKScene drawing the current frame + overlays
    ├── AnnotationModel.swift         # markers, notes, JSON import/export
    ├── InputLog.swift                # NSEvent global monitors → typed events with timestamps
    ├── TimelineView.swift            # SwiftUI: scrubber, transport, speed, marker track
    ├── CaptureView.swift             # SwiftUI: source picker, start/stop, settings
    └── ReplayView.swift              # SwiftUI: SKView + timeline + marker editor
```

### Key design choices

1. **Capture**: `SCStream` with `SCStreamConfiguration` sized from the chosen
   `SCDisplay`, `minimumFrameInterval` = 1/30 s, `queueDepth` 3. The
   `SCStreamOutput` delegate receives `SCStreamFrame`, converts the
   `CVPixelBuffer` to `CGImage` via `CIContext`, and appends `(timestamp, image)`
   to `FrameStore` on the main actor.
2. **Permissions**: `CGPreflightScreenCaptureAccess()` is checked before capture;
   `CGRequestScreenCaptureAccess()` is called on the first capture attempt only.
   Input logging additionally needs Accessibility permission
   (`AXIsProcessTrustedWithOptions`) for global key events — requested lazily with
   a clear explanation, and the UI degrades to mouse-only if it is denied.
   Denials are surfaced in the status bar, never fatal.
3. **Frame store**: an array of `(TimeInterval, CGImage)` plus a derived
   `frameInterval` (median inter-frame delta). Frames are held in memory; a
   1080p frame is ~8 MB, so a 10-minute 30 fps capture would be far too large —
   therefore **PNG-sequence export is the primary persistence path** and the
   store is capped at a configurable max (default 1800 frames / 60 s) with a
   visible "recording will stop" warning. Replay of longer sessions reads back
   from the exported PNG sequence.
4. **Replay clock**: `SKScene.update(_:)` advances a virtual time
   `t += dt * speed`; the displayed frame index is the last frame whose
   timestamp is `<= t`. Scrubbing sets `t` directly. Speed changes take effect
   immediately and do not reset position.
5. **Annotations**: `[Marker]` with `(time, label, note)`; JSON round-trip via
   `Codable`; markers render as ticks on the timeline track and as callouts in
   the scene. Clicking a marker seeks to its time.
6. **Input log**: global `NSEvent.addGlobalMonitorForEvents` for
   `.keyDown`/`.mouseDown`/`.mouseMoved`, plus a local monitor for events while
   the app is focused. Key events are recorded as character + modifier flags
   (never raw keycodes alone, so the log is human-readable); mouse events as
   normalized (0-1) screen coordinates. Each event carries the capture clock
   timestamp, so replay can show what was typed/clicked at that moment.
   The global keyboard monitor is gated behind an explicit opt-in toggle and the
   Accessibility permission prompt.

### Playback of exported sessions

A recorded session is persisted as a directory:
```
<out>/frames/frame_00001.png ...
<out>/session.json     # metadata: fps, frame count, source size, markers
<out>/events.json     # input event log
```
`session.json` carries the markers, so reopening a session restores annotations
without a separate export step.

## Build & run

```
cd /Volumes/ZMac1TB/Workspace/StudioProjects/ScreenCapReplay
swift build
swift run ScreenCapReplay
```

First launch triggers macOS screen-recording permission prompts (System Settings →
Privacy & Security → Screen Recording; Accessibility if the input log is enabled).
Subsequent runs work without prompts.

## Verification plan

1. `swift build` succeeds with no external dependencies.
2. `swift run` opens a window; the capture screen lists the current display.
3. Record a 5-second capture of the desktop; stop; verify the timeline shows ~150
   frames, scrubbing is responsive, and 2× speed plays back correctly.
4. Enable the input log, type a few keys and click; verify events land in
   `events.json` with timestamps, and that replay shows them as on-screen
   indicators near their timestamps.
5. Add two markers with notes; close and reopen the session directory; markers
   reappear at correct times from `session.json`.
6. Export PNG sequence to a temp directory; verify frame count matches and a
   frame decodes with `sips -g pixelWidth`.
7. Quit cleanly; no leaked `SCStream` or `SKScene` (no crashes on exit).
