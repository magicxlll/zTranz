# Kế hoạch: build/test Windows thật qua Desktop Tailscale

## Mục tiêu

Bổ sung một lane xác minh Windows thật cho zTranz, bên cạnh workflow GitHub Actions hiện tại, để có thể chạy `cargo check`, test và build release trên `DESKTOP-0HA1J56` qua Tailscale. Không đưa credential hoặc địa chỉ máy nội bộ vào artifact/repository.

## Phạm vi và phát hiện hiện tại

- Tailscale tới `100.88.17.56` đang hoạt động; `tailscale ping` đạt khoảng 7 ms.
- Peer API Tailscale trên cổng 48532 trả lời, xác nhận node Windows online.
- TCP 22, 5985 và 5986 hiện timeout; chưa thể thực thi lệnh/build từ Mac.
- Workflow `.github/workflows/build-windows.yml` hiện chỉ build trên `windows-latest` của GitHub Actions.
- `Cargo.toml` hiện dùng `windows = 0.62.2`; không tự ý đổi dependency version.

## Thiết kế đề xuất

1. **Tạo script kiểm tra Windows tái lập được**
   - Thêm script PowerShell trong repo để kiểm tra Rust toolchain/target, chạy fmt check, check, test (nếu có), và release build.
   - Script dùng strict error handling, ghi rõ command/version/target và trả exit code thất bại.
   - Giữ script độc lập với transport (chạy trực tiếp trên Desktop, qua SSH/WinRM, hoặc từ self-hosted runner).

2. **Mở rộng CI theo hướng không phá lane hiện tại**
   - Giữ `windows-latest` làm lane cloud reproducible.
   - Thêm workflow/manual job tùy chọn cho self-hosted Windows runner với label riêng, để Desktop có thể đăng ký an toàn vào GitHub Actions nếu user muốn.
   - Job gọi cùng script PowerShell, upload log và executable khi build thành công.
   - Không tự động chạy trên push cho máy cá nhân; dùng `workflow_dispatch` hoặc label rõ ràng để tránh chạy ngoài ý muốn.

3. **Thêm tài liệu vận hành cho máy thật**
   - Ghi các bước cài Rust stable/MSVC target, Visual Studio Build Tools, đăng ký self-hosted runner hoặc bật transport được cho phép.
   - Không ghi token runner, mật khẩu, private key, hay thông tin nhạy cảm vào repo.
   - Nêu rõ cách chọn giữa self-hosted runner (khuyến nghị cho GitHub Actions) và Tailscale SSH/WinRM (chỉ khi dịch vụ đã được bật).

4. **Kết nối và xác minh trên Desktop**
   - Sau khi user bật một phương thức được phép (ưu tiên self-hosted runner; thay thế là OpenSSH Server hoặc WinRM qua Tailscale), kiểm tra kết nối bằng lệnh vô hại.
   - Chạy script trên Windows thật, lưu lại phiên bản OS/Rust/compiler, target, kết quả fmt/check/test/build và đường dẫn artifact.
   - Nếu app có GUI/hotkey/OCR cần desktop session, bổ sung smoke-test thủ công có giới hạn; không giả định CI service session có thể kiểm thử UI.

5. **Cập nhật trạng thái dự án**
   - Ghi kết quả thực tế và giới hạn còn lại vào `PROJECT_MEMORY.md`/tài liệu phù hợp.
   - Chỉ commit thay đổi source/workflow/docs sau khi kiểm tra diff; không commit credential hoặc log chứa secret.

## Xác minh sau triển khai

- Mac: `cargo fmt --all -- --check` và các kiểm tra không phụ thuộc Windows vẫn pass.
- GitHub Actions: lane cloud hiện tại pass; lane self-hosted chỉ chạy khi runner online.
- Windows thật: script trả thành công cho fmt/check/test/release build; artifact tồn tại và có kích thước hợp lý.
- Nếu transport chưa bật: báo chính xác blocker và không giả vờ đã build/test trên Desktop.

## Quyết định cần xác nhận trước khi thực thi

Phần repo có thể triển khai theo thiết kế trên. Để thật sự chạy trên Desktop, cần user bật hoặc cung cấp một trong hai cách truy cập:

- **Khuyến nghị:** đăng ký Desktop làm GitHub Actions self-hosted runner riêng, label `ztranz-windows-real`.
- **Thay thế:** bật OpenSSH Server trên Desktop và cho phép Tailscale ACL/firewall; WinRM chỉ dùng nếu môi trường đã quản lý sẵn.
