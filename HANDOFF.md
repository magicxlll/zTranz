# HANDOFF — Giao việc cho AI-agent trên máy Windows

> Cách dùng: mở một AI-agent có quyền chỉnh sửa file & chạy terminal TRÊN MÁY WINDOWS
> (đã đăng nhập tài khoản GitHub `magicxlll`), rồi dán nguyên khối phần PROMPT bên dưới.
> Agent sẽ clone repo, đọc `PROJECT_MEMORY.md`, build/debug/smoke test rồi commit/push lại.

---

## PROMPT (copy từ đây xuống)

```text
Bạn là AI-agent làm việc trên một máy Windows có quyền clone/push với tài khoản
GitHub `magicxlll`. Nhiệm vụ: clone mã nguồn dự án zTranz, đọc bộ nhớ dự án,
triển khai build thành công ra .exe trên chính máy Windows này, debug từng lỗi,
rồi cập nhật lại bộ nhớ dự án. Làm đúng thứ tự dưới đây và báo cáo trung thực.

== 1. Clone mã nguồn ==
- Repository: https://github.com/magicxlll/zTranz.git (branch main)
- `git clone https://github.com/magicxlll/zTranz.git` rồi làm việc trong thư mục `zTranz/`.

== 2. Đọc trước khi làm bất cứ gì ==
- Đọc TOÀN BỘ `PROJECT_MEMORY.md`. Đây là bộ nhớ bắt buộc.
- Mục tiêu cốt lõi: zTranz là "ứng dụng dịch MỌI NƠI trên Windows" — dịch văn bản
  đang chọn, nội dung clipboard, hoặc văn bản trên màn hình (OCR khi không chọn được),
  chạy nền qua system tray, hotkey/overlay, popup bản dịch.
- Răn: KHÔNG được biến dự án thành app ghi màn hình / replay / annotation video.
  `src/ScreenCapReplay-plan.md` là tài liệu LẠC PHẠM VI, không dùng làm spec.

== 3. Xác minh toolchain trên Windows (làm thật, không suy đoán) ==
- `cargo --version`, `rustc --version`. Nếu thiếu thì cài Rust.
- Target MSVC: `rustup target list --installed` phải có `x86_64-pc-windows-msvc`.
  Thiếu thì `rustup target add x86_64-pc-windows-msvc`.
- Cần MSVC Build Tools / Windows SDK (để link). Nếu build báo thiếu `link.exe` /
  SDK, cài "Visual Studio Build Tools" (workload C++) rồi mở lại shell.

== 4. Build + test (chạy thật, ghi kết quả) ==
- Có sẵn script: chạy `scripts\test-windows.ps1` (Windows PowerShell 5.1) — nó đã làm
  fmt, check, test, build + xác nhận .exe theo thứ tự đúng.
- Hoặc làm tay:
    cargo fmt --all -- --check
    cargo check --target x86_64-pc-windows-msvc
    cargo test  --target x86_64-pc-windows-msvc
    cargo build --release --target x86_64-pc-windows-msvc
- Xác nhận file `.exe` thật sự tạo ra (kích thước, đường dẫn) TRƯỚC khi tuyên bố
  "đã build thành công". Cấm nói build xong nếu chưa thấy .exe.

== 5. Debug từng module theo lỗi compile/runtime ==
Cấu trúc dự án (Rust + crate `windows` 0.62.2 + Win32):
- entry: `src/main.rs`, điều phối: `src/main_dispatch.rs`
- dịch + provider: `src/translate.rs`, `src/http.rs`
- OCR (Windows.Media.Ocr): `src/ocr.rs`, capture: `src/capture.rs`
- UI: `src/tray.rs`, `src/popup.rs`, `src/overlay.rs`
- clipboard/UIA: `src/clipboard.rs`
- state/config: `src/state.rs`, `src/config.rs`
- helper OCR (không phải recorder): `screen_capture.py`
Hotkey: Win+Shift+C chọn vùng OCR, Win+Shift+V dịch selection/clipboard, Win+Shift+Q thoát.
Mỗi lần sửa: build lại + kiểm tra. Không sửa lan man ngoài mục tiêu "dịch mọi nơi".

== 6. Smoke test GUI (nếu .exe chạy được) ==
- Chạy zTranz, kiểm tra system tray hiện, hotkey hoạt động, OCR vùng chọn + popup
  bản dịch. Có hướng dẫn trong `WINDOWS-REAL.md` (mục GUI smoke test).

== 7. Cập nhật bộ nhớ + commit/push (máy này có quyền push) ==
- Sửa `PROJECT_MEMORY.md`: ghi trạng thái mới nhất (file đã đổi, build thành công/lỗi
  cụ thể, lỗi còn lại, bước tiếp theo, bài học). Ghi KẾT QUẢ THẬT, không tô vẽ.
- `git add` + `git commit` (message rõ) + `git push origin main`.
- Xác minh push thành công (thấy SHA mới trên remote), rồi mới báo "đã push".

== Quy tắc cứng ==
- Đọc file → build → test → ghi kết quả TRUNG THỰC; không khai báo thao tác chưa chạy.
- Không mở rộng phạm vi sang ghi màn hình/replay.
- Trước khi kết thúc, để lại trong PROJECT_MEMORY.md đủ context để phiên khác
  (hoặc bạn) tiếp tục ngay: hiện trạng, file đổi, kết quả build/test, việc kế tiếp.
```

---

## Ghi chú cho người dùng (không cần dán cho agent)

- Remote `main` (SHA ~`2071533`) đã có đủ: mã nguồn + `PROJECT_MEMORY.md` +
  `scripts/test-windows.ps1` + `WINDOWS-REAL.md` + `plan-build-windows.yml.md`.
- File `.github/workflows/test-windows-real.yml` KHÔNG có trên remote (token GitHub
  trên Mac thiếu scope `workflow`) — không ảnh hưởng, vì agent build cục bộ,
  không cần GitHub Actions.
- Agent Windows push xong, phía Mac chỉ cần `git pull` là có phiên bản mới.
