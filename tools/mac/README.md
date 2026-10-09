# tools/mac — harness macOS (MAC-060…063, P2-5 §4/§5/§6)

Công cụ chạy trên **máy Mac thật** (hoặc runner macOS có GUI). Mọi script đều
"fail-open theo lý do": thiếu GUI/quyền → in lý do rồi `exit 3` (SKIP) — **không**
được im lặng pass (P2-5 §4 RM5). Gate PR vẫn là corpus headless
(`textvn-cli replay corpus/… --adapter mac`), chạy được trên mọi OS.

| File | Task | Chạy |
|---|---|---|
| `soak.sh` | MAC-063 | `tools/mac/soak.sh -Hours 24` (weekly) · `-Hours 2 -Interval 60` (nightly) · `-Seconds 60` (thử nhanh) |
| `smoke-imk.sh` | P2-5 §4.1 | `tools/mac/smoke-imk.sh` — TextEdit + 6 case + toggle EN/VN |
| `mem-check.sh` | P2-5 §5 | `tools/mac/mem-check.sh` — IMK < 60 MB · TextVN.app < 80 MB · CPU idle |
| `targets/*.json` | MAC-061 | input cho `ax-driver` (MAC-060) |
| `ax-driver/` | MAC-060 | **chưa code** — xem §2 |

## 1. Targets JSON — cùng schema với `tools/appcomptest/targets/` (WIN-061)

Giữ **nguyên tên khoá** của bản Windows để 1 loader dùng chung được (nguyên tắc G15
"reuse tối đa"); nghĩa từng khoá map sang API Accessibility (AX) của macOS:

| Khoá | Windows (UIA) | macOS (AX) |
|---|---|---|
| `match.any[].exe` | tên exe | **`bundle_id`** (vd `com.apple.TextEdit`) |
| `launch.kind` | `exe`/`com`/`shell` | `app` (mở bằng `open -b <bundle_id>`) |
| `launch.paths` | đường dẫn exe | đường dẫn `.app` (fallback khi `open -b` fail) |
| `launch.ready_class` | class cửa sổ | **`ready_role`** = AX role xuất hiện khi app sẵn sàng |
| `fields[].field_role` | whitelist 11 role (P0-3 §2.1) | **giống hệt** |
| `fields[].preset` | preset `win.*` | preset `mac.*` (P2-3 §3, 20 preset trong `data/appdb.default.json`) |
| `locators[].control_type` | ControlType UIA | **AX role** (`AXTextField`, `AXTextArea`, `AXWebArea`…) |
| `locators[].automation_id` | AutomationId | `AXIdentifier` (trùng nếu rỗng) |
| `locators[].name` | Name (exact) | `AXTitle`, fallback `AXDescription` |
| `locators[].class_name` | ClassName | `AXSubrole` (vd `AXStandardWindow`) |
| `locators[].name_regex` | regex client-side | **giống hệt** |

Quy tắc (như bản Windows): mỗi locator = AND các khoá; thứ tự mảng = độ ưu tiên;
mỗi field ≥ 2 locator (fallback); readiness poll **theo từng field** ≤ 10s.

Trạng thái verify ghi trong `notes` của từng file — **không tick trước** khi chưa
chạy trên máy thật (app nào chưa cài thì ghi rõ "chưa verify", đúng cách bản Windows
đã ghi cho slack/discord/jetbrains).

**Gate (chạy được mọi OS):** `cargo run -q -p xtask -- check-mac-targets` — bắt
`app_id` ≠ tên file, `match.any[].exe` (dấu vết schema Windows), `field_role` ngoài
whitelist P0-3 §2.1, `preset` không phải `id` **có thật** trong `data/appdb.default.json`,
`control_type` thiếu tiền tố `AX`, field < 2 locator, và thiếu 1 trong 12 app CI.
Gate này nằm trong job `xtask check-tables` của `ci-shared.yml`; 9 unit test parser/
validator chạy cùng `cargo test -p xtask`.

## 2. `ax-driver` (MAC-060) — thiết kế, chưa code

Chưa viết Swift executable vì **không thể build/verify Swift ngoài máy Mac**; viết
mù sẽ vi phạm quy tắc "code xong ≠ verified" của `IMPLEMENTATION-STATUS.md`. Thiết
kế đã chốt (P2-5 §4), chỉ còn cài đặt:

```text
ax-driver --suite ci|full --only <app_id> --report <out.json>
1. Đọc tools/mac/targets/<app_id>.json (schema §1)
2. Mở app: `open -b <bundle_id>` (fallback `open <paths[0]>`) → đợi `ready_role` ≤ 15s
3. Focus field: AXUIElementSetAttributeValue(kAXFocusedUIElementAttribute)
   (fallback: CGEvent click toạ độ tâm element)
4. Gửi input: CGEvent keyDown/keyUp (đi ĐÚNG đường phím — KHÔNG set AX value);
   case paste: NSPasteboard + ⌘V
5. Assert: AXValue / kAXSelectedTextAttribute → so `:expect` (format P0-4)
6. Report JSON: {id, app, case, status, ms, t_glyph_ms, notes} — cùng format
   appcomptest Windows để hợp nhất dashboard
7. Cleanup: ⌘Q app, khôi phục clipboard
```

- Timeout 15s/case → `fail`, không treo suite (P2-5 §4).
- `t_glyph_ms` = từ lúc post key đến khi AX text đổi (poll 1ms) → p50/p95/p99.
- **RM5:** lần chạy đầu phải thử trên runner `macos-latest`; nếu TCC chặn AX →
  report ghi `status: "skip", notes: "AX harness = local nightly (TCC)"` và job nightly
  (sẽ tạo cùng `ax-driver` — hiện **chưa có** workflow nightly) vẫn xanh (gate PR không phụ
  thuộc layer này).

## 3. Việc còn lại trên máy thật (cập nhật 2026-10-09)

macOS đã hoàn tất phần sản phẩm (chủ repo xác nhận trên máy thật 2026-10-08) — xem
`docs/30-macos/IMPLEMENTATION-STATUS.md` (mục "Còn mở"). Phần còn lại của thư mục này:
viết `ax-driver` (MAC-060); chạy `soak.sh`/`mem-check.sh` rồi ghi `perf/baseline-mac.json`
bằng `cargo run -p textvn-bench --release -- write perf/baseline-mac.json` (MAC-062/063);
`smoke-imk.sh` → ghi `docs/release/rc-checklist-mac.md` (file chưa tạo).
