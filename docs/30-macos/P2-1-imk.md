# P2-1 — IMK ADAPTER (macOS) — Solution chi tiết

> WS1 · `adapters/macos-imk/` (**Swift package + bundle `TextVN-IM.app`** — theo `PLAN §4.1`: macOS IMK = Swift;
> không nằm Rust workspace — `P0-1 §1`). ADR-006: IMK primary.
> Hợp đồng core: `../10-shared/P0-2-engine-ffi-contract.md` (đọc lại trước khi code).
> Engine link static qua `libtextvn_ffi.a` (xác minh ở spike `MAC-003`, xem RM1).

## 1. Quyết định & capability

| Quyết định | Nội dung | Vì sao |
|---|---|---|
| IMK primary, không tap mặc định | Gõ đi qua `IMKInputController.handle` — **không cần Accessibility permission** | PLAN §5.1; giảm rủi ro RM8 |
| Swift + Swift (SwiftUI cho settings) | Theo `PLAN §4.1` (chọn thay Obj-C) | Native, hợp lệ App Review |
| Preedit = IMK marked text | `setMarkedText`/`insertText` | Chuẩn macOS, app render đúng |
| Caps của IMK v1 | `IME_CAP_PREEDIT \| IME_CAP_FIELD_DETECT \| IME_CAP_SELECTION` (+ `IME_CAP_INJECT_VK` **chỉ khi spike MAC-004 pass**) | Chưa biết cơ chế xóa N ký tự → không tự nhận capability trước khi chứng minh |
| 1 instance engine / process IMK | IMK server = 1 process cho session; mọi IMKInputController dùng chung 1 `ime_instance` trên main thread | P0-2 §3 (main thread = thread duy nhất của IMK callback) |

## 2. Cấu trúc bundle & repo

```
adapters/macos-imk/
├── Package.swift                     # SwiftPM: targets IMKApp (executable), CoreBridge (FFI)
├── Sources/
│   ├── IMKApp/
│   │   ├── main.swift                # khởi tạo IMKServer(name:bundleIdentifier:)
│   │   ├── TextVNInputController.swift   # IMKInputController: handle/activateServer/…
│   │   ├── KeyTranslator.swift       # UCKeyTranslate + modifierFlags → ime_key_v1
│   │   ├── ApplyReplace.swift        # §6 — nơi duy nhất sửa text
│   │   ├── Marked.swift              # §7 — quản lý marked text/commit ngắn (B11/B13)
│   │   ├── FieldDetect.swift         # AX query (opt-in) → FieldContext
│   │   ├── IpcClient.swift           # unix socket, schema ipc.v1.md
│   │   └── Diagnostics.swift         # log (không text), heartbeat
│   └── CoreBridge/                   # Swift wrapper quanh textvn_ffi.h (modulemap)
├── Resources/Info.plist              # §8 — keys đăng ký IMK
├── build-rust.sh                     # cargo build --release --target aarch64/x86_64-apple-darwin
└── TextVN-IM.entitlements           # hardened runtime (khi ký)
```

- `build-rust.sh` build cả 2 arch → `lipo` → `libtextvn_ffi.a` (universal). CI chạy script này trước khi `swift build`.
- Swift file **không** giữ state cục bộ ngoài `Engine` singleton; mọi trạng thái ngữ nghĩa nằm trong engine (P0-2 §3).

## 3. Vòng đời IMK

```text
launch (system spawn khi user chọn input source)
  main.swift: IMKServer(name: "TextVN-IM_Connection", bundleIdentifier: "vn.textvn.im")
IMKInputController.init(server:…)
activateServer(sender)                  # client focus →
  1. Engine.shared.ensureInstance(): đọc config (Application Support/TextVN/config.json)
  2. ipc connect unix socket → GetSnapshot (thất bại → offline, không block — P0-3 §4)
  3. AppContext.current = FieldDetect.resolve(focused pid) → ime_set_context
deactivateServer(sender)                # mất focus →
  1. nếu marked/đang dở → COMMIT ngay (B13: không mất chữ, "commit-before-hide")
  2. ime_reset()
didTerminate / cancelComposition        # client đóng → commit + reset
App terminate                           # đóng socket, ime_instance_free
```

- `setValue(_:forTag:client:)` bắt tag đổi input mode (nếu dùng `ComponentInputModeDict`) → đồng bộ `method`.
- **Không** đăng ký observer nào ngoài IMK (tránh retain cycle → leak giữ process sống mãi).

## 4. State machine (giống `P1-1 §4`, marked text thay composition)

```
       phím sinh text (REPLACE/COMMIT)
Idle ───────────────────────────────► Marked(pretedit)
  ▲                                     │
  │ Space/Enter/động từ (WORD_END)      │ phím khác với preedit
  └──────── COMMIT (insertText) ◄───────┘
  phím hệ thống/Cmd+combo → không đổi state (PASS)
  focus lost/cancel → COMMIT + Idle (không mất chữ — B13)
```

- `hasMarkedText == false` nhưng engine tin là Marked (app tự clear) → **self-heal**: `ime_reset()` + về Idle
  (P1-1 §4 — cùng triết lý, không đoán text của app).

## 5. Key flow

```text
override func recognizedEvents(_:) -> [.keyDown, .flagsChanged]
override func handle(_ event: NSEvent!, client sender: Any!) -> Bool
  0. event.cgEvent?.userData == kTextVNInjectedMarker → return false   (chống loop — P2-2 §6)
  1. Cmd/Option+phím tắt hệ thống, Cmd+Tab… → return false (B6, không nuốt hotkey)
  2. flagsChanged → chỉ ghi modifier state (để Ctrl+Shift+Space toggle) → return false
  3. ch = KeyTranslator.translate(event) (UCKeyTranslate theo layout hiện tại — không hardcode ABC)
  4. ctx = Engine.context (secure/enabled/app_id/field_role/caps) từ FieldDetect cache
  5. r = ime_key(inst, {vk: event.keyCode, ch, mods, key_down:1}, &out)
  6. match out.action:
       PASS     → return false
       REPLACE  → ApplyReplace.apply(r, client) → return true   (§6)
       COMMIT   → client.insertText(utf32→String(out.insert), replacementRange: notFound) → true
       RESTORE  → ApplyReplace.apply (delete_count + insert) → true
  7. mọi lỗi/swift error → return false (fail-open — S4)
```

- `ime_result_v1.preedit` ≠ "" → `client.setMarkedText(preedit, selectionRange: tại cuối, replacementRange: NSNotFound)`.
- **Không bao giờ** block main thread (IMK callback = gõ người dùng trực tiếp); budget `ime_key` <0.5ms (P0-3 §3.3).

## 6. `apply_replace` — cách sửa text trên macOS (P0-2 §4/§7 trỏ tới đây)

### 6.1 `Preedit` (mặc định — strategy `Preedit`)
```text
ReplaceEditPlan (chỉ marked):
  setMarkedText(out.preedit, selectionRange: {len,0}, replacementRange: NSNotFound)
  khi WORD_END/Space/Enter → insertText(preedit đã xong + " ") … (xem §7)
```

### 6.2 `SelectionReplace` (bug B1 — thanh địa chỉ/Spotlight/Excel)
```text
  sel = client.selectedRange()                    # IMKTextInput
  if sel.length > 0:
      client.insertText(insertString, replacementRange: sel)   # app thay đúng selection
  else:
      fallback §6.3 với delete_count (preset đã cảnh báo: app không giữ selection)
  # KHÔNG synthetic Backspace → autocomplete không bị kích hoạt lại (cùng lý do với P1-1 §6.2)
```

### 6.3 `BackspaceType` (cơ chế xóa — SPIKE `MAC-004` quyết định thứ tự ưu tiên)
```text
thứ tự thử (ghi kết quả vào docs/specs/macos-delete-spike.md):
 (a) nếu client responds(to: #selector(deleteBackward(_:))) → gọi N lần           [không cần quyền]
 (b) nếu IMKTextInput cho phép performKeyAction/doCommand(by: #selector(deleteBackward))
 (c) fallback: synthetic CGEvent BackSpace (kèm CGEventSourceSetUserData = marker,
     handle() ở §5.0 loại phím có marker → chống loop) — CẦN check lại latency + RM2
xong phần xóa → §6.4 chèn text
```
- Nếu cả 3 fail → **không khai** `IME_CAP_INJECT_VK`; preset mac dùng `SelectionReplace`/`Preedit`
  (đa số app mac nhận selection/marked — verify bằng corpus mac trước khi chốt).

### 6.4 Chèn text (dùng chung mọi strategy)
```text
insertText(utf32_to_String(out.insert), replacementRange: NSNotFound)
  - NSNotFound → thay tại caret (hoặc thay selection hiện có — hành vi chuẩn NSTextInput)
  - tính delete_count bằng ký tự UTF-16/UTF-8 đồng nhất (String.count của Swift = grapheme;
    dùng utf16.count khi so với NSRange — unit test đếm surrogate)
```

## 7. Marked text ngắn & B11/B13 (không giữ marked lâu)

| Sự kiện | Hành động |
|---|---|
| Engine trả preedit có dấu cách/word boundary | COMMIT ngay `insertText` (không để marked sống qua Space) |
| Gõ giữa từ với preedit dài | Giới hạn marked ≤ 8 grapheme; vượt → commit phần đầu |
| Phím Backspace sửa preedit | gửi vào engine (không cho app sửa marked trực tiếp) |
| Click/focus sang ô khác | `deactivateServer` → COMMIT (B13) |
| App từ chối marked (terminal…) | Phát hiện `setMarkedText` không có effect (kiểm qua `markedRange`) → fallback strategy `ForwardAsCommit` cho app này (preset override) |

Corpus: `bug_B11_marked_commit`, `bug_B13_focus_loss` (`P2-5 §2`).

## 8. Đăng ký input source (Info.plist + cài đặt)

```xml
<!-- Resources/Info.plist — keys chính (xác minh lại bằng spike MAC-002 từ template Xcode Input Method) -->
<key>InputMethodConnectionName</key>   <string>TextVN-IM_Connection</string>
<key>InputMethodServerControllerClass</key> <string>TextVNInputController</string>
<key>TISInputSourceID</key>            <string>vn.textvn.inputmethod.TextVNIM</string>
<key>TISIntendedLanguage</key>         <string>vi</string>
<key>tsInputMethodIconFileKey</key>    <string>vieste.icns</string>
<key>ComponentInputModeDict</key>      <!-- nếu cần list mode: tsInputModeListKey, default state -->
```

```text
Cài (pkg, per-user):  copy TextVN-IM.app → ~/Library/Input Methods/
Đăng ký hiển thị menu: spike MAC-007 chốt 1 trong:
   (a) TIS API đăng ký lại (nếu còn API hợp lệ)
   (b) pkill TextInputMenuAgent (Settings > Keyboard tự reload)
   (c) yêu cầu logout/login lần đầu
Gỡ: xóa bundle + (nếu có API) unregister — 0 residue (P2-5 §6)
```
- **Không bao giờ** ghi `/Library/Input Methods/` (system) ở chế độ per-user (PLAN §3.7).
- Sau khi cài: app hiện hướng dẫn bật **System Settings → Keyboard → Input Sources → Add TextVN**
  (macOS không cho add tự động — ghi vào `P2-4 §4`).

## 9. Spike checklist (tasks `MAC-002/003/004/005/007` — tuần 1–2, chặn mọi task khác)

| # | Câu hỏi | Kỳ vọng | Task |
|---|---|---|---|
| S1 | Tạo project **Input Method** (Xcode template) chạy được, TextEdit hiện "được" với marked text | ✅ demo | MAC-002 |
| S2 | Đúng Info.plist keys (so với template thật) → list hiện trong Input menu | ✅ | MAC-002 |
| S3 | **SwiftPM link `libtextvn_ffi.a`** (modulemap + 2 arch lipo) → gọi `ime_key` từ Swift, verify size bằng `textvn sizes` | ✅ | MAC-003 |
| S4 | `handle()` nhận keyDown, `UCKeyTranslate` ra đúng char theo layout ABC/VNI | ✅ | MAC-002 |
| S5 | **Cơ chế xóa N ký tự**: selector nào dùng được? synthetic CGEvent có loop không (marker)? | chốt (a)/(b)/(c) | MAC-004 |
| S6 | `client.selectedRange()` trong Safari address bar / Spotlight có trả selection? | ✅/ghi | MAC-004 |
| S7 | AX: IMK process query AX focused element → có bị TCC chặn? prompt thế nào? | ghi rõ | MAC-005 |
| S8 | Secure input (mật khẩu): IMK có bị pause đúng? `AXSecureTextField` detect được không? | ghi | MAC-005 |
| S9 | Đăng ký input source sau khi copy vào `~/Library/Input Methods/`: cách nào làm menu cập nhật ngay không cần logout? | chốt (a)/(b)/(c) | MAC-007 |
| S10 | CI `macos-latest`: build + `replay corpus` + (thử) AX permission trên runner | ✅/ghi → RM5 | MAC-007 |

**Exit:** 10/10 có kết quả trong `docs/specs/macos-spike.md`. S3 fail → RM1 (quy trình xcframework), S5 fail → chốt caps.

## 10. Playbook triển khai (mapping `P2-6-TASKS.md`)

| Bước | Task | Nội dung | Acceptance tóm tắt |
|---|---|---|---|
| 1 | MAC-001..009 | Env + 7 spike + corpus mac đầu | `docs/specs/macos-*-spike.md` đủ 10 mục §9 |
| 2 | MAC-010 | Bundle + IMKServer + controller rỗng | Cài vào Input menu, TextEdit không crash |
| 3 | MAC-011 | handle → `ime_key` PASS toàn bộ | Gõ tiếng Anh không đổi (corpus `combo_pass`) |
| 4 | MAC-012 | Preedit qua marked text + commit ngắn (§7) | `dduocj` → `được` trong TextEdit; corpus `tsf_preedit_*` bản mac |
| 5 | MAC-013 | Key translator (UCKeyTranslate) + layout đổi | Gõ với layout Tiếng Việt (VNI Windows) không sai |
| 6 | MAC-014 | SelectionReplace §6.2 | corpus `bug_B1_safari_url`, `bug_B1_spotlight` pass |
| 7 | MAC-015 | BackspaceType theo S5 + RESTORE | corpus `mac/*_bs_type` ≥ 30 case |
| 8 | MAC-016 | Focus/commit-before-hide + reset | corpus `bug_B13_focus_loss` pass |
| 9 | MAC-017 | Secure field (S8) → `secure=1` | corpus `secure_field_passthrough` pass (mật khẩu) |
| 10 | MAC-018 | Toggle EN/VN + hotkey (`Ctrl+Shift+Space` hoặc CapsLock mode — chốt ở spike) | `restore_en_*` pass (B5) |
| 11 | MAC-019 | IpcClient unix socket | Đổi method trong Settings → gõ đổi <1s |

## 11. Chẩn đoán

- `TEXTVN_LOG=1` → `~/Library/Logs/TextVN/im-<pid>.log` (state transition, action, **độ dài** — không text, S2).
- Menu bar → "Sức khỏe": IMK process PID + heartbeat, AX permission state, pipe/socket state.
- `textvn doctor` (CLI mac build): input source registered? socket? caps? (task MAC-058).

## 12. Failure modes

| Tình huống | Xử lý |
|---|---|
| `ime_key` lỗi/panic | `catch_unwind` trong FFI (P0-2 §5) → PASS + crash counter → status item cảnh báo |
| App không nhận marked (§7) | Phát hiện qua `markedRange` → preset `ForwardAsCommit`/`BackspaceType` cho app đó |
| Xóa ký tự cơ chế (a)/(b)/(c) fail giữa chừng | Fallback xuống cơ chế kế tiếp; cả 3 fail → `Passthrough` cho context đó + log warn |
| Socket chết/tray không chạy | Offline với config đã đọc lúc activate — không block (P0-3 §4) |
| IMK bị system kill (memory) | Relaunch tự nhiên; marked mất → hướng dẫn app auto-save (không tự phục được — ghi trong docs known-issue) |
| Tap module nhúng (nếu user bật) | Tách module, crash tap không ảnh hưởng IMK (P2-2 §9) |
