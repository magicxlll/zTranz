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
- Chưa thay đổi mã nguồn Rust trong phiên xác định nhầm lẫn.
- Cần làm tiếp: xác lập spec/plan/task chính cho zTranz, kiểm tra repo/git remote, làm cho build/test có thể chạy, rồi hoàn thiện các luồng dịch cốt lõi.

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
