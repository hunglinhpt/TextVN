// SPDX-License-Identifier: GPL-3.0-or-later
//! Bất biến đầu ra P0-6 §3 chạy trên session rỗng (SF-1a).
//!
//! Quy ước test của repo (P0-4): không crate dev ngoài — PRNG tự viết (splitmix64),
//! tất định theo seed, không dùng đồng hồ. Khi SF-2+ thêm logic thật, các test này
//! phải vẫn xanh (luật đầu ra là CHUNG mọi OS — G22).

use textvn_session::{
    AssistAction, AssistItem, Caps, Commands, ResetKind, Session, SessionCommand, SessionEvent,
    TimerId,
};

/// PRNG splitmix64 — tất định, không dependency (P0-4: test không dùng clock/IO).
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

const APP_IDS: [&str; 3] = ["notepad", "code", "term"];
const BEFORE_TEXTS: [&str; 4] = ["", "xin chào ", "héllo", " tiếng việt"];

fn make_event(rng: &mut Rng, step: u64) -> SessionEvent<'static> {
    match rng.below(10) {
        0 => SessionEvent::KeyDown {
            vk: rng.below(256) as u32,
            ch: if rng.below(4) == 0 {
                0
            } else {
                0x61 + rng.below(26) as u32
            },
            mods: rng.below(8) as u32,
            now_ms: step * 10,
        },
        1 => SessionEvent::KeyUp {
            vk: rng.below(256) as u32,
            mods: rng.below(8) as u32,
            now_ms: step * 10,
        },
        2 => SessionEvent::FocusIn {
            app_id: APP_IDS[rng.below(3) as usize],
            field_role: rng.below(8) as u32,
            secure: rng.below(4) == 0,
            terminal: rng.below(8) == 0,
        },
        3 => SessionEvent::FocusOut,
        4 => SessionEvent::CaretMoved,
        5 => SessionEvent::Surrounding {
            before: BEFORE_TEXTS[rng.below(4) as usize],
        },
        6 => SessionEvent::Reset {
            kind: match rng.below(3) {
                0 => ResetKind::Toolkit,
                1 => ResetKind::Focus,
                _ => ResetKind::App,
            },
            now_ms: step * 10,
        },
        7 => SessionEvent::Timer {
            id: TimerId(rng.below(4) as u32),
            now_ms: step * 10,
        },
        8 => SessionEvent::Ipc(textvn_ipc::Message::Ping),
        _ => SessionEvent::AssistAction {
            action: match rng.below(3) {
                0 => AssistAction::Select {
                    index: rng.below(8) as u16,
                },
                1 => AssistAction::Dismiss,
                _ => AssistAction::Undo,
            },
        },
    }
}

fn count_cmds(out: &[SessionCommand], f: impl Fn(&SessionCommand) -> bool) -> usize {
    out.iter().filter(|c| f(c)).count()
}

fn is_commit(c: &SessionCommand) -> bool {
    matches!(c, SessionCommand::Commit { .. })
}

fn is_delete(c: &SessionCommand) -> bool {
    matches!(c, SessionCommand::DeleteBefore { .. })
}

fn is_key_decision(c: &SessionCommand) -> bool {
    matches!(c, SessionCommand::ForwardKey | SessionCommand::EatKey)
}

fn is_hide_preedit(c: &SessionCommand) -> bool {
    matches!(c, SessionCommand::SetPreedit { text, .. } if text.is_empty())
}

/// Lệnh chứa nội dung cho người dùng/cửa sổ khác: ghost, trợ lý, IPC (S3).
fn is_text_output(c: &SessionCommand) -> bool {
    matches!(
        c,
        SessionCommand::Ghost { .. }
            | SessionCommand::GhostAfterKeyUp
            | SessionCommand::AssistShow { .. }
            | SessionCommand::Ipc(_)
    )
}

/// Khung rỗng: KeyDown ⇒ đúng chuỗi `[ForwardKey]`, với MỌI cấu hình Caps.
#[test]
fn khung_rong_keydown_tra_dung_mot_forwardkey() {
    let caps_list = [
        Caps::none(),
        Caps {
            preedit: true,
            surrounding_text: true,
            delete_surrounding: true,
            inline_ghost: true,
            key_up_events: true,
            forward_key: true,
        },
        Caps {
            forward_key: true,
            ..Caps::none()
        },
    ];
    for caps in caps_list {
        let mut session = Session::new(caps);
        let out = session.event(SessionEvent::KeyDown {
            vk: 0x41,
            ch: 0x61,
            mods: 0,
            now_ms: 0,
        });
        assert_eq!(out.as_slice(), [SessionCommand::ForwardKey]);
    }
}

/// Khung rỗng: event không phải KeyDown ⇒ chuỗi lệnh rỗng (chưa có logic SF-2+).
#[test]
fn khung_rong_event_khac_keydown_tra_rong() {
    let mut session = Session::new(Caps::none());
    let events: [SessionEvent<'static>; 9] = [
        SessionEvent::KeyUp {
            vk: 0x41,
            mods: 0,
            now_ms: 1,
        },
        SessionEvent::FocusIn {
            app_id: "notepad",
            field_role: 0,
            secure: false,
            terminal: false,
        },
        SessionEvent::FocusOut,
        SessionEvent::CaretMoved,
        SessionEvent::Surrounding { before: "abc" },
        SessionEvent::Reset {
            kind: ResetKind::Toolkit,
            now_ms: 2,
        },
        SessionEvent::Timer {
            id: TimerId(1),
            now_ms: 3,
        },
        SessionEvent::Ipc(textvn_ipc::Message::Ping),
        SessionEvent::AssistAction {
            action: AssistAction::Dismiss,
        },
    ];
    for ev in events {
        let is_reset = matches!(ev, SessionEvent::Reset { .. });
        let out = session.event(ev);
        if is_reset {
            // SF-3 (R2-171/E78): mỗi Reset ra đúng một EngineReset (không có Enter ⇒ xoá sạch).
            assert_eq!(
                out.as_slice(),
                &[SessionCommand::EngineReset { line_start: false }]
            );
        } else {
            assert!(
                out.is_empty(),
                "khung rỗng phải trả chuỗi rỗng cho event không phải KeyDown/Reset"
            );
        }
    }
}

/// Luật R2-165 (P0-6 §3): mỗi event tối đa 1 DeleteBefore + tối đa 1 Commit.
#[test]
fn thuoc_tinh_moi_event_toi_da_1_xoa_1_commit() {
    let mut so_loai_event = [0_u64; 10];
    for seed in 0..64_u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let mut session = Session::new(Caps {
            preedit: true,
            surrounding_text: true,
            delete_surrounding: true,
            inline_ghost: true,
            key_up_events: true,
            forward_key: true,
        });
        session.event(SessionEvent::FocusIn {
            app_id: "notepad",
            field_role: 0,
            secure: false,
            terminal: false,
        });
        for step in 0..512_u64 {
            let ev = make_event(&mut rng, step);
            let loai = match &ev {
                SessionEvent::KeyDown { .. } => 0,
                SessionEvent::KeyUp { .. } => 1,
                SessionEvent::FocusIn { .. } => 2,
                SessionEvent::FocusOut => 3,
                SessionEvent::CaretMoved => 4,
                SessionEvent::Surrounding { .. } => 5,
                SessionEvent::Reset { .. } => 6,
                SessionEvent::Timer { .. } => 7,
                SessionEvent::Ipc(_) => 8,
                SessionEvent::AssistAction { .. } => 9,
            };
            so_loai_event[loai] += 1;
            let out = session.event(ev);
            assert!(
                count_cmds(out.as_slice(), is_commit) <= 1,
                "seed {seed} step {step}: vượt 1 Commit"
            );
            assert!(
                count_cmds(out.as_slice(), is_delete) <= 1,
                "seed {seed} step {step}: vượt 1 DeleteBefore"
            );
        }
    }
    // Bộ sinh phải phủ đủ 10 loại event để luật có ý nghĩa với mọi variant.
    for (loai, n) in so_loai_event.iter().enumerate() {
        assert!(*n > 0, "bộ sinh không phủ event loại {loai}");
    }
}

/// Luật R2-167 (P0-6 §3): Commit luôn đứng TRƯỚC ẩn preedit `SetPreedit{""}`.
#[test]
fn thuoc_tinh_commit_truoc_khi_an_preedit() {
    for seed in 0..64_u64 {
        let mut rng = Rng(seed ^ 0xA5A5_5A5A_0000_0001);
        let mut session = Session::new(Caps::none());
        for step in 0..512_u64 {
            let ev = make_event(&mut rng, step);
            let out = session.event(ev);
            let mut da_an_preedit = false;
            for cmd in out.as_slice() {
                if is_hide_preedit(cmd) {
                    da_an_preedit = true;
                }
                if is_commit(cmd) {
                    assert!(
                        !da_an_preedit,
                        "seed {seed} step {step}: Commit đứng SAU khi ẩn preedit (R2-167)"
                    );
                }
            }
        }
    }
}

/// Luật P0-6 §3: KeyDown có đúng MỘT quyết định phím; event khác KHÔNG có.
#[test]
fn thuoc_tinh_dung_mot_quyet_dinh_phim_cho_keydown() {
    for seed in 0..64_u64 {
        let mut rng = Rng(seed.wrapping_add(0xDEAD_BEEF));
        let mut session = Session::new(Caps::none());
        for step in 0..512_u64 {
            let ev = make_event(&mut rng, step);
            let la_keydown = matches!(ev, SessionEvent::KeyDown { .. });
            let out = session.event(ev);
            let so_quyet_dinh = count_cmds(out.as_slice(), is_key_decision);
            if la_keydown {
                assert_eq!(
                    so_quyet_dinh, 1,
                    "seed {seed} step {step}: KeyDown phải có đúng 1 ForwardKey/EatKey"
                );
            } else {
                assert_eq!(
                    so_quyet_dinh, 0,
                    "seed {seed} step {step}: event không phải KeyDown không được trả ForwardKey/EatKey"
                );
            }
        }
    }
}

/// Luật S3 (P0-6 §3): vào ô secure — không Ghost, không AssistShow, không IPC.
/// (Khung rỗng không bao giờ trả lệnh chứa chữ; test khoá luật để SF-2+ không phá.)
#[test]
fn thuoc_tinh_secure_khong_ghost_assist_ipc() {
    for seed in 0..32_u64 {
        let mut rng = Rng(seed.wrapping_add(0x5E3_9001));
        let mut session = Session::new(Caps {
            preedit: true,
            inline_ghost: true,
            ..Caps::none()
        });
        // Vào ô secure ngay đầu stream, rồi bấm phím/caret/reset trong 256 bước.
        let focus = SessionEvent::FocusIn {
            app_id: APP_IDS[1],
            field_role: 0,
            secure: true,
            terminal: false,
        };
        let out_focus = session.event(focus);
        assert!(count_cmds(out_focus.as_slice(), is_text_output) == 0);
        for step in 0..256_u64 {
            let ev = make_event(&mut rng, step);
            let out = session.event(ev);
            assert!(
                count_cmds(out.as_slice(), is_text_output) == 0,
                "seed {seed} step {step}: lệnh chứa chữ trong ô secure (S3)"
            );
        }
    }
}

/// Tất định (P0-6 §2 bất biến 1): cùng stream ⇒ cùng chuỗi lệnh, chạy 2 lần giống hệt.
#[test]
fn tat_dinh_chay_hai_lan_giong_he() {
    fn chay_mot_luot() -> Vec<String> {
        let mut rng = Rng(42);
        let mut session = Session::new(Caps::none());
        let mut log = Vec::new();
        for step in 0..256_u64 {
            let ev = make_event(&mut rng, step);
            let out = session.event(ev);
            let hang: Vec<String> = out.as_slice().iter().map(|c| format!("{c:?}")).collect();
            log.push(hang.join(";"));
        }
        log
    }
    let lan_1 = chay_mot_luot();
    let lan_2 = chay_mot_luot();
    assert_eq!(lan_1, lan_2);
}

/// `Commands` cho phép trình tự hợp lệ: Commit TRƯỚC ẩn preedit (R2-167).
#[test]
fn commands_cho_phep_commit_truoc_an_preedit() {
    let mut cmds = Commands::new();
    cmds.push(SessionCommand::SetPreedit {
        text: "ă".into(),
        cursor: 1,
    });
    cmds.push(SessionCommand::Commit { text: "ă".into() });
    cmds.push(SessionCommand::SetPreedit {
        text: String::new(),
        cursor: 0,
    });
    cmds.push(SessionCommand::ForwardKey);
    assert_eq!(cmds.len(), 4);
    assert!(!cmds.is_empty());
    let v = cmds.into_vec();
    assert_eq!(count_cmds(&v, is_commit), 1);
    assert_eq!(count_cmds(&v, is_hide_preedit), 1);
}

/// Luật đột biến (D3): cho phép 2 Commit trong một event ⇒ test đỏ.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "R2-165")]
fn commands_tu_choi_2_commit() {
    let mut cmds = Commands::new();
    cmds.push(SessionCommand::Commit { text: "a".into() });
    cmds.push(SessionCommand::Commit { text: "b".into() });
}

/// Luật R2-165: 2 DeleteBefore trong một event ⇒ từ chối.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "R2-165")]
fn commands_tu_choi_2_xoa() {
    let mut cmds = Commands::new();
    cmds.push(SessionCommand::DeleteBefore { chars: 1 });
    cmds.push(SessionCommand::DeleteBefore { chars: 2 });
}

/// Luật R2-167: Commit SAU khi ẩn preedit ⇒ từ chối.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "R2-167")]
fn commands_tu_choi_commit_sau_an_preedit() {
    let mut cmds = Commands::new();
    cmds.push(SessionCommand::SetPreedit {
        text: String::new(),
        cursor: 0,
    });
    cmds.push(SessionCommand::Commit { text: "a".into() });
}

/// Luật P0-6 §3: 2 quyết định phím (ForwardKey + EatKey) trong một KeyDown ⇒ từ chối.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "dung MOT")]
fn commands_tu_choi_2_quyet_dinh_phim() {
    let mut cmds = Commands::new();
    cmds.push(SessionCommand::ForwardKey);
    cmds.push(SessionCommand::EatKey);
}

/// API hỗ trợ tiêu thụ chuỗi lệnh: `IntoIterator` + `AssistShow` mang `Vec<AssistItem>`.
#[test]
fn commands_tieu_thu_bang_iterator() {
    let mut cmds = Commands::new();
    cmds.push(SessionCommand::AssistShow {
        items: vec![AssistItem {
            text: "gợi ý".into(),
            replace: "đề xuất".into(),
        }],
    });
    cmds.push(SessionCommand::AssistHide);
    let dem: usize = cmds.into_iter().count();
    assert_eq!(dem, 2);
}

/// `Caps` mặc định = không năng lực; `Session` giữ đúng caps khai báo.
#[test]
fn caps_mac_dinh_va_session_giu_caps() {
    assert_eq!(Caps::default(), Caps::none());
    let caps = Caps {
        forward_key: true,
        ..Caps::none()
    };
    let session = Session::new(caps);
    assert_eq!(session.caps(), caps);
}
