# Tài liệu cấu hình (VI)

Schema cấu hình chuẩn:

- [`../../../src/config/schema.rs`](../../../src/config/schema.rs)

Mã tải/gộp cấu hình:

- [`../../../src/config/mod.rs`](../../../src/config/mod.rs)

Các khóa plugin mới:

- `[plugins].marketplace_enabled` (mặc định `false`, bắt buộc để cài từ HTTP(S))
- `[plugins].allowed_permissions` (allowlist quyền được chấp nhận khi cài plugin)
