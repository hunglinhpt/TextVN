// SPDX-License-Identifier: GPL-3.0-or-later
//! R2-26 — kiểm thử đầu-cuối trên tiến trình **AppContainer** và **LPAC** THẬT (cùng mô hình
//! token với ô tìm kiếm Start / Calculator / ứng dụng Store).
//!
//! Test cha dựng đúng mã phát hành: `SvcManager` (ghi config + bản sao sandbox có ACL),
//! `IpcServer` với pipe đầy đủ + pipe sandbox (tên riêng của test — tray thật có thể đang chạy).
//! Rồi chạy CHÍNH binary test này trong AppContainer (`CreateAppContainerProfile` +
//! `PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES`), lần hai thêm cờ LPAC. Tiến trình con chứng
//! minh: config thật bị từ chối (lỗi 5), bản sao đọc được và không chứa gõ tắt, pipe đầy đủ bị
//! từ chối → `open_tray_pipe_from` rơi xuống pipe sandbox, snapshot không lộ override app khác,
//! lệnh nguy hiểm bị từ chối, Ctrl+Shift toàn cục tới được tray.
//!
//! `TEXTVN_SKIP_APPCONTAINER_E2E=1` bỏ qua (môi trường không tạo được hồ sơ AppContainer).

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use textvn_ipc::{decode_exact_frame, encode_frame, Message, MAX_FRAME_BYTES};

const CHILD_ENV: &str = "TEXTVN_AC_E2E_CHILD";
const SECRET: &str = "R2-26-SECRET-12 Lê Lợi 0912345678";

fn send(stream: &mut std::fs::File, msg: &Message) {
    let frame = encode_frame(msg).expect("encode");
    stream.write_all(&frame).expect("ghi pipe");
    stream.flush().expect("flush pipe");
}

fn recv(stream: &mut std::fs::File) -> Message {
    let mut len = [0u8; 4];
    stream.read_exact(&mut len).expect("đọc độ dài");
    let n = u32::from_le_bytes(len) as usize;
    assert!(n <= MAX_FRAME_BYTES);
    let mut frame = vec![0u8; 4 + n];
    frame[..4].copy_from_slice(&len);
    stream.read_exact(&mut frame[4..]).expect("đọc frame");
    decode_exact_frame(&frame).expect("decode")
}

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("thiếu biến {name}"))
}

/// Token tiến trình hiện tại là AppContainer?
fn is_app_container() -> bool {
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Security::{GetTokenInformation, TokenIsAppContainer, TOKEN_QUERY};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    // SAFETY: handle token đóng ngay; buffer 4 byte cho DWORD.
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut value = 0u32;
        let mut ret = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenIsAppContainer,
            Some(&mut value as *mut u32 as *mut _),
            4,
            &mut ret,
        )
        .is_ok();
        let _ = CloseHandle(token);
        ok && value != 0
    }
}

fn own_app_id() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().to_ascii_lowercase())
        })
        .unwrap_or_default()
}

/// Tiến trình CON (chạy trong AppContainer/LPAC). Không có biến môi trường → không làm gì
/// (khi `cargo test` thường chạy tới nó).
#[test]
fn child_in_appcontainer() {
    if std::env::var_os(CHILD_ENV).is_none() {
        return;
    }
    assert!(
        is_app_container(),
        "tiến trình con phải chạy trong AppContainer"
    );
    let config_dir = PathBuf::from(env("TEXTVN_AC_E2E_DIR"));

    // 1. Lỗi gốc R2-26: config thật bị từ chối.
    let real = std::fs::read(config_dir.join("config.json"));
    assert_eq!(
        real.as_ref().err().and_then(|e| e.raw_os_error()),
        Some(5),
        "config.json thật phải bị từ chối trong sandbox"
    );

    // 2. Bản sao đọc được, đúng tuỳ chọn, không có nội dung người dùng.
    let mirror = [config_dir.join(textvn_ipc::sandbox::SANDBOX_DIR_NAME)];
    let cfg = textvn_ipc::sandbox::read_sandbox_file_in(&mirror, "config.json")
        .expect("đọc bản sao config");
    let cfg = String::from_utf8(cfg).unwrap();
    assert!(!cfg.contains(SECRET), "gõ tắt lọt vào bản sao sandbox");
    assert_eq!(
        textvn_config::parse_config(&cfg).unwrap().method,
        textvn_config::Method::Vni,
        "tiến trình sandbox phải theo kiểu gõ của người dùng"
    );
    assert!(textvn_ipc::sandbox::read_sandbox_file_in(&mirror, "state.json").is_some());
    assert!(
        textvn_ipc::sandbox::current_process_is_app_container(),
        "TIP phải nhận ra mình chạy trong AppContainer (chỉ khi đó mới đọc bản sao)"
    );

    // 3. Pipe đầy đủ bị từ chối → rơi xuống pipe sandbox (server xác thực theo phiên).
    let full = env("TEXTVN_AC_E2E_FULL");
    let sandbox = env("TEXTVN_AC_E2E_SANDBOX");
    assert_eq!(
        textvn_ipc::pipe_client_options()
            .open(&full)
            .err()
            .and_then(|e| e.raw_os_error()),
        Some(5),
        "pipe đầy đủ không được mở cho sandbox"
    );
    let mut stream =
        match textvn_ipc::sandbox::open_tray_pipe_from(std::slice::from_ref(&full), &sandbox) {
            textvn_ipc::sandbox::PipeOpen::Connected { file, sandboxed } => {
                assert!(sandboxed, "phải nối qua pipe sandbox");
                file
            }
            textvn_ipc::sandbox::PipeOpen::Busy(_) => panic!("pipe sandbox bận"),
            textvn_ipc::sandbox::PipeOpen::Unavailable => panic!("không nối được pipe sandbox"),
        };

    // 3b. Audit 0.2.93: sandbox KHÔNG tạo được instance server của pipe sandbox (GENERIC_WRITE
    // của pipe gồm FILE_CREATE_PIPE_INSTANCE — DACL chỉ cấp quyền dữ liệu).
    {
        use windows::core::PCWSTR;
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
        use windows::Win32::System::Pipes::{CreateNamedPipeW, PIPE_TYPE_BYTE, PIPE_WAIT};
        let name: Vec<u16> = sandbox.encode_utf16().chain(Some(0)).collect();
        // SAFETY: tên NUL-terminated; handle (nếu có) đóng ngay.
        let squat = unsafe {
            CreateNamedPipeW(
                PCWSTR(name.as_ptr()),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_WAIT,
                255,
                512,
                512,
                0,
                None,
            )
        };
        let created = !squat.is_invalid();
        if created {
            // SAFETY: handle hợp lệ vừa tạo.
            unsafe {
                let _ = CloseHandle(squat);
            }
        }
        assert!(
            !created,
            "sandbox tạo được instance của pipe sandbox — mạo danh tray"
        );
    }

    // 4. Snapshot: thấy `*`, `assist`, override của chính mình — KHÔNG thấy app khác.
    let pid = std::process::id();
    send(
        &mut stream,
        &Message::Hello {
            pid,
            abi: 1,
            version: "e2e".into(),
        },
    );
    let Message::Snapshot { state, .. } = recv(&mut stream) else {
        panic!("Hello phải nhận Snapshot");
    };
    assert!(state.contains_key("*") && state.contains_key("assist"));
    assert_eq!(
        state.get(&own_app_id()),
        Some(&false),
        "override của chính app"
    );
    assert!(
        !state.contains_key("chrome.exe"),
        "lộ override app khác: {state:?}"
    );

    // 5. Lệnh nguy hiểm bị từ chối (Ack, không làm gì).
    for msg in [
        Message::Shutdown,
        Message::ToggleViEn {
            app_id: "chrome.exe".into(),
            enabled: true,
        },
        Message::ToggleMath,
        Message::AssistNotification {
            title: "x".into(),
            message: "y".into(),
        },
    ] {
        send(&mut stream, &msg);
        assert_eq!(recv(&mut stream), Message::Ack, "{msg:?} phải bị từ chối");
    }
    // Drain tự học: chỉ còn bộ đếm (từ "mậtkhẩu" không được vào hồ sơ).
    send(
        &mut stream,
        &Message::StyleUpdate {
            json: r#"{"words":[{"word":"mậtkhẩu","count":9}],"rsi_delta":{"keys_typed":5}}"#.into(),
        },
    );
    assert_eq!(recv(&mut stream), Message::Ack);

    // 6. Ctrl+Shift toàn cục từ ô tìm kiếm Start tới tray.
    let want: bool = env("TEXTVN_AC_E2E_TOGGLE") == "1";
    // E82: chống dội 250 ms của tray (`try_claim_global_toggle`) là trạng thái TOÀN tiến trình —
    // test khác của cùng binary chạy song song có thể vừa chiếm cửa sổ ⇒ lệnh bị bỏ qua đúng
    // thiết kế (tray trả snapshot hiện tại). Thử lại sau cửa sổ, như TIP thật khi người dùng bấm lại.
    let mut last = None;
    for _ in 0..6 {
        send(
            &mut stream,
            &Message::ToggleViEn {
                app_id: "*".into(),
                enabled: want,
            },
        );
        let Message::Snapshot { state, .. } = recv(&mut stream) else {
            panic!("ToggleViEn phải nhận Snapshot");
        };
        assert!(!state.contains_key("chrome.exe"));
        last = state.get("*").copied();
        if last == Some(want) {
            break;
        }
        std::thread::sleep(Duration::from_millis(300));
    }
    assert_eq!(last, Some(want), "Ctrl+Shift từ sandbox phải tới tray");
}

/// Chạy `exe` trong AppContainer `sid` (LPAC nếu `lpac`), đợi tối đa 60 s, trả exit code.
fn run_in_appcontainer(
    exe: &Path,
    sid: windows::Win32::Security::PSID,
    lpac: bool,
    vars: &[(&str, String)],
) -> u32 {
    use windows::core::{PCWSTR, PWSTR};
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Security::SECURITY_CAPABILITIES;
    use windows::Win32::System::Threading::{
        CreateProcessW, DeleteProcThreadAttributeList, GetExitCodeProcess,
        InitializeProcThreadAttributeList, TerminateProcess, UpdateProcThreadAttribute,
        WaitForSingleObject, CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT,
        LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION,
        PROC_THREAD_ATTRIBUTE_ALL_APPLICATION_PACKAGES_POLICY,
        PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES, STARTUPINFOEXW,
    };
    // Khối môi trường = môi trường hiện tại + biến của test (UTF-16, kết thúc 2 NUL).
    let mut block: Vec<u16> = Vec::new();
    for (k, v) in std::env::vars_os() {
        let k = k.to_string_lossy();
        if vars.iter().any(|(name, _)| name.eq_ignore_ascii_case(&k)) {
            continue;
        }
        block.extend(format!("{k}={}", v.to_string_lossy()).encode_utf16());
        block.push(0);
    }
    for (k, v) in vars {
        block.extend(format!("{k}={v}").encode_utf16());
        block.push(0);
    }
    block.push(0);
    let cmd = format!(
        "\"{}\" sandbox_e2e::child_in_appcontainer --exact --nocapture --test-threads=1",
        exe.display()
    );
    let mut cmd_w: Vec<u16> = cmd.encode_utf16().chain(Some(0)).collect();
    // SAFETY: danh sách thuộc tính sống tới sau CreateProcessW; SECURITY_CAPABILITIES và cờ
    // LPAC sống trên stack cùng phạm vi; handle tiến trình/thread được đóng.
    unsafe {
        let mut caps = SECURITY_CAPABILITIES {
            AppContainerSid: sid,
            Capabilities: std::ptr::null_mut(),
            CapabilityCount: 0,
            Reserved: 0,
        };
        let count = if lpac { 2 } else { 1 };
        let mut size = 0usize;
        let _ = InitializeProcThreadAttributeList(None, count, None, &mut size);
        let mut buf = vec![0u8; size];
        let list = LPPROC_THREAD_ATTRIBUTE_LIST(buf.as_mut_ptr() as *mut _);
        InitializeProcThreadAttributeList(Some(list), count, None, &mut size)
            .expect("InitializeProcThreadAttributeList");
        UpdateProcThreadAttribute(
            list,
            0,
            PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
            Some(&mut caps as *mut _ as *const _),
            std::mem::size_of::<SECURITY_CAPABILITIES>(),
            None,
            None,
        )
        .expect("SECURITY_CAPABILITIES");
        // PROCESS_CREATION_ALL_APPLICATION_PACKAGES_OPT_OUT = 1 → LPAC.
        let opt_out: u32 = 1;
        if lpac {
            UpdateProcThreadAttribute(
                list,
                0,
                PROC_THREAD_ATTRIBUTE_ALL_APPLICATION_PACKAGES_POLICY as usize,
                Some(&opt_out as *const u32 as *const _),
                std::mem::size_of::<u32>(),
                None,
                None,
            )
            .expect("LPAC policy");
        }
        let mut si = STARTUPINFOEXW::default();
        si.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
        si.lpAttributeList = list;
        let mut pi = PROCESS_INFORMATION::default();
        let created = CreateProcessW(
            PCWSTR::null(),
            Some(PWSTR(cmd_w.as_mut_ptr())),
            None,
            None,
            true,
            EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
            Some(block.as_ptr() as *const _),
            PCWSTR::null(),
            &si.StartupInfo,
            &mut pi,
        );
        DeleteProcThreadAttributeList(list);
        created.expect("CreateProcessW trong AppContainer");
        let waited = WaitForSingleObject(pi.hProcess, 60_000);
        if waited.0 != 0 {
            let _ = TerminateProcess(pi.hProcess, 99);
        }
        let mut code = 0u32;
        let _ = GetExitCodeProcess(pi.hProcess, &mut code);
        let _ = CloseHandle(pi.hThread);
        let _ = CloseHandle(pi.hProcess);
        code
    }
}

#[test]
fn sandbox_appcontainer_end_to_end() {
    if std::env::var_os(CHILD_ENV).is_some()
        || std::env::var_os("TEXTVN_SKIP_APPCONTAINER_E2E").is_some()
    {
        return;
    }
    use windows::core::HSTRING;
    use windows::Win32::Security::FreeSid;
    use windows::Win32::Security::Isolation::{
        CreateAppContainerProfile, DeleteAppContainerProfile,
        DeriveAppContainerSidFromAppContainerName,
    };

    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("textvn-ac-e2e-{pid}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // config người dùng có gõ tắt chứa "địa chỉ + số điện thoại" (nội dung riêng tư).
    std::fs::write(
        dir.join("config.json"),
        format!(
            r#"{{"config_version":1,"method":"vni","macros":[{{"trigger":"dc","expand":"{SECRET}"}}]}}"#
        ),
    )
    .unwrap();
    let svc = crate::svc::SvcManager::new(Some(dir.clone()));
    let me = own_app_id();
    svc.set_app_enabled("chrome.exe", false);
    svc.set_app_enabled(&me, false);
    assert!(
        !std::fs::read_to_string(dir.join("sandbox").join("config.json"))
            .unwrap()
            .contains(SECRET),
        "bản sao sandbox không được chứa gõ tắt"
    );

    let full = format!(r"\\.\pipe\textvn-ipc-test-{pid}-acfull");
    let sandbox = format!(r"\\.\pipe\textvn-ipc-test-{pid}-acsandbox");
    let server = crate::ipc_server::IpcServer::with_pipes(
        svc.clone(),
        &[(full.clone(), false), (sandbox.clone(), true)],
    );
    server.start();
    std::thread::sleep(Duration::from_millis(150));

    let exe = std::env::current_exe().unwrap();
    // Binary test phải đọc + chạy được bởi AppContainer/LPAC (như `register` cấp cho DLL).
    let granted = std::process::Command::new("icacls")
        .arg(&exe)
        .args(["/grant", "*S-1-15-2-1:(RX)", "*S-1-15-2-2:(RX)", "/Q"])
        .status()
        .expect("icacls");
    assert!(granted.success(), "không cấp được quyền RX cho binary test");

    let name = HSTRING::from(format!("TextVN.Test.R2-26.{pid}"));
    // SAFETY: SID do API cấp, FreeSid đúng một lần; hồ sơ xoá ở cuối test.
    let sid = unsafe {
        match CreateAppContainerProfile(&name, &name, &name, None) {
            Ok(sid) => sid,
            Err(_) => DeriveAppContainerSidFromAppContainerName(&name)
                .expect("tạo/lấy hồ sơ AppContainer (đặt TEXTVN_SKIP_APPCONTAINER_E2E=1 nếu môi trường không hỗ trợ)"),
        }
    };

    let mut results = Vec::new();
    for (lpac, toggle) in [(false, false), (true, true)] {
        let denied_before = server.sandbox_denied();
        let code = run_in_appcontainer(
            &exe,
            sid,
            lpac,
            &[
                (CHILD_ENV, "1".into()),
                ("TEXTVN_AC_E2E_DIR", dir.display().to_string()),
                ("TEXTVN_AC_E2E_FULL", full.clone()),
                ("TEXTVN_AC_E2E_SANDBOX", sandbox.clone()),
                (
                    "TEXTVN_AC_E2E_TOGGLE",
                    if toggle { "1" } else { "0" }.into(),
                ),
            ],
        );
        // Chờ handler kịp xử lý lệnh cuối (Ack đã về trước khi con thoát).
        std::thread::sleep(Duration::from_millis(100));
        results.push((
            lpac,
            code,
            server.sandbox_denied() - denied_before,
            svc.is_global_enabled(),
        ));
        std::thread::sleep(Duration::from_millis(300)); // qua cửa sổ debounce 250 ms
    }

    server.stop();
    // SAFETY: SID còn sống; xoá hồ sơ test.
    unsafe {
        FreeSid(sid);
        let _ = DeleteAppContainerProfile(&name);
    }
    for (lpac, code, denied, global) in &results {
        let kind = if *lpac { "LPAC" } else { "AppContainer" };
        assert_eq!(
            *code, 0,
            "{kind}: tiến trình con thất bại (xem log phía trên)"
        );
        assert!(
            *denied >= 4,
            "{kind}: lệnh nguy hiểm phải bị từ chối (đếm {denied}) — con có chạy thật không?"
        );
        assert_eq!(
            *global, *lpac,
            "{kind}: Ctrl+Shift từ sandbox phải tới tray"
        );
    }
    assert_eq!(
        svc.app_override("chrome.exe"),
        Some(false),
        "sandbox không được đổi override app khác"
    );
    assert!(
        !svc.config()
            .style_profile
            .words
            .iter()
            .any(|w| w.word.contains("mậtkhẩu")),
        "từ từ sandbox không được vào hồ sơ học"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
