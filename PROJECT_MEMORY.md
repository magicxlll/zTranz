---
name: ztranz-project-memory
description: Bộ nhớ dài hạn bắt buộc cho dự án zTranz — ứng dụng dịch mọi nơi trên Windows.
metadata:
  type: project
---

# zTranz — PROJECT MEMORY

## MỤC TIÊU CHÍNH — BẮT BUỘC ĐỌC TRƯỚC KHI LÀM VIỆC

Xây dựng **zTranz: ứng dụng dịch mọi nơi trên Windows**. Người dùng có thể dịch văn bản đang chọn, nội dung clipboard, hoặc văn bản xuất hiện trên màn hình khi không thể chọn trực tiếp. Ứng dụng cần hoạt động nền qua system tray, dùng hotkey/overlay, OCR khi cần, và hiển thị bản dịch nhanh trong popup/overlay.

**Không được biến dự án thành app ghi màn hình, replay màn hình, hay công cụ annotation video.** `src/ScreenCapReplay-plan.md` là tài liệu lạc phạm vi/sai hướng đối với zTranz và không được dùng làm mục tiêu triển khai.

## Rules bắt buộc cho mọi AI-agent/phiên làm việc tiếp theo

1. Đọc file này trước khi phân tích, sửa code hoặc lập kế hoạch.
2. Luôn bám mục tiêu “dịch mọi nơi trên Windows”; trước mỗi thay đổi phải tự đối chiếu với goal, spec, plan và task.
3. Không lấy `ScreenCapReplay-plan.md` làm spec của zTranz. Nếu cần ghi chú, đánh dấu rõ tài liệu đó là out-of-scope.
4. Sau mỗi phiên debug/triển khai:
   - cập nhật file này với trạng thái mới nhất;
   - ghi bài học kinh nghiệm, lỗi đã sửa, lỗi còn lại và bước tiếp theo;
   - kiểm tra thay đổi bằng build/test phù hợp và ghi kết quả trung thực.
5. Git chính để sao lưu định kỳ mã nguồn: `https://github.com/magicxlll/zTranz.git`.
6. Sau mỗi phiên debug, kiểm tra trạng thái git và sao lưu/commit/push theo quyền truy cập và chính sách hiện hành. Không tuyên bố đã push nếu chưa xác minh thành công.
7. Trước khi kết thúc phiên, phải để lại context đủ để phiên sau tiếp tục ngay: mục tiêu, hiện trạng, file đã đổi, kiểm thử, vấn đề, hành động kế tiếp.
8. Không mở rộng phạm vi sang ghi màn hình, replay, annotation video, hoặc tính năng không phục vụ dịch mọi nơi nếu chưa có yêu cầu mới rõ ràng.

## Đường dẫn tài liệu chính

- Project memory: `PROJECT_MEMORY.md` (file này)
- Memory index: `MEMORY.md`
- Manifest/dependency: `Cargo.toml`
- Entry point: `src/main.rs`
- Điều phối luồng: `src/main_dispatch.rs`
- Clipboard/UI Automation: `src/clipboard.rs`
- Capture/OCR: `src/capture.rs`, `src/ocr.rs`
- Dịch và provider: `src/translate.rs`, `src/http.rs`
- Tray/popup/overlay: `src/tray.rs`, `src/popup.rs`, `src/overlay.rs`
- State/config: `src/state.rs`, `src/config.rs`
- Tài liệu hiện có nhưng **không thuộc goal zTranz**: `src/ScreenCapReplay-plan.md`
- Git remote dự kiến: `https://github.com/magicxlll/zTranz.git`

## Hiện trạng đã xác nhận

- Đây là Rust/Windows app, package `ztranz`, description “translate anything, anywhere (Windows)”.
- Có hotkey: Win+Shift+C để chọn vùng OCR, Win+Shift+V để dịch selection/clipboard, Win+Shift+Q để thoát.
- Có Google Translate và OpenAI-compatible LLM provider.
- OCR dùng Windows.Media.Ocr.
- Có system tray, popup và overlay chọn vùng.
- `screen_capture.py` là helper chụp/crop ảnh phục vụ OCR, không phải recorder.
- Môi trường hiện tại không có lệnh `cargo` trên PATH; build Rust chưa thể xác nhận bằng CLI trong phiên này.
- Thư mục dự án hiện chưa được nhận diện là Git repository tại đường dẫn làm việc; cần xác minh/khởi tạo hoặc clone đúng repository trước khi sao lưu.

## Trạng thái phiên gần nhất

- Đã xác định lỗi định hướng: trước đó một agent đã nhầm `ScreenCapReplay-plan.md` là kế hoạch chính và kết luận sai về app ghi màn hình.
- Đã tạo tài liệu nguồn: `PROJECT_MEMORY.md`, `MEMORY.md`, `SPEC.md`, `PLAN.md`, `TASKS.md`.
- Đã khởi tạo Git repository tại thư mục dự án (branch `main`), đặt tác giả `PHAN VIET <vietph@Zit-M4.local>` (khớp các repo cùng Workspace), thêm `.gitignore` (loại `target/`, `ztranz_rust_recovered/`), tạo baseline commit `0283440`.
- Đã thêm remote `origin = https://github.com/magicxlll/zTranz.git` (repo remote hiện trống, chưa có branch nào).
- Push tới GitHub **chưa thành công**: thiếu thông tin xác thực (`fatal: unable to get password from user`). Cần token/SSH credentials của GitHub cho repo `magicxlll/zTranz` (ghi rõ: chưa push được, chưa xác minh remote nhận mã nguồn).
- Toolchain Rust có sẵn: `cargo 1.97.0`, `rustc 1.97.0` (tại `~/.cargo/bin`, không có trên PATH mặc định — dùng `export PATH="$HOME/.cargo/bin:$PATH"`).
- Dự án là app Windows thuần (crate `windows` 0.62.2 + Win32 API: RegisterHotKey, UpdateLayeredWindow, Windows.Media.Ocr). Host hiện tại là macOS (Apple Silicon), nên build native không đại diện cho build Windows; cần cross-compile hoặc build trên host Windows.
- Đã chạy `PATH="$HOME/.cargo/bin:$PATH" cargo check`; thất bại ở dependency `windows-future 0.3.2` với lỗi thiếu `windows_core::imp::{IMarshal, marshaler}` và `windows_threading::submit` trên host macOS. Chưa được phép kết luận mã nguồn zTranz có lỗi compile Windows; cần chạy `cargo check --target <Windows target>` trên toolchain/host Windows và/hoặc rà lại feature/version của crate `windows`.
- Thư mục `target/` chiếm ~3.4 GB (artifacts cũ), đã được `.gitignore` loại khỏi git.

## Bài học kinh nghiệm

- Luôn đọc `Cargo.toml`, `src/main.rs`, `src/main_dispatch.rs` và file memory trước khi tin một plan bất kỳ.
- Tên/tài liệu “ScreenCapReplay” không phản ánh mục tiêu zTranz; phải kiểm tra package description và luồng runtime.
- Không được báo cáo kết quả build nếu công cụ build không tồn tại hoặc chưa chạy thành công.

## Next steps ưu tiên

1. Tạo/chuẩn hóa `SPEC.md`, `PLAN.md`, `TASKS.md` cho mục tiêu dịch mọi nơi.
2. Xác minh Git repository và remote GitHub; tạo commit baseline chỉ khi đã xem xét thay đổi.
3. Cài/khôi phục toolchain Rust phù hợp hoặc xác định workflow build Windows đáng tin cậy.
4. Chạy build/check và sửa lỗi compile/runtime theo từng luồng.
5. Hoàn thiện trải nghiệm: dịch selection/clipboard, OCR vùng màn hình, popup/overlay, cấu hình provider, lỗi mạng và packaging.
6. Cập nhật file này sau mỗi phiên debug.

## Trạng thái phiên build Windows thật — 2026-11-16

- Đã xác nhận peer Windows `DESKTOP-0HA1J56` online qua Tailscale tại `100.88.17.56`; `tailscale ping` khoảng 7 ms và Peer API cổng 48532 trả lời.
- SSH cổng 22 và WinRM cổng 5985/5986 không phản hồi từ Mac; chưa có quyền thực thi lệnh/build trên Desktop. Không được báo cáo đã build Windows thật.
- Đã thêm `scripts/test-windows.ps1`: kiểm tra Rust/MSVC target, fmt, check, test, build và xác nhận executable/artifact.
- Đã thêm `.github/workflows/test-windows-real.yml`: workflow thủ công cho self-hosted runner labels `self-hosted`, `Windows`, `X64`, `ztranz-windows-real`; dùng Windows PowerShell 5.1.
- Đã thêm `WINDOWS-REAL.md` với hướng dẫn chạy trực tiếp, self-hosted runner, giới hạn Tailscale và GUI smoke test.
- Kế hoạch chi tiết nằm ở `plan-build-windows.yml.md`; chưa commit/push các file mới.

### Kiểm thử phiên này

- Đã kiểm tra trạng thái git và diff file mới.
- Chưa thể chạy `scripts/test-windows.ps1` trên Mac; chưa có Windows runner online để chạy workflow.
- Cần bước tiếp theo: đăng ký Desktop làm self-hosted runner hoặc bật OpenSSH/WinRM được phép, sau đó dispatch workflow và ghi kết quả thực tế.

## SỰ KIỆN THEN CHỐT — TAILSCALE SSH KHÔNG HỖ TRỢ WINDOWS

- Người dùng thử `tailscale up --ssh` trên Desktop Windows và nhận thông báo chính thức: **"The Tailscale SSH server is not supported on windows"**.
- Do đó **Tailscale SSH bị loại khỏi danh sách phương án** kết nối từ Mac tới Desktop. Mọi tài liệu/hướng dẫn cũ nhắc Tailscale SSH cho Windows là sai.
- Còn lại đúng 2 đường để chạy build thật:
  1. **GitHub Actions self-hosted runner** (khuyến nghị, không cần mở port): cài runner trên Desktop, đặt label `self-hosted, Windows, X64, ztranz-windows-real`, push workflow rồi dispatch từ GitHub UI. Runner nên chạy trong **phiên đăng nhập tương tác** (GUI/OCR mới hoạt động), không chạy làm Windows Service.
  2. **OpenSSH Server Windows** (cho phép Mac điều khiển trực tiếp): cài bằng `Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0`, khởi động service `sshd`, bật Automatic, thêm firewall rule cho port 22 **chỉ trên mạng Tailscale**, rồi `ssh 'DESKTOP-0HA1J56\1111'@100.88.17.56`.

## BÁO CÁO TỔNG KẾT PHIÊN (để bàn cho người dùng)

- **Đã đạt:**
  - Xác định Desktop Windows `DESKTOP-0HA1J56` (Tailscale IP `100.88.17.56`) đang online và đạt được qua mạng Tailscale.
  - Loại bỏ các phương án kết nối không khả thi: SSH trực tiếp (port 22 không mở), WinRM (5985/5986 không mở), và **Tailscale SSH (không hỗ trợ trên Windows — thông báo chính thức từ Tailscale)**.
  - Tạo bộ công cụ build/test cho Windows thật:
    - `scripts/test-windows.ps1` — script PowerShell (tương thích Windows PowerShell 5.1): kiểm tra rustc/cargo, kiểm tra target MSVC đã cài (`rustup target list --installed`), chạy `cargo fmt --all -- --check`, `cargo check --target x86_64-pc-windows-msvc`, `cargo test --target x86_64-pc-windows-msvc`, build release/debug theo tham số, xác nhận executable tồn tại, copy thành artifact và in thông báo PASS kèm kích thước.
    - `.github/workflows/test-windows-real.yml` — workflow dispatch thủ công: checkout mã nguồn, chạy `scripts/test-windows.ps1` theo profile (release/debug), upload executable và log làm artifact. Runner label: `self-hosted, Windows, X64, ztranz-windows-real`.
    - `WINDOWS-REAL.md` — hướng dẫn: chạy trực tiếp trên Desktop, cách đăng ký self-hosted runner (chú ý chạy runner trong phiên tương tác, không dùng service), các giới hạn Tailscale đã xác minh, và checklist smoke test GUI/hotkey/OCR sau khi build thành công.
    - `plan-build-windows.yml.md` — kế hoạch chi tiết từng bước.
  - Cập nhật section "Trạng thái phiên build Windows thật — 2026-11-16" trong file này.

- **Chưa đạt / không được kết luận:**
  - Chưa chạy được bất kỳ lệnh build/compile/test nào trên máy Windows thật — không có quyền thực thi lệnh từ Mac (SSH/WinRM/Tailscale SSH đều không khả dụng). **Cấm báo cáo "đã build thành công trên Windows" trước khi workflow/runner chạy xong trên Desktop.**
  - Chưa commit/push các file mới (`scripts/`, `.github/workflows/test-windows-real.yml`, `WINDOWS-REAL.md`, `plan-build-windows.yml.md`, cập nhật `PROJECT_MEMORY.md`).
  - Chưa xác nhận Desktop đã cài Rust/MSVC cho target `x86_64-pc-windows-msvc`.

- **Rủi ro / cạm bẫy cho phiên sau:**
  - Tailscale SSH: không hỗ trợ Windows (đọc lại sự kiện trên).
  - Self-hosted runner phải chạy trong phiên đăng nhập tương tác; chạy làm service sẽ làm GUI/OCR không hoạt động.
  - Mac đang là macOS Apple Silicon: `cargo check` local đã thất bại ở dependency `windows-future 0.3.2` (thiếu `windows_core::imp::{IMarshal, marshaler}` và `windows_threading::submit`) — không đại diện cho build Windows; chỉ kết luận sau khi chạy trên Desktop.
  - Git: baseline đã có (`0283440`), remote `origin` là `https://github.com/magicxlll/zTranz.git` nhưng **chưa push được** (thiếu credentials). Cần token/SSH key.

## SỰ KIỆN THEN CHỐT — BUILD WINDOWS THẬT THÀNH CÔNG — 2026-11-17

Phiên này **chạy build thật trên máy Windows thật** (không qua runner/SSH) và thành công. Kết quả ghi lại trung thực:

### Trạng thái toolchain (đã xác nhận chạy được)

- Rust **1.100.0** / cargo 1.100.0 đã cài (`~/.cargo/bin`), host target mặc định `x86_64-pc-windows-gnu`.
- Target **`x86_64-pc-windows-msvc`** đã thêm và hoạt động. MSVC Build Tools 18 (`C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools`) có sẵn; link.exe x64 OK.
- Đã thêm component `rustfmt` (ban đầu thiếu → `cargo fmt` lỗi).
- Helper build: `scripts/run-msvc.cmd` (call `vcvarsall.bat x64`, thêm `~/.cargo/bin` vào PATH, rồi chạy `cargo %*`). Gọi dạng `cmd //c "scripts\run-msvc.cmd build --release --target x86_64-pc-windows-msvc"`.

### Nguyên nhân gốc của 82 lỗi compile (và cách sửa)

- Mã nguồn viết theo **API layout của `windows` crate đời cũ**, trong khi `Cargo.toml`/`Cargo.lock` pin **`windows 0.62.2`**. Đây là port hệ thống, không phải lỗi logic.
- Giữ nguyên pin `0.62.2` (deterministic, có sẵn registry cache) và **port các call site** sang ký hiệu 0.62.2. Các thay đổi chính:
  - `Cargo.toml`: thêm feature WinRT `Graphics_Imaging` (cho `Graphics::Imaging`/`Media_Ocr` pipeline), thêm dependency `serde` (không default-features).
  - Hàm đổi module/kiểu trong 0.62.2: `GetClientRect`/`GetCursorPos` → `WindowsAndMessaging`; `RedrawWindow`/`REDRAW_WINDOW_FLAGS` → `Graphics::Gdi`; `INFINITE` → `System::Threading`; `CreateCompatibleDC`/`GetDC` trả `HDC` trực tiếp (không `Result`); `SelectObject`/`DeleteObject` nhận `HGDIOBJ` (dùng `.into()`); `RegisterHotKey` nhận `HOT_KEY_MODIFIERS` (typed const); `PostMessageW`/`RedrawWindow` nhận `Option<HWND>`; `MsgWaitForMultipleObjectsEx` chỉ 4 tham số (loại `INFINITE`).
  - `IOutputStream::WriteBytes` **bị gỡ** trong 0.62.2 → viết lại `ocr.rs` đọc PNG trực tiếp qua `StorageFile::GetFileFromPathAsync` + `OpenAsync` (tránh copy byte vào stream memory).
  - `BSTR` (UIAutomation `GetText`) không có `to_string_lossy` → dùng `String::from_utf16_lossy(t.as_ref())`.
  - Sửa static không const (`state.rs` `Mutex<String>` khởi tạo rỗng), `static mut` dùng `&raw const` (loại warning `static_mut_refs`).
  - **Lỗi hotkey conflict không crash app nữa**: `register_hotkeys` từng `return Err` khi 1 hotkey đã bị app khác giữ (vd SnippingTool giữ Win+Shift+C) → giờ chỉ log warning và bỏ qua phím đó; tray menu vẫn dùng được. `util::log` cũng tự tạo thư mục log.

### Kết quả build/test thật (đã chạy, không suy đoán)

- `cargo check --target x86_64-pc-windows-msvc`: **0 lỗi, 0 warning** (ban đầu 82 lỗi).
- `cargo test --target x86_64-pc-windows-msvc`: ok (0 test — crate chưa có unit test).
- `scripts/test-windows.ps1 -Profile release`: **PASS** — artifact `artifacts/zTranz-windows-x64-release.exe` (~328 KB).
- Chạy trực tiếp `zTranz.exe`: app khởi động, tạo tray, log hotkey conflict, **chạy liên tục không crash**.
- **OCR pipeline xác nhận chạy thật**: tạo probe crate riêng (`ocr_probe/`, ngoài dự án) render chữ bằng GDI+ rồi chạy đúng `ocr_pipeline` của `src/ocr.rs` → nhận đúng "The quick brown fox 12345". Chứng tỏ WinRT `join()` blocking path hoạt động trên worker thread trong build release.

### File đã đổi trong phiên này

- Source: `src/{capture,clipboard,config,http,main,main_dispatch,ocr,overlay,popup,state,tray,util}.rs`
- `Cargo.toml`, `Cargo.lock`, `.gitignore` (bỏ `/artifacts`), `scripts/run-msvc.cmd`, `scripts/msvc-env.cmd`.
- Probe riêng (không commit vào dự án): `ocr_probe/` — chỉ để xác minh OCR.

### Hạn chế / chưa làm (ghi thẳng, không phóng đại)

- **Chưa test đầy đủ GUI tương tác** (drag overlay chọn vùng, click tray, popup) trong phiên này — mới chạy được đến mức "app khởi động ổn định + OCR pipeline OK". Cần người dùng test GUI tay.
- **Chưa xác nhận mạng/provider**: chưa gọi thật Google Translate / LLM endpoint (cần key + mạng). Lỗi mạng chưa smoke test.
- Chưa có unit test; `cargo test` chỉ build không chạy test nào.
- `config.save()` hiện chỉ `#[allow(dead_code)]` — chưa có UI settings để lưu config.

## HƯỚNG DẪN CHO PHIÊN SAU (PROMPT HANDOFF)

> **Prompt gợi ý để AI phiên sau copy/dùng (điều chỉnh tên người dùng nếu cần):**
>
> "Xin chào, tiếp tục dự án zTranz (ứng dụng dịch mọi nơi trên Windows). Trước tiên đọc `PROJECT_MEMORY.md` trong thư mục dự án `zTranz/` — đặc biệt các mục 'MỤC TIÊU CHÍNH', 'Rules bắt buộc', 'SỰ KIỆN THEN CHỐT — TAILSCALE SSH KHÔNG HỖ TRỢ WINDOWS', 'BÁO CÁO TỔNG KẾT PHIÊN' và mục kế.
>
> **Mục tiêu phiên:** Có được kết quả build Windows thật (check + test + build release) của zTranz trên Desktop `DESKTOP-0HA1J56` (Tailscale IP `100.88.17.56`), và commit/push toàn bộ thay đổi chưa commit.
>
> **Trạng thái hiện tại:**
> - `scripts/test-windows.ps1`, `.github/workflows/test-windows-real.yml`, `WINDOWS-REAL.md`, `plan-build-windows.yml.md` đã được tạo trong phiên trước nhưng **chưa commit**.
> - Desktop đã online qua Tailscale, nhưng SSH (22), WinRM (5985/5986) và Tailscale SSH đều **không khả dụng** (Tailscale SSH: không hỗ trợ Windows — đã xác nhận chính thức).
> - Chưa có kết quả build thật trên Windows; mọi kết luận build phải dựa trên kết quả thực tế từ Desktop, không suy đoán.
>
> **Hành động đề xuất (theo thứ tự):**
> 1. Commit các file mới trong phiên trước (script, workflow, tài liệu) và ghi rõ nội dung trong message commit.
> 2. Xác nhận với người dùng phương án kết nối Desktop: (a) đăng ký Desktop làm GitHub Actions self-hosted runner (khuyến nghị — không cần mở port, chạy trong phiên đăng nhập tương tác), hoặc (b) bật OpenSSH Server trên Desktop qua `Add-WindowsCapability` để Mac có thể ssh trực tiếp. Chọn (a) nếu Desktop đã có GitHub runner khả dụng; nếu chưa, hướng dẫn người dùng cài runner và đặt label `self-hosted, Windows, X64, ztranz-windows-real`.
> 3. Push mã nguồn (cần credentials GitHub) và dispatch workflow `Test on real Windows host` với profile `release` (và/hoặc `debug`).
> 4. Theo dõi kết quả runner: nếu lỗi thiếu target MSVC, chạy `rustup target add x86_64-pc-windows-msvc` trên Desktop; nếu lỗi compile, sửa theo từng module (`translate.rs`, `ocr.rs`, `popup.rs`, `overlay.rs`, `tray.rs`, `clipboard.rs`) và commit lại.
> 5. Sau khi build thành công, chạy smoke test GUI/hotkey/OCR theo checklist trong `WINDOWS-REAL.md` và ghi kết quả thật (thành công/lỗi) vào `PROJECT_MEMORY.md`.
> 6. Cập nhật `PROJECT_MEMORY.md` đầy đủ trước khi kết thúc phiên: trạng thái mới, file đã đổi, kết quả build/test thật, và handoff cho phiên sau nếu chưa xong."
>
> **Cấm đoán (răn cũ vẫn giữ):** Không dùng Tailscale SSH cho Windows. Không báo cáo "đã build" nếu chưa có kết quả thực tế từ Desktop. Không mở rộng phạm vi sang tính năng không phục vụ "dịch mọi nơi".
