// SPDX-License-Identifier: GPL-3.0-or-later
//! Corpus phím chuyển Việt/Anh `corpus/toggle/toggle.cases` (SF-2, SF-G1/G2).
//!
//! Một nguồn kịch bản cho mọi harness (G22): crate này parse và dựng chuỗi sự kiện phím
//! (VK canonical + bit modifier đang giữ + `now_ms`); harness Rust (session, TSF) dùng
//! trực tiếp, harness C/Swift đọc cùng tệp theo đúng cú pháp ở đầu tệp. Không I/O: caller
//! đưa nội dung tệp vào [`parse`].

use crate::toggle::{MOD_ALT, MOD_CTRL, MOD_SHIFT, VK_CONTROL, VK_MENU, VK_SHIFT, VK_SPACE};
use crate::{Caps, ResetKind, Session, SessionCommand, SessionEvent};

/// Khoảng cách mặc định giữa 2 sự kiện (> debounce 250 ms).
pub const DEFAULT_GAP_MS: u64 = 300;
/// Mốc thời gian của sự kiện đầu tiên.
pub const START_MS: u64 = 1_000;
/// VK của "phím thường" `K` (chữ A).
pub const VK_OTHER: u32 = 0x41;

/// Phím trong corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorpusKey {
    /// `C`
    Ctrl,
    /// `S`
    Shift,
    /// `A` — Alt/Win/Super (macOS Option/Cmd).
    Alt,
}

impl CorpusKey {
    /// VK canonical.
    #[must_use]
    pub const fn vk(self) -> u32 {
        match self {
            Self::Ctrl => VK_CONTROL,
            Self::Shift => VK_SHIFT,
            Self::Alt => VK_MENU,
        }
    }

    const fn bit(self) -> u32 {
        match self {
            Self::Ctrl => MOD_CTRL,
            Self::Shift => MOD_SHIFT,
            Self::Alt => MOD_ALT,
        }
    }
}

/// Một bước trong kịch bản.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorpusStep {
    /// `+X`
    Down(CorpusKey),
    /// `-X`
    Up(CorpusKey),
    /// `K` — phím thường xuống + lên.
    Other,
    /// `P` — Space xuống + lên.
    Space,
    /// `R` — mất focus / reset.
    Reset,
    /// `wN` — từ đây các sự kiện cách nhau N ms.
    Gap(u64),
}

/// Một ca trong corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleCase {
    /// Tên ca.
    pub name: String,
    /// Các bước.
    pub steps: Vec<CorpusStep>,
    /// Số lần bật/tắt của session (vết vàng).
    pub session_toggles: u32,
    /// Khác biệt đã biết của mã cũ theo OS (`tsf`/`linux`/`mac`).
    pub legacy: Vec<(String, u32)>,
}

impl ToggleCase {
    /// Số lần bật/tắt mong đợi của mã cũ trên `os` (mặc định = session).
    #[must_use]
    pub fn legacy_toggles(&self, os: &str) -> u32 {
        self.legacy
            .iter()
            .find(|(o, _)| o == os)
            .map_or(self.session_toggles, |(_, n)| *n)
    }
}

/// Sự kiện phím đã dựng: `mods` = modifier đang giữ TRƯỚC sự kiện (không gồm chính phím đó).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorpusEvent {
    /// Nhấn phím.
    KeyDown {
        /// VK canonical.
        vk: u32,
        /// Bit modifier đang giữ.
        mods: u32,
        /// Mốc thời gian.
        now_ms: u64,
    },
    /// Nhả phím.
    KeyUp {
        /// VK canonical.
        vk: u32,
        /// Bit modifier đang giữ (sau khi nhả).
        mods: u32,
        /// Mốc thời gian.
        now_ms: u64,
    },
    /// Mất focus / reset.
    Reset {
        /// Mốc thời gian.
        now_ms: u64,
    },
}

/// Parse nội dung `toggle.cases`.
///
/// # Errors
/// Dòng sai cú pháp (kèm số dòng).
pub fn parse(src: &str) -> Result<Vec<ToggleCase>, String> {
    let mut cases = Vec::new();
    for (idx, raw) in src.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let ctx = || format!("toggle.cases:{}", idx + 1);
        let cols: Vec<&str> = line.split('|').map(str::trim).collect();
        if cols.len() < 3 {
            return Err(format!("{}: cần `tên | sự kiện | số`", ctx()));
        }
        let mut steps = Vec::new();
        for tok in cols[1].split_whitespace() {
            steps.push(parse_step(tok).ok_or_else(|| format!("{}: sự kiện lạ `{tok}`", ctx()))?);
        }
        let session_toggles = cols[2]
            .parse()
            .map_err(|_| format!("{}: số lần bật/tắt sai `{}`", ctx(), cols[2]))?;
        let mut legacy = Vec::new();
        for col in &cols[3..] {
            let (os, n) = col
                .split_once('=')
                .ok_or_else(|| format!("{}: cột `{col}` phải là os=số", ctx()))?;
            if !matches!(os, "tsf" | "linux" | "mac") {
                return Err(format!("{}: os lạ `{os}`", ctx()));
            }
            let n = n
                .parse()
                .map_err(|_| format!("{}: số sai trong `{col}`", ctx()))?;
            legacy.push((os.to_string(), n));
        }
        cases.push(ToggleCase {
            name: cols[0].to_string(),
            steps,
            session_toggles,
            legacy,
        });
    }
    if cases.is_empty() {
        return Err("toggle.cases: không có ca nào".into());
    }
    Ok(cases)
}

fn parse_step(tok: &str) -> Option<CorpusStep> {
    let key = |c: &str| match c {
        "C" => Some(CorpusKey::Ctrl),
        "S" => Some(CorpusKey::Shift),
        "A" => Some(CorpusKey::Alt),
        _ => None,
    };
    if let Some(k) = tok.strip_prefix('+') {
        return key(k).map(CorpusStep::Down);
    }
    if let Some(k) = tok.strip_prefix('-') {
        return key(k).map(CorpusStep::Up);
    }
    if let Some(n) = tok.strip_prefix('w') {
        return n.parse().ok().map(CorpusStep::Gap);
    }
    match tok {
        "K" => Some(CorpusStep::Other),
        "P" => Some(CorpusStep::Space),
        "R" => Some(CorpusStep::Reset),
        _ => None,
    }
}

/// Dựng chuỗi sự kiện phím của một ca (dùng chung mọi harness key-based).
#[must_use]
pub fn events(case: &ToggleCase) -> Vec<CorpusEvent> {
    let mut out = Vec::new();
    let mut held = 0u32;
    let mut now = START_MS;
    let mut gap = DEFAULT_GAP_MS;
    for step in &case.steps {
        match *step {
            CorpusStep::Gap(n) => {
                gap = n;
                continue;
            }
            CorpusStep::Down(k) => {
                out.push(CorpusEvent::KeyDown {
                    vk: k.vk(),
                    mods: held & !k.bit(),
                    now_ms: now,
                });
                held |= k.bit();
            }
            CorpusStep::Up(k) => {
                held &= !k.bit();
                out.push(CorpusEvent::KeyUp {
                    vk: k.vk(),
                    mods: held,
                    now_ms: now,
                });
            }
            CorpusStep::Other | CorpusStep::Space => {
                let vk = if *step == CorpusStep::Space {
                    VK_SPACE
                } else {
                    VK_OTHER
                };
                out.push(CorpusEvent::KeyDown {
                    vk,
                    mods: held,
                    now_ms: now,
                });
                out.push(CorpusEvent::KeyUp {
                    vk,
                    mods: held,
                    now_ms: now,
                });
            }
            CorpusStep::Reset => out.push(CorpusEvent::Reset { now_ms: now }),
        }
        now += gap;
    }
    out
}

/// Chạy một ca qua [`Session`] mới; trả số lần bật/tắt (`ModeIndicator`).
#[must_use]
pub fn run_session(case: &ToggleCase) -> u32 {
    let mut session = Session::new(Caps::none());
    let mut toggles = 0;
    for ev in events(case) {
        let cmds = match ev {
            CorpusEvent::KeyDown { vk, mods, now_ms } => session.event(SessionEvent::KeyDown {
                vk,
                ch: 0,
                mods,
                now_ms,
            }),
            CorpusEvent::KeyUp { vk, mods, now_ms } => {
                session.event(SessionEvent::KeyUp { vk, mods, now_ms })
            }
            CorpusEvent::Reset { now_ms } => session.event(SessionEvent::Reset {
                kind: ResetKind::Focus,
                now_ms,
            }),
        };
        toggles += cmds
            .as_slice()
            .iter()
            .filter(|c| matches!(c, SessionCommand::ModeIndicator { .. }))
            .count() as u32;
    }
    toggles
}
