# Performance Audit — VietIME Windows Platform

Tài liệu này ghi lại các yêu cầu hiệu năng (NFR), điểm đo lường, và kết quả audit cho Windows platform.

---

## 1. Non-Functional Requirements (NFR)

| ID | Yêu cầu | Ngưỡng |
|----|---------|--------|
| NFR-P1 | Hook callback latency (WH_KEYBOARD_LL) | < 2ms (timebox cứng) |
| NFR-P2 | TSF key event processing | < 5ms end-to-end |
| NFR-P3 | IPC roundtrip (TSF → Tray → TSF) | < 10ms |
| NFR-P4 | Startup time (`vietime-tray.exe`) | < 1 giây |
| NFR-P5 | CPU idle (tray không gõ) | < 0.1% |
| NFR-P6 | RAM working set (`vietime-tray.exe`) | < 20 MB |
| NFR-P7 | RAM working set (`vietime-tsf.dll` trong host) | < 5 MB thêm |
| NFR-P8 | DLL first-load (Activate) | < 200ms |

---

## 2. Phân tích hot path

### 2a. WH_KEYBOARD_LL Hook Callback

```
vietime-hook.exe:
  low_level_keyboard_proc()
  ├── [GUARD] IN_INJECTION.load() → 1 atomic op ~1ns
  ├── [GUARD] timebox check: Instant::now() vs 2ms budget
  ├── determine_mode() → check HookState (no alloc)
  ├── HookEngine::process_key() → pure computation
  │     └── strategy.transform() → no alloc trong hot path
  ├── [IF REPLACE] SendInput() → Windows kernel IPC
  └── CallNextHookEx() / return HC_SKIP
```

**Đã implement:**
- `IN_INJECTION: AtomicBool` — guard chống đệ quy (`Ordering::SeqCst` trong LOAD, `Release` trong STORE)
- `TIMEBOX_MS = 2` — nếu exceed → pass through (fail-open)
- Không alloc trong hot path (strategy dùng array buffer)

**Cần audit:** Xác nhận `process_key()` không alloc trong worst-case (Telex multi-char).

### 2b. TSF Key Event Sink

```
vietime-tsf.dll:
  KeySink::OnKeyDown()
  ├── KeySink::is_passthrough() → password field check (MSAA call, ~1ms)
  ├── engine.process_key() → same as above
  ├── [IF COMMIT] RequestEditSession() → async edit session
  │     └── ReplaceEditSession::DoEditSession() → SetText() + SetSelection()
  └── return BOOL
```

**Rủi ro:**
- `IAccessible::accRole()` (password detect) có thể chậm trong một số app → cần cache per-HWND
- `RequestEditSession` là async → không block key sink thread ✅

### 2c. IPC Named Pipe

```
TSF → Tray:
  IpcClient::send() → WriteFile() → pipe buffer → IpcServer → handler
  Latency: ~0.5–2ms trên máy nhanh, ~5ms trên máy cũ
```

---

## 3. CPU Usage Analysis

### Tray app (idle)
- Message loop: `GetMessageW()` → blocking (0% CPU khi idle) ✅
- IPC server: thread pool, `ConnectNamedPipe()` blocking ✅
- SvcManager: không có timer/polling ✅

### Hook process (idle)
- Message loop: `PeekMessageW` / `GetMessageW` → blocking ✅
- Không có background thread ✅

### Potential issues
- **IPC reconnect**: nếu tray crash, client TSF có thể retry loop với sleep → OK (đã có back-off)
- **Autostart check**: chạy 1 lần khi login → không liên tục ✅

---

## 4. Memory Footprint

| Component | Estimated RSS | Notes |
|-----------|--------------|-------|
| `vietime-tray.exe` | ~8-12 MB | Win32 + pipe server + config cache |
| `vietime-tsf.dll` (trong host) | ~3-5 MB | TSF COM objects + IPC client |
| `vietime-hook.exe` | ~4-6 MB | Hook + engine state |
| Config JSON | < 50 KB | Parsed once, cached |

---

## 5. Startup Time Budget

| Phase | Target | Notes |
|-------|--------|-------|
| tray: `main()` → tray icon visible | < 500ms | WndClass + Shell_NotifyIcon |
| tray: IPC server ready | < 100ms | Named pipe created |
| TSF: `Activate()` → IPC connected | < 200ms | DLL load + COM init |
| hook: SetWindowsHookExW | < 50ms | |

---

## 6. Optimization Checklist

### Done ✅
- [x] `IN_INJECTION` guard với AtomicBool — chống re-entrancy
- [x] Hook timebox 2ms — fail-open nếu processing quá chậm
- [x] `IpcServer::stop()` dùng dummy connect — không block indefinitely
- [x] `IpcClient::stop()` → `Mutex<Option<JoinHandle>>` — không deadlock
- [x] Edit session async — không block key sink thread
- [x] Null-terminator padding cho TSF `SetText` (S3-2 finding)

### Cần làm (TODO)
- [ ] Cache HWND → password field status (tránh MSAA call mỗi keystroke)
- [ ] Profile engine với `criterion` benchmark cho 1000 keystrokes mixed Telex
- [ ] Đo startup time thực tế với ETW trace
- [ ] Set process priority `ABOVE_NORMAL` cho hook.exe (như UniKey)
- [ ] Verify không có LOH allocation trong strategy.transform()

---

## 7. Benchmark targets

Chạy benchmark với:
```powershell
cargo bench -p vietime-bench
```

Expected output:
```
telex_1000_mixed    time:   [X.XX ms XX.X ms XX.X ms]  
vni_1000_mixed      time:   [X.XX ms XX.X ms XX.X ms]
```

Target: < 5ms cho 1000 keystrokes (5µs/keystroke average).

---

## 8. Windows System Timers

> [!WARNING]
> `timeBeginPeriod(1)` (1ms timer resolution) tăng CPU interrupt rate toàn hệ thống.
> VietIME **không** gọi `timeBeginPeriod` — dùng blocking I/O thay vì polling.

---

*Cập nhật lần cuối: 2026-09-27. Xem thêm: `docs/specs/performance.md`*
