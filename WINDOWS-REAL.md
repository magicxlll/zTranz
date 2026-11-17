# Build/test trên Windows thật

Repo có hai lane Windows:

- `build-windows.yml`: `windows-latest`, chạy tự động khi push lên `main`.
- `test-windows-real.yml`: máy Windows thật, chỉ chạy thủ công qua `workflow_dispatch`.

## Cách chạy trực tiếp trên Desktop

Mở PowerShell trong thư mục repo và chạy:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\test-windows.ps1 -Profile release
```

Script kiểm tra Rust/MSVC target, format, `cargo check`, `cargo test`, release build và sao chép executable vào `artifacts\`.

## Self-hosted runner

Cách an toàn và tái lập nhất là đăng ký Desktop làm GitHub Actions self-hosted runner riêng với các label:

- `self-hosted`
- `Windows`
- `X64`
- `ztranz-windows-real`

Không commit runner token, private key hoặc credential vào repo. Sau khi runner online, vào Actions → **Test on real Windows host** → **Run workflow**.

## Kết nối qua Tailscale

Tailscale chỉ cung cấp đường mạng; máy Windows vẫn phải bật một phương thức quản trị (self-hosted runner, OpenSSH Server hoặc WinRM). Hiện tại node `100.88.17.56` online và ping được, nhưng SSH `:22` và WinRM `:5985/:5986` không phản hồi, nên chưa thể điều khiển máy từ Mac. Không coi `tailscale ping` là bằng chứng đã build/test.

## GUI smoke test

`cargo test` và runner service kiểm tra compile/test, không chứng minh hotkey, tray, popup, OCR hoặc clipboard trong desktop session. Các hạng mục đó cần chạy executable trong phiên Windows đang đăng nhập và kiểm tra thủ công.
