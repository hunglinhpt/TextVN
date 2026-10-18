// SPDX-License-Identifier: GPL-3.0-or-later
//! Hồi quy auto-restore English (B5) trên corpus EN thật (yêu cầu audit 0.2.33):
//! gõ từ tiếng Anh + Space phải **tự trả lại đúng chuỗi gõ** — người dùng không
//! phải xoá (`Restore` thay REPLACE tại ranh giới từ). Cặp mơ hồ hai chiều
//! (`as`→`á`, `host`→`hót`, `too`→`tô`…) GIỮ TIẾNG VIỆT theo policy
//! `data/en_common.txt` (UniKey cũng vậy) — ghim ở đây để không ai lật ngược
//! policy vô tình làm hỏng gõ tiếng Việt (`thí sinh`, `chim hót`, `bõ công`…).
//!
//! Người gõ EN vẫn có đường thoát cho cặp mơ hồ: **Escape** (restore raw) hoặc
//! thêm từ vào `config.english_words`, hoặc Tab hoàn tất từ (`try_english_complete`).

use textvn_core::{Action, Engine, EngineOptions, KeyEvent};

/// Gõ `keys` + Space qua engine mặc định (mặc định sản phẩm 0.2.33: dấu cũ,
/// auto_restore_english bật), trả về text trong buffer.
fn type_word(keys: &str) -> String {
    let mut e = Engine::new(EngineOptions::default());
    let mut buf: Vec<char> = Vec::new();
    for c in keys.chars().chain(std::iter::once(' ')) {
        let k = KeyEvent::char_down(c);
        match e.key(&k).action {
            Action::Pass => buf.push(c),
            Action::Replace {
                delete_count,
                insert,
            }
            | Action::Restore {
                delete_count,
                insert,
            } => {
                buf.truncate(buf.len() - delete_count as usize);
                buf.extend(insert);
            }
            Action::Commit { insert } => buf.extend(insert),
        }
    }
    buf.into_iter().collect()
}

/// Từ EN mà Telex/VNI biến đổi nhưng **phải tự trả lại** khi gõ dấu cách:
/// fold không hợp lệ (nhánh cấu trúc) hoặc nằm trong `data/en_common.txt`.
#[test]
fn en_words_auto_restore_on_space() {
    let restore = [
        // Nhánh cấu trúc — fold ra âm tiết Việt KHÔNG hợp lệ.
        "look", "book", "good", "school", "tool", "food", "send", "event", "windows", "testing",
        "server", "remote", "desktop", "push", "pull", "request", "issue", "commit", "branch",
        "merge", "release", "build", "deploy", "swift", "linux", "wifi", "mouse", "screen",
        "browser", "website", "page", "link", "download", "upload", "install", "setup", "update",
        "login", "logout", "password", "admin", "root", "domain", "network", "port", "was", "war",
        "wax", "wow", "away", "defend", "deed", "deeds", "hex", "next", "art", "cart", "sort",
        // Nhánh từ điển EN thông dụng — fold hợp lệ nhưng không phải âm tiết Việt
        // thông dụng (`data/en_common.txt`, oracle `data_en_common_file_fires`).
        "text", "is", "see", "test", "chat", "message", "system", "file", "email", "mode", "data",
        "video", "audio", "photo", "phone", "user",
    ];
    let mut failures = Vec::new();
    for w in restore {
        let got = type_word(w);
        if got != format!("{w} ") {
            failures.push(format!("{w} → {got:?} (cần \"{w} \")"));
        }
    }
    // Từ EN không đụng phím dấu gõ thẳng từ đầu (không transform) — cũng phải
    // nguyên vẹn, không bị nuốt chữ.
    for w in [
        "java", "python", "macos", "window", "media", "keyboard", "monitor", "code", "dew", "mat",
    ] {
        let got = type_word(w);
        if got != format!("{w} ") {
            failures.push(format!("{w} → {got:?} (không được biến đổi)"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} từ EN không tự restore:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Cặp mơ hồ hai chiều GIỮ TIẾNG VIỆT (policy `en_common.txt` — UniKey cũng vậy):
/// fold là âm tiết Việt thông dụng nên không restore. Kỳ vọng lấy từ corpus
/// thật 2026-10-18 (mặc định dấu cũ 0.2.33).
#[test]
fn ambiguous_pairs_keep_vietnamese() {
    let keep = [
        ("too", "tô"),
        ("rust", "rút"),
        ("host", "hót"),
        ("list", "lít"),
        ("nest", "nét"),
        ("rest", "rét"),
        ("last", "lát"),
        ("must", "mút"),
        ("cast", "cát"),
        ("this", "thí"),
        ("his", "hí"),
        ("lost", "lót"),
        ("most", "mót"),
        ("cost", "cót"),
        ("best", "bét"),
        ("vast", "vát"),
        ("arm", "ảm"),
        ("vow", "vơ"),
        ("row", "rơ"),
        ("box", "bõ"),
        ("gas", "gá"),
        ("cow", "cơ"),
        ("sex", "sẽ"),
        ("max", "mã"),
        ("queen", "quên"),
        ("teen", "tên"),
        ("been", "bên"),
        ("moon", "môn"),
        ("has", "há"),
        ("bus", "bú"),
        ("thus", "thú"),
        ("us", "ú"),
        ("its", "ít"),
        ("as", "á"),
        ("car", "cả"),
        ("tar", "tả"),
        ("bar", "bả"),
        ("chef", "chè"),
    ];
    let mut failures = Vec::new();
    for (typed, want) in keep {
        let got = type_word(typed);
        if got != format!("{want} ") {
            failures.push(format!(
                "{typed} → {got:?} (policy giữ tiếng Việt: {want:?})"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} cặp mơ hồ không còn giữ tiếng Việt:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Toàn bộ 3000 từ tiếng Anh tần suất cao nhất (`data/en_freq3000.txt`, nguồn
/// google-10000-english MIT — yêu cầu 0.2.33 "≈3000 từ thông dụng phải gõ đúng")
/// phải gõ ra **đúng từ đó + dấu cách**: không biến đổi, tự restore cấu trúc,
/// hoặc nằm trong `data/en_common.txt`. Ngoại lệ duy nhất là các cặp mơ hồ
/// fold ra âm tiết Việt thông dụng — GIỮ TIẾNG VIỆT theo policy
/// `en_common.txt` (UniKey cũng vậy), ghim ở `AMBIGUOUS` bên dưới.
#[test]
fn top_3000_english_words_type_correctly() {
    const SOURCE: &str = include_str!("../../data/en_freq3000.txt");
    // (EN, fold tiếng Việt thông dụng được giữ) — output 2026-10-18, mặc định dấu cũ.
    const AMBIGUOUS: [(&str, &str); 54] = [
        ("of", "ò"),
        ("this", "thí"),
        ("or", "ỏ"),
        ("as", "á"),
        ("us", "ú"),
        ("if", "ì"),
        ("has", "há"),
        ("his", "hí"),
        ("now", "nơ"),
        ("been", "bên"),
        ("how", "hơ"),
        ("its", "ít"),
        ("list", "lít"),
        ("last", "lát"),
        ("most", "mót"),
        ("best", "bét"),
        ("sex", "sẽ"),
        ("must", "mút"),
        ("car", "cả"),
        ("low", "lơ"),
        ("teen", "tên"),
        ("too", "tô"),
        ("box", "bõ"),
        ("cost", "cót"),
        ("bar", "bả"),
        ("lost", "lót"),
        ("usr", "ủ"),
        ("gas", "gá"),
        ("thus", "thú"),
        ("max", "mã"),
        ("rest", "rét"),
        ("host", "hót"),
        ("os", "ó"),
        ("lots", "lót"),
        ("ex", "ẽ"),
        ("bus", "bú"),
        ("ar", "ả"),
        ("cast", "cát"),
        ("queen", "quên"),
        ("es", "é"),
        ("row", "rơ"),
        ("moon", "môn"),
        // Mở rộng 2026-10-18: âm tiết Việt KHÔNG có cách gõ Telex khác (R2-56).
        ("hair", "hải"),
        ("las", "lá"),
        ("six", "sĩ"),
        ("own", "ơn"),
        ("lee", "lê"),
        ("chair", "chải"),
        ("tax", "tã"),
        ("tree", "trê"),
        ("ups", "úp"),
        ("var", "vả"),
        ("wa", "ưa"),
        ("mar", "mả"),
    ];
    let mut failures = Vec::new();
    for line in SOURCE.lines() {
        let w = line.trim();
        if w.is_empty() || w.starts_with('#') || !w.chars().all(|c| c.is_ascii_alphabetic()) {
            continue;
        }
        if let Some((_, vn)) = AMBIGUOUS.iter().find(|(en, _)| *en == w) {
            let got = type_word(w);
            if got != format!("{vn} ") {
                failures.push(format!("{w} → {got:?} (policy giữ tiếng Việt: {vn:?})"));
            }
        } else {
            let got = type_word(w);
            if got != format!("{w} ") {
                failures.push(format!("{w} → {got:?} (cần \"{w} \")"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} từ trong top-3000 EN gõ sai:
{}",
        failures.len(),
        failures.join(
            "
"
        )
    );
}
