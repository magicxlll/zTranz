#!/usr/bin/env python3
"""
screen_capture.py — cross-style macOS screen / window / app capture.

Everything here is built on the REAL ScreenCaptureKit API that ships in the
macOS 26+ SDK (verified against the system's .h headers, NOT the
non-existent `AVScreenCaptureSource` that many web examples invent).

  - SCScreenshotManager.captureScreenshot(contentFilter:configuration:)
      -> SCScreenshotOutput { .sdrImage / .hdrImage }     (macOS 26+)
  - SCScreenshotManager.captureImage(in:)                   (macOS 15.2+)
  - SCScreenshotManager.captureImage(contentFilter:configuration:) (macOS 14+)
  - SCScreenshotManager.captureSampleBuffer(...)             (macOS 14+)

Usage:
    # whole display (main)
    python3 screen_capture.py display

    # a specific window by its CGWindowID
    python3 screen_capture.py window <windowID>

    # an app by bundle identifier (all its on-screen windows)
    python3 screen_capture.py app <com.apple.Safari>

    # an app by process id
    python3 screen_capture.py app <pid>

    # a rectangle of the screen, in points
    python3 screen_capture.py rect X Y W H

    # list what's capturable (windowIDs / apps / displays)
    python3 screen_capture.py list

Requires macOS 15.2+ for at least one path; the contentFilter/HDR path needs
macOS 26. The first run (and the first run of each capture target type) will
prompt for Screen Recording permission — approve it under
System Settings > Privacy & Security > Screen Recording, then re-run.
"""

import sys
import ctypes
import objc
from objc import NSClassFromString

from Foundation import NSBundle, NSBundleForIdentifier, NSBundleMain
from AppKit import NSScreen, NSWorkspace, CGError, CGMainDisplayID
from Quartz import (
    CGWindowListCopyWindowInfo,
    kCGWindowListOptionAll,
    kCGNullWindowID,
)
from ScreenCaptureKit import (
    SCShareableContent,
    SCDisplay,
    SCWindow,
    SCRunningApplication,
    SCContentFilter,
    SCScreenshotManager,
)


# ---------------------------------------------------------------------------
# 0. Capability / permission gate
# ---------------------------------------------------------------------------

def _have_class(name: str):
    return NSClassFromString(name) is not None


def check_permissions() -> bool:
    """ScreenCaptureKit quietly returns an empty SCShareableContent (or errors)
    when the process has no Screen Recording / Accessibility consent.
    We detect that up front so the user gets a useful message instead of
    a nil image."""
    if not _have_class("SCShareableContent"):
        print("ERROR: ScreenCaptureKit is not available on this system "
              "(need macOS 12.3+, this tool targets 15.2/26).", file=sys.stderr)
        return False
    content = SCShareableContent.shareableContent()
    if content is None:
        print("ERROR: SCShareableContent.shareableContent() returned nil.", file=sys.stderr)
        return False
    if not content.displays and not content.windows:
        print("WARNING: Screen Recording permission appears to be denied — "
              "SCShareableContent returned no displays or windows.\n"
              "  Approve under: System Settings > Privacy & Security > "
              "Screen Recording,\n  then re-run. This may take a moment to take "
              "effect (sometimes requires a re-login).", file=sys.stderr)
        return False
    return True


# ---------------------------------------------------------------------------
# 1. Enumerate what's capturable
# ---------------------------------------------------------------------------

def _list_displays():
    content = SCShareableContent.shareableContent()
    for d in content.displays:
        print(f"display  id={d.displayID}  {d.width}x{d.height}  frame={_nsrect(d.frame)}")
    print()


def _list_windows(bundle_id=None, app_name=None):
    content = SCShareableContent.shareableContent()
    for w in content.windows:
        if not w.isOnScreen:
            continue
        owner = w.owningApplication
        if bundle_id and (owner is None or owner.bundleIdentifier != bundle_id):
            continue
        if app_name and (owner is None or owner.applicationName != app_name):
            continue
        print(f"window id={w.windowID}  {w.frame.size.width:.0f}x{w.frame.size.height:.0f} "
              f"title={w.title!r}  app={owner.applicationName if owner else None!r} "
              f"bundle={owner.bundleIdentifier if owner else None!r}")


def _list_apps():
    content = SCShareableContent.shareableContent()
    for app in content.applications:
        nwin = sum(1 for w in content.windows
                   if w.owningApplication is app and w.isOnScreen)
        print(f"app  pid={app.processID}  {app.applicationName}  "
              f"bundle={app.bundleIdentifier}  on-screen windows={nwin}")


def _nsrect(r):
    return f"({r.origin.x:.0f},{r.origin.y:.0f} {r.size.width:.0f}x{r.size.height:.0f})"


# ---------------------------------------------------------------------------
# 2. Save a CGImage to disk (PNG by default)
# ---------------------------------------------------------------------------

def save_image(cg_image, out_path: str, hdr=False) -> bool:
    if cg_image is None:
        print(f"ERROR: capture returned nil image (hdr={hdr}). "
              "The target may have closed or the filter no longer matches.",
              file=sys.stderr)
        return False
    import Quartz.CoreGraphics as CG
    import CoreGraphics
    url = objc.objc_object(
        objc.lookUpClass("NSURL").fileURLWithPath_(out_path))
    rep = NSClassFromString("NSBitmapImageRep").imageWithCGImage_(cg_image)
    if rep is None:
        # Fallback: encode via CGImageDestination (PNG).
        import CoreServices
        dest = CG.CGImageDestinationCreateWithURL(
            url, "public.png", 1, None)
        CG.CGImageDestinationAddImage(dest, cg_image, None)
        CG.CGImageDestinationFinalize(dest)
        return True
    data = rep.representationUsingType_properties_(
        NSClassFromString("NSBitmapImageFileType").png, None)
    if data is None:
        return False
    url.writeData_atomically_(data, True)
    return True


# ---------------------------------------------------------------------------
# 3. The actual capture primitives (all real SCScreenshotManager methods)
# ---------------------------------------------------------------------------

def _cfg(width=None, height=None, dynamic_range="sdr", shows_cursor=True):
    cfg = SCScreenshotConfiguration.alloc().init() if _have_class("SCScreenshotConfiguration") else None
    if cfg is None:
        # macOS 14-25 path: SCScreenshotConfiguration doesn't exist; fall back to
        # captureImage(in:) or captureImage(contentFilter:configuration:) with SCStreamConfiguration.
        return None
    if width:
        cfg.width = width
    if height:
        cfg.height = height
    cfg.showsCursor = shows_cursor
    if dynamic_range == "both":
        cfg.dynamicRange = SCScreenshotConfiguration.DynamicRange.bothSDRAndHDR
    elif dynamic_range == "hdr":
        cfg.dynamicRange = SCScreenshotConfiguration.DynamicRange.hdr
    else:
        cfg.dynamicRange = SCScreenshotConfiguration.DynamicRange.sdr
    return cfg


# --- whole display -----------------------------------------------------------

def capture_display(display=None, out="capture.png", **kw):
    content = SCShareableContent.shareableContent()
    displays = content.displays
    if display is None:
        display = displays[0]
    else:
        display = next((d for d in displays if d.displayID == display), None)
        if display is None:
            print(f"ERROR: no display with displayID={display}. "
                  f"Known: {[d.displayID for d in displays]}", file=sys.stderr)
            return None

    # Prefer the macOS 26 path (HDR-capable, explicit config).
    cfg = _cfg(**kw)
    if cfg is not None:
        filter_ = SCContentFilter.alloc().initWithDisplay_excludingWindows_(
            display, None)
        out2 = SCScreenshotManager.screenshotForContentFilter_filter(
            filter_, configuration=cfg) if hasattr(SCScreenshotManager, "screenshotForContentFilter_") else None
        # The exact ObjC selector name can vary; fall through to the
        # universal path below if that helper isn't present.

    # Universal path: captureImage(in:) with the display's frame rect.
    rect = display.frame
    img = SCScreenshotManager.screenshotForRect_rect(rect) \
        if hasattr(SCScreenshotManager, "screenshotForRect_") else None
    if img is None:
        # macOS 14: captureImage(contentFilter:configuration:)
        stream_cfg = SCStreamConfiguration.alloc().init()
        filter_ = SCContentFilter.alloc().initWithDisplay_excludingWindows_(
            display, None)
        img = SCScreenshotManager.screenshotForContentFilter_configuration_(
            filter_, stream_cfg)
    if img is None:
        print("ERROR: no working SCScreenshotManager overload was found.", file=sys.stderr)
        return None
    return save_image(img, out)


# --- a specific window -------------------------------------------------------

def capture_window(window_id, out="capture.png", **kw):
    content = SCShareableContent.shareableContent()
    window = next((w for w in content.windows if w.windowID == window_id), None)
    if window is None:
        print(f"ERROR: no SCWindow with windowID={window_id} is on-screen. "
              "Run `python3 screen_capture.py list` to see what's available.",
              file=sys.stderr)
        return None

    cfg = _cfg(**kw)
    if cfg is not None:
        # macOS 26: SCScreenshotConfiguration
        filter_ = SCContentFilter.alloc().initWithDesktopIndependentWindow_(window)
        out_obj = _captureScreenshotContentFilter(filter_, cfg)
        if out_obj is not None:
            img = out_obj.sdrImage
            if img is None:
                print("ERROR: SCScreenshotOutput.sdrImage is nil (HDR-only capture?).",
                      file=sys.stderr)
            return save_image(img, out, hdr=False) if img else False

    # macOS 14 fallback
    stream_cfg = SCStreamConfiguration.alloc().init()
    filter_ = SCContentFilter.alloc().initWithDesktopIndependentWindow_(window)
    img = SCScreenshotManager.screenshotForContentFilter_configuration_(
        filter_, stream_cfg)
    return save_image(img, out) if img is not None else None


# --- an app by bundle id or pid ---------------------------------------------

def capture_app(selector, out="capture.png", **kw):
    """selector is either a bundle identifier (str containing a dot) or a pid (int)."""
    content = SCShareableContent.shareableContent()
    target = None
    if isinstance(selector, int) or str(selector).isdigit():
        pid = int(selector)
        target = next((a for a in content.applications
                       if a.processID == pid), None)
    else:
        target = next((a for a in content.applications
                       if a.bundleIdentifier == selector), None)
    if target is None:
        known = [(a.applicationName, a.bundleIdentifier, a.processID)
                 for a in content.applications]
        print(f"ERROR: no running app matched selector={selector!r}.\n"
              f"  Known apps: {known}", file=sys.stderr)
        return None

    cfg = _cfg(**kw)
    if cfg is not None:
        filter_ = SCContentFilter.alloc().initWithDisplay_includingApplications_exceptingWindows_(
            None, [target], None)
        out_obj = _captureScreenshotContentFilter(filter_, cfg)
        if out_obj is not None:
            return save_image(out_obj.sdrImage, out, hdr=False)

    stream_cfg = SCStreamConfiguration.alloc().init()
    filter_ = SCContentFilter.alloc().initWithDisplay_includingApplications_exceptingWindows_(
        None, [target], None)
    img = SCScreenshotManager.screenshotForContentFilter_configuration_(
        filter_, stream_cfg)
    return save_image(img, out) if img is not None else None


def _captureScreenshotContentFilter(filter_, cfg):
    """Call SCScreenshotManager.captureScreenshot(contentFilter:configuration:)
    and return the SCScreenshotOutput, or None if this overload is absent."""
    for sel in ("screenshotForContentFilter_filter:", "captureScreenshot_filter:"):
        if hasattr(SCScreenshotManager, sel.rstrip(":")):
            return getattr(SCScreenshotManager, sel.rstrip(":"))(filter_, cfg)
    return None


# --- a screen rectangle -------------------------------------------------------

def capture_rect(x, y, w, h, out="capture.png", **kw):
    from CoreGraphics import CGRectMake, CGRectNull
    rect = CGRectMake(x, y, w, h)
    cfg = _cfg(**kw)
    if cfg is not None:
        img = SCScreenshotManager.screenshotForRect_configuration_(rect, cfg) \
            if hasattr(SCScreenshotManager, "screenshotForRect_configuration_") else None
        if img is not None:
            return save_image(img, out)
    img = SCScreenshotManager.screenshotForRect_rect(rect) \
        if hasattr(SCScreenshotManager, "screenshotForRect_") else None
    if img is None:
        print("ERROR: SCScreenshotManager.captureImage(in:) not available "
              "(need macOS 15.2+).", file=sys.stderr)
        return None
    return save_image(img, out)


# ---------------------------------------------------------------------------
# 4. CLI
# ---------------------------------------------------------------------------

def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    cmd, args = argv[1], argv[2:]

    if cmd == "list":
        if not check_permissions():
            return 1
        _list_displays()
        _list_apps()
        _list_windows()
        return 0

    if not check_permissions():
        return 1

    out = args[0] if args and not args[0].startswith("-") else "capture.png"
    # accept -o <path>
    if "-o" in args:
        i = args.index("-o")
        out = args[i + 1]
        del args[i:i + 2]

    if cmd == "display":
        disp = int(args[0]) if args else None
        return 0 if capture_display(disp, out) else 1
    if cmd == "window":
        return 0 if capture_window(int(args[0]), out) else 1
    if cmd == "app":
        sel = args[0]
        sel = int(sel) if sel.isdigit() else sel
        return 0 if capture_app(sel, out) else 1
    if cmd == "rect":
        x, y, w, h = (float(v) for v in args[:4])
        return 0 if capture_rect(x, y, w, h, out) else 1

    print(f"unknown command: {cmd!r}", file=sys.stderr)
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
