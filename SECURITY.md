# Security Policy

## Báo cáo lỗ hổng bảo mật

**Không mở Issue công khai** cho lỗ hổng bảo mật. Thay vào đó:

1. Gửi email private đến maintainer (xem GitHub profile)
2. Hoặc dùng GitHub Private Security Advisory

Chúng tôi sẽ phản hồi trong vòng **7 ngày làm việc** và phát hành bản vá trong vòng **30 ngày** tùy mức độ nghiêm trọng.

## Các phiên bản được hỗ trợ

| Phiên bản | Hỗ trợ bảo mật |
|-----------|---------------|
| 0.1.x (current) | ✅ |
| < 0.1 | ❌ |

## Security Design

- **S2**: Không log text/nội dung người dùng gõ
- **S3**: Passthrough ô mật khẩu (`WS_EX_NOPARENTNOTIFY` + `IsPasswordField`)
- **S5**: Chạy per-user, không đòi admin (`asInvoker` manifest)
- **S9**: Không xóa config user khi uninstall
