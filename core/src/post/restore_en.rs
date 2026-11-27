// SPDX-License-Identifier: GPL-3.0-or-later
//! Auto-restore English — bug **B5** (`PLAN §1.3`: "gõ tiếng Anh ra dấu").
//!
//! Ba nhánh, thứ tự kiểm tra trong `should_restore`:
//!
//! 1. **Cấu trúc**: từ đã bị transform mà kết quả **không phải âm tiết Việt**
//!    → trả lại đúng chuỗi phím người dùng gõ khi gặp ranh giới từ.
//!    Ví dụ: `asdf` → fold `àd` (không hợp lệ) → Space → `asdf `; `download`
//!    → `dơwnload` (không hợp lệ) → Space → `download `.
//!
//! 2. **Từ điển EN thông dụng dựng sẵn** (bật mặc định từ 2026-10-02): chuỗi
//!    gõ nằm trong `data/en_common.txt` — những từ EN mà Telex biến thành âm
//!    tiết Việt **hợp lệ về cấu trúc** (nhánh 1 không bắt được: `text` →
//!    `tẽt`, `see` → `sê`, `is` → `í`) → trả lại chuỗi gõ. Cặp mơ hồ hai chiều
//!    (`cow` là cách gõ Telex của `cơ`, `sex` → `sẽ`, `queen` → `quên`…) được
//!    giải quyết theo hướng **giữ tiếng Việt**: kết quả fold nằm trong
//!    `data/vn_common.txt` thì không restore. Người gõ EN vẫn cứu được bằng
//!    Escape hoặc thêm từ vào `config.english_words`.
//!
//! 3. **`config.english_words[]`** (opt-in): danh sách từ EN của chủ gõ —
//!    luôn restore, kể cả khi fold trùng âm tiết Việt thông dụng (quyết định
//!    tường minh của người dùng thắng mọi phỏng đoán).
//!
//! Opt-out toàn bộ: `config.auto_restore_english = false`.
//!
//! ESC (`restore_last`) là đường restore thủ công (P0-3 §1.1
//! `hotkeys.restore_last`); Tab gợi ý hoàn tất từ EN dùng chung dữ liệu ở
//! `complete_word`.

use crate::post::predict::vn_prefix_possible;
use crate::transform::stroke::is_stroke;
use crate::transform::tone::is_vowel;
use crate::validate::is_valid_word;
use std::collections::HashSet;
use std::sync::OnceLock;

/// `data/en_common.txt` — nhúng lúc biên dịch (core **không** I/O: S1).
pub const EN_COMMON_DATA: &str = include_str!("../../../data/en_common.txt");
/// `data/assist/vn_ascii_syllables.txt` — âm tiết Việt KHÔNG dấu (deaccent
/// từ `vn_freq3000`, cùng file với L0 của `assist` — dùng chung một nguồn dữ
/// liệu duy nhất). Guard R2-4/R2-56 cho `complete_word` (vòng 37): chuỗi gõ
/// là âm tiết Việt không dấu thì KHÔNG quét freq/tech tìm từ EN — "cac" không
/// được hoàn tất thành "cache" ("các" thiếu dấu), "dat" → "date" ("đắt"…
/// thiếu dấu). File nhúng, core vẫn 0-I/O (S1).
const VN_ASCII_SYLLABLES_DATA: &str = include_str!("../../../data/assist/vn_ascii_syllables.txt");

/// Tập âm tiết Việt không dấu (build 1 lần lúc gọi đầu — OnceLock, 0 I/O).
fn vn_ascii_syllables() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| {
        VN_ASCII_SYLLABLES_DATA
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect()
    })
}
/// `data/vn_common.txt` — nhúng lúc biên dịch.
pub const VN_COMMON_DATA: &str = include_str!("../../../data/vn_common.txt");
/// `data/stop_en.txt` — danh sách mẫu cho `config.english_words` (opt-in).
pub const STOP_EN_DATA: &str = include_str!("../../../data/stop_en.txt");
/// `data/en_freq3000.txt` — 3000 từ EN tần suất cao nhất, nguồn cho **gợi ý
/// hoàn tất** (Tab/Space, vòng 24): danh sách `en_common` (~100 từ) chỉ phục
/// vụ chính sách restore (Fassist-96) nên quá ngắn cho completion — README
/// đã quảng cáo "3000 từ" từ 0.2.33, bản này mới thực sự nạp đủ nguồn.
pub const EN_FREQ_DATA: &str = include_str!("../../../data/en_freq3000.txt");

/// `data/en_words10k.txt` — google-10000-english (vòng 52): mở rộng vốn từ
/// **hoàn tất** lên ~10k. CHỈ prediction (prefix ghost + Tab), KHÔNG tham gia
/// `should_restore`/`is_known_en_word` — chính sách restore giữ nguyên để
/// corpus FPR 0 không đổi.
pub const EN_WORDS_10K_DATA: &str = include_str!("../../../data/en_words10k.txt");

/// OP-006 (P5): thuật ngữ lập trình + kinh doanh cấp **gợi ý hoàn tất từ**
/// (`try_english_complete`: "ebit" + Tab → "EBITDA"). Case chuẩn lưu dạng hiển
/// thị; so khớp lowercase nên người gõ "kpi" cũng khớp KPI.
///
/// LƯU Ý: danh sách này KHÔNG tham gia `should_restore` — heuristic hồi quy
/// corpus đã bảo vệ các từ này đủ (`rust`→"rút", `var`→"vả" là cặp mơ hồ
/// policy R2 giữ tiếng Việt; `target`/`margin`/`brief` fold ra âm Việt vô hiệu
/// được nhánh `is_valid_word` trả lại raw). Ghim vào test để không ai lật.
/// So khớp tuyến tính 1 lần/kết thúc từ — 63 mục không đáng kể.
pub const TECH_BUSINESS_TERMS: &[&str] = &[
    // Coder / Tech terms & keywords
    "async",
    "await",
    "cargo",
    "class",
    "const",
    "def",
    "docker",
    "export",
    "func",
    "graphql",
    "impl",
    "import",
    "interface",
    "javascript",
    "kubectl",
    "let",
    "nginx",
    "npm",
    "package",
    "printf",
    "private",
    "public",
    "python",
    "redis",
    "return",
    "rust",
    "struct",
    "typescript",
    "var",
    "yarn",
    // Identifier camelCase quen thuộc — hoàn tất qua tiền tố lowercase
    // ("uses" + Tab → "useState"): sau camel-repair từ ĐÃ active nên
    // thỏa gate Tab của `try_english_complete` (v24: Tab chỉ cứu từ
    // đã transform).
    "useState",
    "useEffect",
    "getUser",
    "isFetching",
    "onPress",
    "getElementById",
    "querySelector",
    "toString",
    "parseInt",
    // Business / Finance / Accounting terms (case chuẩn để completion)
    "ARPU",
    "B2B",
    "B2C",
    "CAC",
    "CPC",
    "CRM",
    "CTR",
    "EBITDA",
    "ERP",
    "GMV",
    "KPI",
    "LTV",
    "OKRs",
    "ROI",
    "SOP",
    "YoY",
    "brief",
    "budget",
    "contract",
    "feedback",
    "invoice",
    "margin",
    "pipeline",
    "pitch",
    "proposal",
    "quotation",
    "revenue",
    "target",
    // OS APIs & Systems Programming terms (Win32, POSIX/Linux, Cocoa/macOS)
    "CGRect",
    "CreateFile",
    "GetLastError",
    "HRESULT",
    "HINSTANCE",
    "HWND",
    "LPARAM",
    "LPVOID",
    "NSArray",
    "NSDictionary",
    "NSError",
    "NSString",
    "POSIX",
    "PostMessage",
    "RunLoop",
    "SendMessage",
    "VirtualAlloc",
    "WaitForSingleObject",
    "WPARAM",
    "cgroup",
    "dispatch_async",
    "epoll_create",
    "fcntl",
    "ioctls",
    "madvise",
    "mmap",
    "munmap",
    "pthread_create",
    "pthread_mutex_lock",
    "pwrite",
    "sigaction",
    "syscall",
    "sysctl",
    // Modern Gen Z & Internet terms
    "brainrot",
    "crashout",
    "cringe",
    "delulu",
    "highkey",
    "lore",
    "lowkey",
    "pookie",
    "rizz",
    "sigma",
    "skibidi",
    "slay",
    "unserious",
    "yapping",
];

/// Kiểm tra một chuỗi gõ có phải thuật ngữ kỹ thuật / kinh doanh không
/// (không phân biệt hoa/thường).
pub fn is_tech_business_term(word: &str) -> bool {
    let lower = word.to_lowercase();
    TECH_BUSINESS_TERMS
        .iter()
        .any(|t| t.to_lowercase() == lower)
}

/// Chuỗi gõ có nằm trong danh sách `english_words` của người dùng không?
/// (không phân biệt hoa thường)
fn listed(raw: &[char], english_words: &[String]) -> bool {
    if english_words.is_empty() {
        return false;
    }
    let typed: String = raw.iter().collect::<String>().to_lowercase();
    english_words.iter().any(|w| w.to_lowercase() == typed)
}

// Vòng 41 (Pass B PB-1): 3 nguồn tra cứu parse **MỘT LẦN** qua OnceLock —
// `data_contains` quét `lines()` mỗi gọi (is_known_en_word = 80µs/call trên
// đường peek **mỗi phím**). HashSet đã-lowercase ⇒ contains O(1), ngữ nghĩa
// giữ nguyên (so lowercase, bỏ `#`/dòng rỗng).
fn static_lines_lower(
    data: &'static str,
    cache: &'static OnceLock<Vec<&'static str>>,
) -> &'static [&'static str] {
    cache.get_or_init(|| {
        data.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect()
    })
}

fn static_set_lower(
    data: &'static str,
    cache: &'static OnceLock<HashSet<String>>,
) -> &'static HashSet<String> {
    cache.get_or_init(|| {
        data.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| l.to_lowercase())
            .collect()
    })
}

fn en_common_lines() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    static_lines_lower(EN_COMMON_DATA, &CACHE)
}

fn en_common_set() -> &'static HashSet<String> {
    static CACHE: OnceLock<HashSet<String>> = OnceLock::new();
    static_set_lower(EN_COMMON_DATA, &CACHE)
}

fn vn_common_set() -> &'static HashSet<String> {
    static CACHE: OnceLock<HashSet<String>> = OnceLock::new();
    static_set_lower(VN_COMMON_DATA, &CACHE)
}

fn en_freq_lines() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    static_lines_lower(EN_FREQ_DATA, &CACHE)
}

/// Từ có trong danh sách 10k từ tiếng Anh không (tìm nhị phân trên bản đã sắp).
pub(crate) fn en_words10k_contains(word: &str) -> bool {
    en_words10k_sorted().binary_search(&word).is_ok()
}

/// Bản đã sắp của `en_common`/`en_freq3000`: `complete_word_candidates` chạy mỗi phím
/// qua `peek_completion` nên tìm theo tiền tố thay vì quét cả danh sách (R2-150).
fn en_common_sorted() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| sorted_copy(en_common_lines()))
}

fn en_freq_sorted() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| sorted_copy(en_freq_lines()))
}

fn sorted_copy(lines: &[&'static str]) -> Vec<&'static str> {
    let mut v = lines.to_vec();
    v.sort_unstable();
    v
}

/// Các mục bắt đầu bằng `prefix` trong danh sách đã sắp.
fn with_prefix<'a>(
    sorted: &'a [&'static str],
    prefix: &'a str,
) -> impl Iterator<Item = &'static str> + 'a {
    let start = sorted.partition_point(|w| *w < prefix);
    sorted[start..]
        .iter()
        .copied()
        .take_while(move |w| w.starts_with(prefix))
}

/// google-10000-english đã sắp alphabet (vòng 52) — binary search trong
/// `complete_word`, chỉ chạy khi qua gate âm-tiết-Việt.
fn en_words10k_sorted() -> &'static [&'static str] {
    static CACHE: OnceLock<Vec<&'static str>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut v: Vec<&'static str> = EN_WORDS_10K_DATA
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        v.sort_unstable();
        v
    })
}

fn en_freq_set() -> &'static HashSet<String> {
    static CACHE: OnceLock<HashSet<String>> = OnceLock::new();
    static_set_lower(EN_FREQ_DATA, &CACHE)
}

/// Từ tiếng Anh thông dụng (nhánh 2)? — O(1) qua HashSet (PB-1).
pub fn en_common_contains(word: &str) -> bool {
    let needle = word.to_lowercase();
    en_common_set().contains(needle.as_str())
}

/// Âm tiết Việt thông dụng được bảo vệ (đích fold phải tránh)?
pub fn vn_common_contains(syllable: &str) -> bool {
    let needle = syllable.to_lowercase();
    vn_common_set().contains(needle.as_str())
}

/// Từ có nằm trong nguồn EN **hoàn tất** (en_common + 3000 từ tần suất) không?
/// Guard Space-completion (vòng 24): chuỗi gõ đã là từ EN đúng → không kéo dài.
pub fn is_known_en_word(word: &str) -> bool {
    let needle = word.to_lowercase();
    en_common_set().contains(needle.as_str()) || en_freq_set().contains(needle.as_str())
}

/// Từ tiếng Anh viết đôi phụ âm (`ss`/`ff`/`rr`), họ Bắc Âu `-sson`, và từ đuôi `z` thiếu trong
/// `en_words10k.txt` — giữ nguyên dạng gốc khi người dùng gõ thẳng (`dessert`,
/// `marry`, `scarred`, `busses`, `buff`, `buffing`, `newsstand`, `wolff`, `andersson`, `quartz`…).
/// Không đưa `"hiss"`, `"buss"`, `"lass"`, `"lasser"` vào bảng này để giữ hoàn tác UniKey (`his`/`bus`/`las`/`laser`).
const EN_EXTRA_LEXICON_WORDS: &[&str] = &[
    "alvarez",
    "andersson",
    "baroness",
    "barrage",
    "barred",
    "barring",
    "biassed",
    "biasses",
    "biassing",
    "bliss",
    "bluff",
    "bluffed",
    "blurred",
    "blurry",
    "borrow",
    "buff",
    "buffed",
    "buffing",
    "buffs",
    "burr",
    "bussed",
    "busses",
    "bussing",
    "carcass",
    "caress",
    "carlsson",
    "carrot",
    "charred",
    "charring",
    "compass",
    "compress",
    "conferred",
    "conferrer",
    "conferring",
    "cortez",
    "countess",
    "cuffed",
    "cutlass",
    "dandruff",
    "deferred",
    "deferring",
    "dessert",
    "desserts",
    "deterred",
    "deterring",
    "digress",
    "diss",
    "dissed",
    "disses",
    "distress",
    "dorff",
    "doss",
    "dossed",
    "dossing",
    "eriksson",
    "ersatz",
    "fernandez",
    "floss",
    "fluff",
    "flurry",
    "focussed",
    "focusses",
    "focussing",
    "forrester",
    "fortress",
    "fritz",
    "furr",
    "furry",
    "garry",
    "gassed",
    "gasses",
    "gassing",
    "gauss",
    "gigahertz",
    "gloss",
    "goddess",
    "gruff",
    "gustafsson",
    "handcuff",
    "hansson",
    "hausdorff",
    "hernandez",
    "hertz",
    "hissed",
    "hisses",
    "hissing",
    "horowitz",
    "hostess",
    "hurry",
    "inferred",
    "inferring",
    "ingress",
    "jarred",
    "jarring",
    "johansson",
    "jonsson",
    "juarez",
    "kilohertz",
    "kirchhoff",
    "klutz",
    "kurtz",
    "larsson",
    "lassed",
    "lasses",
    "lioness",
    "lorenz",
    "marred",
    "married",
    "marries",
    "marring",
    "marry",
    "marrying",
    "martinez",
    "massacre",
    "massaged",
    "massages",
    "mastiff",
    "megahertz",
    "newsstand",
    "newsstands",
    "nilsson",
    "occurred",
    "occurrer",
    "olsson",
    "orff",
    "ortiz",
    "overpass",
    "parred",
    "parring",
    "parrot",
    "passable",
    "passbook",
    "passcode",
    "passkey",
    "passover",
    "passthrough",
    "perez",
    "perror",
    "persson",
    "pettersson",
    "plaintiff",
    "poetess",
    "poisson",
    "preferred",
    "preferrer",
    "preferring",
    "priestess",
    "protectress",
    "puffed",
    "purr",
    "quartz",
    "ramirez",
    "recess",
    "recurred",
    "redress",
    "referred",
    "referrer",
    "regress",
    "repress",
    "rodriguez",
    "scarred",
    "scarring",
    "schwartz",
    "schwarz",
    "scoff",
    "scuffed",
    "scurry",
    "serverless",
    "sheriff",
    "sniffed",
    "sorrow",
    "sparred",
    "sparring",
    "sparrow",
    "sparrows",
    "spitz",
    "starred",
    "starry",
    "stderr",
    "stiff",
    "stuffed",
    "suarez",
    "surpass",
    "svensson",
    "tariff",
    "tarred",
    "tarring",
    "transferred",
    "transferrer",
    "transferring",
    "transship",
    "transsonic",
    "underpass",
    "waitress",
    "warred",
    "warring",
    "weierstrass",
    "whirr",
    "windlass",
    "wolff",
];

/// Thuật ngữ lập trình / CLI / khoa học / toán học hay gặp không có trong `en_words10k.txt` —
/// phục vụ tách phím hoàn tác UniKey (`serrde`→`serde`, `varrss`→`vars`, `SerrdeJson`→`SerdeJson`,
/// `Bayess`→`Bayes`, `Eulerr`→`Euler`, `Tensorr`→`Tensor`, `xorr`→`xor`, `Sirri`→`Siri`).
const TECH_UNDO_WORDS: &[&str] = &[
    "arccos",
    "arcsin",
    "arctan",
    "args",
    "async",
    "axios",
    "bayes",
    "bibtex",
    "cantor",
    "centos",
    "cfg",
    "configs",
    "cors",
    "cos",
    "cosh",
    "csrf",
    "cur",
    "curl",
    "defaults",
    "defs",
    "deref",
    "descartes",
    "devops",
    "dir",
    "docker",
    "doppler",
    "eof",
    "errno",
    "eslint",
    "euler",
    "formatter",
    "fourier",
    "frobenius",
    "galois",
    "gif",
    "github",
    "gitlab",
    "idx",
    "impl",
    "inf",
    "ios",
    "kafka",
    "kepler",
    "ker",
    "kronecker",
    "latex",
    "leftmost",
    "lexer",
    "lidar",
    "linter",
    "linus",
    "linux",
    "macos",
    "mgr",
    "molar",
    "mongo",
    "mutex",
    "mysql",
    "navier",
    "nginx",
    "noether",
    "nor",
    "oauth",
    "overleaf",
    "parse",
    "parsed",
    "parser",
    "parsers",
    "parsing",
    "pasteur",
    "planar",
    "poetry",
    "polar",
    "pos",
    "posix",
    "postgres",
    "prettier",
    "prisma",
    "ptr",
    "pulsar",
    "pytorch",
    "quarto",
    "radar",
    "redis",
    "ref",
    "refs",
    "regex",
    "res",
    "rustc",
    "rustfmt",
    "scalar",
    "schrodinger",
    "semver",
    "serde",
    "serverless",
    "shader",
    "sir",
    "siri",
    "sirs",
    "solver",
    "sonar",
    "spoof",
    "src",
    "stderr",
    "stdin",
    "stdout",
    "stokes",
    "svelte",
    "sylvester",
    "tar",
    "tensor",
    "terraform",
    "triton",
    "typst",
    "unref",
    "usr",
    "val",
    "vars",
    "vscode",
    "wasm",
    "wolfram",
    "xor",
];

fn in_base_lexicon_lower(needle: &str, orig: &str, english_words: &[String]) -> bool {
    let uniform_chars = needle
        .chars()
        .next()
        .is_some_and(|f| needle.chars().all(|c| c == f));
    en_common_set().contains(needle)
        || en_freq_set().contains(needle)
        || (!uniform_chars && en_words10k_contains(needle))
        || EN_EXTRA_LEXICON_WORDS.binary_search(&needle).is_ok()
        || is_tech_business_term(needle)
        || english_words.iter().any(|w| w.eq_ignore_ascii_case(orig))
}

fn is_doubled_consonant_inflection(lower: &str, orig: &str, english_words: &[String]) -> bool {
    const SUFFIXES: &[(&str, usize)] = &[
        ("rred", 2),
        ("rring", 3),
        ("ssed", 2),
        ("ssing", 3),
        ("sses", 2),
        ("ffed", 2),
        ("ffing", 3),
    ];
    for &(suf, dbl_strip) in SUFFIXES {
        if lower.ends_with(suf) && lower.len() >= suf.len() + 2 {
            let base = &lower[..lower.len() - dbl_strip];
            if base == "bass" && suf != "sses" {
                continue;
            }
            if base.len() >= 4 && in_base_lexicon_lower(base, orig, english_words) {
                return true;
            }
        }
    }
    false
}

/// Từ có nằm trong bất kỳ nguồn từ vựng EN đã biết nào (common, freq3000, words10k, từ đôi
/// ngoại lệ, biến thể chia thì giữ phụ âm đôi, thuật ngữ tech hoặc danh sách `english_words`
/// của người dùng) hay không (E50/E61).
pub(crate) fn is_english_lexicon_word(word: &str, english_words: &[String]) -> bool {
    if word.is_empty() {
        return false;
    }
    let needle = word.to_lowercase();
    in_base_lexicon_lower(&needle, word, english_words)
        || is_doubled_consonant_inflection(&needle, word, english_words)
}

fn in_undo_exact_lexicon(lower: &str, orig: &str, english_words: &[String]) -> bool {
    if lower.len() < 3 || matches!(lower, "bufing" | "los" | "les") {
        return false;
    }
    in_base_lexicon_lower(lower, orig, english_words)
        || TECH_UNDO_WORDS.binary_search(&lower).is_ok()
}

/// Từ nguồn sạch (>= 4 ký tự nếu từ `en_words10k`, hoặc nằm trong `en_common`/`en_freq3000`/
/// `TECH_BUSINESS_TERMS`/`EN_EXTRA_LEXICON_WORDS`/`TECH_UNDO_WORDS` trừ `"res"`, `"las"`) để làm gốc
/// chia từ / ghép 2 phần — loại bỏ nhiễu 3 chữ cái (`pas`, `mas`, `des`, `std`, `las`…).
fn in_undo_part_lexicon(lower: &str, orig: &str, english_words: &[String]) -> bool {
    if lower.len() < 3 || matches!(lower, "bufing" | "los" | "les" | "res" | "las") {
        return false;
    }
    let uniform_chars = lower
        .chars()
        .next()
        .is_some_and(|f| lower.chars().all(|c| c == f));
    en_common_set().contains(lower)
        || en_freq_set().contains(lower)
        || (lower.len() >= 4 && !uniform_chars && en_words10k_contains(lower))
        || EN_EXTRA_LEXICON_WORDS.binary_search(&lower).is_ok()
        || is_tech_business_term(lower)
        || TECH_UNDO_WORDS.binary_search(&lower).is_ok()
        || english_words.iter().any(|w| w.eq_ignore_ascii_case(orig))
}

fn is_cvc_single_sfr(stem: &str) -> bool {
    let b = stem.as_bytes();
    if b.len() < 3 {
        return false;
    }
    // Âm cuối `-er` không mang trọng âm (`buffer`, `filter`, `cater`, `offer`; trừ các gốc
    // trọng âm `-fer`/`deter`) KHÔNG gấp đôi phụ âm cuối trong tiếng Anh → giữ được UniKey undo `bufferring ` → `buffering `.
    if stem.ends_with("er")
        && b.len() >= 5
        && !matches!(
            stem,
            "refer" | "prefer" | "transfer" | "confer" | "defer" | "infer" | "deter"
        )
    {
        return false;
    }
    let is_v = |ch: u8| matches!(ch, b'a' | b'e' | b'i' | b'o' | b'u');
    !is_v(b[b.len() - 3]) && is_v(b[b.len() - 2]) && matches!(b[b.len() - 1], b'r' | b's' | b'f')
}

/// Kiểm tra từ sau khi lọc phím hoàn tác `s`/`f`/`r` có phải từ tiếng Anh thật hay không (E60/E61):
/// với độ dài 2 chỉ nhận các từ 2 chữ cái thật (`as`, `if`, `is`, `of`, `or`, `os`, `us`) để tránh
/// mã viết tắt nhiễu trong google-10k (`ar`, `er`…) làm hỏng định danh code (`arr[0]`, `err`).
/// Hỗ trợ thêm thuật ngữ tech, hậu tố chia từ và ghép 2 từ thường (`var`+`name`, `task`+`mgr`).
pub(crate) fn is_real_english_word_for_undo(word: &str, english_words: &[String]) -> bool {
    if word.is_empty() {
        return false;
    }
    if english_words.iter().any(|w| w.eq_ignore_ascii_case(word)) {
        return true;
    }
    let needle = word.to_lowercase();
    if matches!(needle.as_str(), "bufing" | "los" | "les") {
        return false;
    }
    if needle.len() == 2 {
        return matches!(
            needle.as_str(),
            "as" | "if" | "is" | "of" | "or" | "os" | "us"
        );
    }
    if in_undo_exact_lexicon(&needle, word, english_words)
        || is_doubled_consonant_inflection(&needle, word, english_words)
    {
        return true;
    }
    // Hậu tố chia từ cơ bản trên gốc sạch (`deffaults` -> `defaults`, `serrverless` -> `serverless`).
    if needle.len() >= 4 && needle.is_ascii() {
        for suf in ["s", "es", "ed", "ing", "er", "less", "most"] {
            if let Some(stem) = needle.strip_suffix(suf) {
                if stem.len() < 3 || !in_undo_part_lexicon(stem, stem, english_words) {
                    continue;
                }
                if suf == "s" && stem.ends_with('s') {
                    continue;
                }
                if suf == "es"
                    && !(stem.ends_with('s')
                        || stem.ends_with('x')
                        || stem.ends_with('z')
                        || stem.ends_with("ch")
                        || stem.ends_with("sh")
                        || stem.ends_with('o'))
                {
                    continue;
                }
                if matches!(suf, "es" | "ed" | "ing" | "er") && is_cvc_single_sfr(stem) {
                    continue;
                }
                return true;
            }
        }
    }
    // Định danh ghép 2 từ thường (`var`+`name`, `sir`+`name`, `task`+`mgr`, `def`+`val`…).
    if needle.len() >= 6 && needle.is_ascii() {
        for split in 3..=needle.len() - 3 {
            let (left, right) = needle.split_at(split);
            if in_undo_part_lexicon(left, left, english_words)
                && in_undo_part_lexicon(right, right, english_words)
            {
                return true;
            }
        }
    }
    false
}

/// Có nên restore chuỗi gõ thay vì giữ kết quả transform?
///
/// `true` khi: kết quả fold **không** là âm tiết Việt hợp lệ (nhánh 1); hoặc
/// chuỗi gõ là từ EN thông dụng mà fold **không** đụng âm tiết Việt thông
/// dụng (nhánh 2); hoặc nằm trong `english_words` của người dùng (nhánh 3).
/// `false` khi `raw` rỗng / không biến đổi.
pub fn should_restore(raw: &[char], display: &[char], english_words: &[String]) -> bool {
    if raw.is_empty() || raw == display {
        return false;
    }
    // Bấm ngoặc hai lần (`m[[` → `m[`) là thao tác chủ đích để lấy dấu ngoặc literal:
    // raw còn phím đôi nên không được trả về — dù sau đó có gõ thêm chữ.
    if raw.windows(2).any(|p| p == ['[', '['] || p == [']', ']']) {
        return false;
    }
    if listed(raw, english_words) {
        return true;
    }
    // Token không có nguyên âm mà vẫn bị biến đổi thì biến đổi đó là `đ` (dấu thanh/mũ/
    // sừng đều cần nguyên âm): `50.000đ`, `ĐT`, `ĐHQG`, `đc`, `đ/c` — chữ viết tắt và đơn
    // vị tiền rất thường gặp, tiếng Anh không có từ nào như vậy (R2-58; UniKey `processDd`
    // cũng cố ý cho `dd` trong chuỗi không phải tiếng Việt).
    if !display.iter().any(|&c| is_vowel(c)) && display.iter().any(|&c| is_stroke(c)) {
        return false;
    }
    if !is_valid_word(display) {
        return true;
    }
    let typed: String = raw.iter().collect::<String>().to_lowercase();
    let folded: String = display.iter().collect::<String>().to_lowercase();
    en_common_contains(&typed) && !vn_common_contains(&folded)
}

/// Parse một data word-list (mỗi dòng 1 từ; `#` và dòng trống là chú thích).
pub fn parse_word_list(src: &str) -> Vec<String> {
    src.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_lowercase())
        .collect()
}

/// Danh sách mẫu đi kèm bản build cho `config.english_words` (rỗng nếu không có data).
pub fn default_english_words() -> Vec<String> {
    parse_word_list(STOP_EN_DATA)
}

/// **Tab gợi ý hoàn tất từ EN**: `typed` (lowercase) là tiền tố nghiêm ngặt của
/// đúng một từ EN thông dụng (ngắn nhất, rồi theo thứ tự bảng chữ) và fold
/// hiện tại không phải âm tiết Việt thông dụng → trả về từ hoàn chỉnh.
///
/// Chỉ chạy khi từ đã bị transform (word active) — từ chưa biến đổi nào là
/// tiếng Anh thuần, Tab phải đi qua cho app (indent, macro kế…).
pub fn complete_word(
    typed_lower: &str,
    folded_lower: &str,
    english_words: &[String],
) -> Option<String> {
    if typed_lower.len() < 2 || vn_common_contains(folded_lower) {
        return None;
    }
    let mut best: Option<String> = None;
    let consider = |best: &mut Option<String>, cand: &str| {
        if !cand.starts_with(typed_lower) || cand == typed_lower {
            return;
        }
        let better = match best {
            Some(b) => (cand.len(), cand) < (b.len(), b.as_str()),
            None => true,
        };
        if better {
            *best = Some(cand.to_owned());
        }
    };
    for &l in en_common_lines() {
        consider(&mut best, l);
    }
    // Nguồn 3000 từ tần suất (vòng 24) — completion cần vốn từ lớn, nhưng chỉ
    // quét khi `folded_lower` CHƯA phải âm tiết tiếng Việt hợp lệ (bảo vệ
    // "em", "con", "cho", "cá", "nó", "chó", "nói", "bỏ", "tủ", "vẻ"… khỏi bị
    // biến thành "email", "confirm", "choice", "case", "nose", "choose"…).
    let folded_chars: Vec<char> = folded_lower.chars().collect();
    // Nếu người dùng gõ một từ tiếng Anh ĐÃ ĐẦY ĐỦ (`is_known_en_word`) nhưng ở Mode VI
    // bị Telex biến đổi thành một âm tiết Việt KHÔNG hợp lệ (ví dụ `sort` -> `sỏt`, `smart` -> `smảt`,
    // `file` -> `fỉle`, `user` -> `úer`), thì Tab phải trả về chính từ tiếng Anh gốc đó (`sort`, `smart`...)
    // thay vì tự ý nối thêm hậu tố số nhiều (`sorts`, `smarts`, `files`, `users`).
    if folded_lower != typed_lower && !is_valid_word(&folded_chars) && is_known_en_word(typed_lower)
    {
        return Some(
            match TECH_BUSINESS_TERMS
                .iter()
                .find(|t| t.eq_ignore_ascii_case(typed_lower))
            {
                Some(t) => (*t).to_string(),
                None => typed_lower.to_string(),
            },
        );
    }
    // R2-4/R2-56 (vòng 37): chuỗi gõ là âm tiết Việt KHÔNG dấu và không phải
    // từ EN đã biết → BỎ QUA quét freq/tech. Đóng chốt cả họ lỗi
    // tiền-tố-VN-thiếu-dấu + từ-domain ("cac"→"cache" (các), "dat"→"date"
    // (đắt/đất), "suc"→"such" (sức)…) — trước đây chỉ được ghim lẻ "cac" ở
    // test R2-4 vì freq-list chưa có tiền tố trùng. Quét `en_common` phía trên
    // không đổi: danh sách đó đã lọc theo policy `en_common.txt`.
    let vn_only_typed =
        vn_ascii_syllables().contains(typed_lower) && !is_known_en_word(typed_lower);
    if !is_valid_word(&folded_chars) && !vn_only_typed {
        for &l in en_freq_lines() {
            consider(&mut best, l);
        }
        // Vòng 52: google-10000-english — vốn từ hoàn tất ~10k, binary search
        // (chỉ scan vùng prefix). Cùng gate âm-tiết-Việt như freq-scan trên.
        {
            let sorted = en_words10k_sorted();
            let start = sorted.partition_point(|w| *w < typed_lower);
            for &l in &sorted[start..] {
                if !l.starts_with(typed_lower) {
                    break;
                }
                consider(&mut best, l);
            }
        }
        // OP-006 (P5): thuật ngữ tech/business — "ebit" + Tab → "EBITDA",
        // cùng guard âm-tiết-Việt như freq-scan ("cac" có thể là "các" thiếu
        // dấu → không cướp). `consider` so prefix case-sensitive nên phải
        // hạ lowercase trước; case chuẩn được khôi ở bước trả về cuối.
        for t in TECH_BUSINESS_TERMS {
            let tl = t.to_lowercase();
            if tl.starts_with(typed_lower) {
                consider(&mut best, &tl);
            }
        }
    }
    for w in english_words {
        // Từ điển cá nhân so khớp không phân biệt hoa thường (như `listed`): người
        // dùng thêm `VnExpress` thì `vn` + Tab vẫn phải gợi ý.
        let w = w.trim().to_lowercase();
        if !w.is_empty() {
            consider(&mut best, &w);
        }
    }
    // OP-006: candidate từ TECH_BUSINESS_TERMS trả về **dạng chuẩn** giữ case
    // ("EBITDA") — còn lại lowercase như cũ.
    best.map(|b| {
        match TECH_BUSINESS_TERMS
            .iter()
            .find(|t| t.to_lowercase() == b.to_lowercase())
        {
            Some(t) => t.to_string(),
            None => b.to_lowercase(),
        }
    })
}

/// Kiểm tra ngữ cảnh trước từ đang gõ có phải ngữ cảnh tiếng Anh không:
/// từ liền trước là từ tiếng Anh đã biết (`is_known_en_word`) và không phải âm tiết Việt.
pub fn has_en_context(sentence: &str) -> bool {
    if let Some(prev_word) = sentence.split_whitespace().last() {
        let prev_clean: String = prev_word
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect::<String>()
            .to_lowercase();
        if prev_clean.len() >= 2
            && prev_clean.chars().all(|c| c.is_ascii_alphabetic())
            && is_known_en_word(&prev_clean)
            && !vn_common_contains(&prev_clean)
            && !vn_ascii_syllables().contains(prev_clean.as_str())
        {
            return true;
        }
    }
    false
}

/// Hoàn tất đa ứng viên tiếng Anh (lên tới `limit` ứng viên cho Tab cycling & Ghost Text),
/// hoạt động trên cả **Mode EN (`!vi_on`)** và **Mode VI (`vi_on`)**:
/// - Tier 0: Từ cá nhân học được (`style_words` + `english_words`) xếp theo frecency.
/// - Tier 1: Từ ghép / cụm danh từ tiếng Anh (`en_compounds.txt`) có từ đầu khớp trọn vẹn
///   với `typed_lower` (`ext == true`, ví dụ `"machine"` → `"machine learning"`, `"customer"` → `"customer service"`).
/// - Tier 2: Từ đơn tiếng Anh (`en_common`, `en_freq3000`, `en_words10k`, `TECH_BUSINESS_TERMS`),
///   xếp theo `(len, -weight, lex)`: từ ngắn trước như `complete_word`, nhưng trong cùng độ
///   dài từ thông dụng hơn đứng trước (`ph` → `php`, không phải `phd`); ứng viên 2..5 cho Tab liên tiếp.
/// - Tier 3: Từ ghép tiếng Anh có từ đầu bắt đầu bằng tiền tố `typed_lower` (`ext == false`).
#[allow(clippy::too_many_arguments)]
pub fn complete_word_candidates(
    typed_lower: &str,
    folded_lower: &str,
    english_words: &[String],
    style_words: &[(String, u32)],
    style_touches: usize,
    vi_on: bool,
    space_mode: bool,
    en_context: bool,
    limit: usize,
) -> Vec<String> {
    if typed_lower.len() < 2 || limit == 0 {
        return Vec::new();
    }
    // Ở mode VI, bảo vệ tuyệt đối âm tiết tiếng Việt thông dụng (R2-56/R2-57).
    // Ở mode EN (!vi_on), người dùng đã chủ động tắt bộ gõ Việt để gõ tiếng Anh → không chặn!
    if vi_on && vn_common_contains(folded_lower) {
        return Vec::new();
    }

    let folded_chars: Vec<char> = folded_lower.chars().collect();
    let vn_only_typed = vi_on
        && vn_ascii_syllables().contains(typed_lower)
        && !is_known_en_word(typed_lower)
        && !en_context;
    // RSI-7 — cổng ngôn ngữ (đường EN): tiền tố còn lớn lên được thành âm tiết
    // Việt (so dạng ĐÃ BỎ DẤU — "gu"→"gụ", "ho"→"họ" mà bảng 1061 âm tiết ASCII
    // bỏ sót) và không có ngữ cảnh EN → chặn hoàn-tiện-prefix nguồn EN tĩnh.
    // Nhường Tier 0 (từ cá nhân) và nhánh RESTORE từ EN gõ trọn bị Telex fold
    // (`sort`→`sỏt` trả lại `sort`) — đó không phải hoàn-tiện-prefix.
    let vn_only_prefix = vi_on && !en_context && vn_prefix_possible(typed_lower);
    let allow_full_en = if !vi_on {
        true
    } else {
        (!is_valid_word(&folded_chars) && !vn_only_typed && !vn_only_prefix)
            || (en_context && folded_lower == typed_lower && !space_mode)
    };

    // (tier, primary_ord, neg_weight, canon_word)
    let mut cands: Vec<(u8, usize, i32, String)> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    let canonicalize = |w: &str| -> String {
        match TECH_BUSINESS_TERMS
            .iter()
            .find(|t| t.eq_ignore_ascii_case(w))
        {
            Some(t) => (*t).to_string(),
            None => w.to_lowercase(),
        }
    };

    // Từ tiếng Anh ĐÃ gõ trọn bị Telex fold thành chuỗi KHÔNG phải tiếng Việt (`sort` → `sỏt`,
    // `user` → `úer`): Tab đầu trả đúng từ gốc, TRƯỚC mọi tier — kể cả từ cá nhân và từ ghép.
    // Dài hơn (`sorts`, `user interface`) nằm ở các Tab sau. Fold ra âm tiết hợp lệ (`hats` →
    // `hát`) là chữ Việt người dùng đang gõ, không đụng.
    if vi_on
        && !space_mode
        && folded_lower != typed_lower
        && !is_valid_word(&folded_chars)
        && (is_known_en_word(typed_lower) || en_words10k_contains(typed_lower))
        && seen.insert(typed_lower.to_string())
    {
        cands.push((0, 0, i32::MIN, typed_lower.to_string()));
    }

    // Tier 0: Từ cá nhân học được (Dream RSI style_words + english_words).
    // F-P10-1 (vòng 59): `style_words` giữ thứ tự **dùng gần nhất** (mới nhất
    // cuối — engine recency touch mỗi lần gõ lại) → cộng recency boost vào
    // weight sort (12 mục cuối +12, 24 mục cuối +6) — từ vừa gõ lại nổi trên
    // từ count cao nhưng lâu không dùng (EMA-recency, không thêm trường giờ — G8).
    // F-A1-2 (audit vòng 59): chỉ boost trong phạm vi `style_touches` entry cuối
    // — sau ConfigReload thứ tự là count-desc (sanitize sort), boost đống
    // count-thấp nhất = sai; đếm về 0 lúc nạp options ⇒ boost = 0 tới touch mới.
    let n_style = style_words.len();
    for (idx, (sw, count)) in style_words.iter().enumerate() {
        let from_end = n_style - 1 - idx;
        let recency: i32 = if from_end >= style_touches {
            0
        } else if from_end < 12 {
            12
        } else if from_end < 24 {
            6
        } else {
            0
        };
        let w = sw.trim().to_lowercase();
        if w.len() > typed_lower.len()
            && w.starts_with(typed_lower)
            && w.chars().all(|c| c.is_ascii_lowercase() || c == ' ')
            && (!space_mode || !w.contains(' '))
            && seen.insert(w.clone())
        {
            cands.push((0, w.len(), -((*count).min(1000) as i32) - recency, w));
        }
    }
    for ew in english_words {
        let w = ew.trim().to_lowercase();
        if w.len() > typed_lower.len() && w.starts_with(typed_lower) && seen.insert(w.clone()) {
            cands.push((0, w.len(), -1, w));
        }
    }

    // Tier 1 & Tier 3: Từ ghép tiếng Anh (`en_compounds.txt`) khi bấm Tab (!space_mode) và allow_full_en
    if allow_full_en && !space_mode {
        if let Some(&b0) = typed_lower.as_bytes().first() {
            for &(comp, weight) in crate::post::predict::en_compounds_buckets()
                .get(&b0)
                .copied()
                .into_iter()
                .flatten()
            {
                if comp.len() > typed_lower.len() && comp.starts_with(typed_lower) {
                    let is_ext = comp.split(' ').next() == Some(typed_lower);
                    let tier = if is_ext { 1 } else { 3 };
                    if seen.insert(comp.to_string()) {
                        cands.push((tier, comp.len(), -(weight as i32), comp.to_string()));
                    }
                }
            }
        }
    }

    // Tier 2: Từ đơn tiếng Anh (giữ nguyên thứ tự ngắn nhất trước của `complete_word`).
    // RSI-7: en_common cũng thuộc cổng ngôn ngữ — chính danh sách ~153 từ restore
    // này sinh phần lớn nhiễu VI ("guys", "horse", "bars", "law", "town" đều ở đây).
    if !vn_only_prefix {
        for l in with_prefix(en_common_sorted(), typed_lower) {
            if l.len() > typed_lower.len() && seen.insert(l.to_string()) {
                cands.push((2, l.len(), -70, l.to_string()));
            }
        }
    }
    if allow_full_en {
        for l in with_prefix(en_freq_sorted(), typed_lower) {
            if l.len() > typed_lower.len() && seen.insert(l.to_string()) {
                cands.push((2, l.len(), -60, l.to_string()));
            }
        }
        for l in with_prefix(en_words10k_sorted(), typed_lower) {
            if l.len() > typed_lower.len() && seen.insert(l.to_string()) {
                cands.push((2, l.len(), -55, l.to_string()));
            }
        }
        for t in TECH_BUSINESS_TERMS {
            let starts = t
                .as_bytes()
                .get(..typed_lower.len())
                .is_some_and(|head| head.eq_ignore_ascii_case(typed_lower.as_bytes()));
            if starts && t.len() > typed_lower.len() && seen.insert(t.to_lowercase()) {
                cands.push((2, t.len(), -85, (*t).to_string()));
            }
        }
    }

    let lex = |a: &str, b: &str| {
        a.bytes()
            .map(|c| c.to_ascii_lowercase())
            .cmp(b.bytes().map(|c| c.to_ascii_lowercase()))
    };
    cands.sort_by(|a, b| {
        a.0.cmp(&b.0).then_with(|| match a.0 {
            // Tier 0 (personal) & Tier 1/3 (compounds): frecency / weight cao hơn trước, rồi ngắn hơn trước
            0 | 1 | 3 => {
                a.2.cmp(&b.2)
                    .then_with(|| a.1.cmp(&b.1))
                    .then_with(|| lex(&a.3, &b.3))
            }
            // Tier 2 (từ đơn): ngắn hơn trước, rồi weight
            _ => {
                a.1.cmp(&b.1)
                    .then_with(|| a.2.cmp(&b.2))
                    .then_with(|| lex(&a.3, &b.3))
            }
        })
    });

    // Tra dạng viết chuẩn chỉ cho vài ứng viên giữ lại, không cho cả tập khớp tiền tố.
    cands
        .into_iter()
        .take(limit)
        .map(|c| canonicalize(&c.3))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method::{fold, Method};
    use crate::transform::DiacriticStyle;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    /// Fold kiểu engine (Telex, style mới, free marking) — trùng đường chạy thật.
    fn telex(raw: &str) -> Vec<char> {
        fold(&chars(raw), Method::Telex, DiacriticStyle::New, true)
    }

    #[test]
    fn restores_invalid_fold() {
        // "asdf" → fold "àd" (âm cuối 'd' không hợp lệ) → restore
        assert!(should_restore(&chars("asdf"), &chars("àd"), &[]));
        // "expect" → fold "ẽpect" (vần/âm cuối không hợp lệ) → restore
        assert!(should_restore(&chars("expect"), &chars("ẽpect"), &[]));
    }

    #[test]
    fn keeps_valid_vietnamese() {
        assert!(!should_restore(&chars("duocj"), &chars("được"), &[]));
        assert!(!should_restore(&chars("hoaf"), &chars("hoà"), &[]));
    }

    /// OP-006/R2-4: thuật ngữ tech KPÍNH ra raw chính xác nhưng fold là âm tiết
    /// Việt thông dụng/ hợp lệ → theo policy corpus cặp mơ hồ GIỮ tiếng Việt;
    /// danh sách `TECH_BUSINESS_TERMS` chỉ cấp completion, không đè restore.
    #[test]
    fn op006_keep_vn_beats_tech_terms() {
        // "rust" → "rút" (âm Việt hợp lệ, không phổ biến) vẫn KHÔNG restore.
        assert!(!should_restore(&chars("rust"), &chars("rút"), &[]));
        // "var" → "vả" (âm Việt thông dụng) giữ tiếng Việt.
        assert!(!should_restore(&chars("var"), &chars("vả"), &[]));
        // Thuật ngữ fold ra âm Việt KHÔNG hợp lệ → restore đúng.
        assert!(should_restore(&chars("target"), &chars("tảget"), &[]));
        assert!(should_restore(&chars("margin"), &chars("mảgin"), &[]));
    }

    /// OP-006: Tab completion tech/business giữ case chuẩn; âm Việt hợp lệ
    /// ("cac" cơ bản) chặn toàn bộ quét mở rộng như freq-3000.
    #[test]
    fn op006_tech_terms_completion_keeps_case() {
        assert_eq!(complete_word("ebi", "ebi", &[]), Some("EBITDA".to_string()));
        assert_eq!(complete_word("kp", "kp", &[]), Some("KPI".to_string()));
        assert_eq!(
            complete_word("ebit", "ebit", &[]),
            Some("EBITDA".to_string())
        );
        // R2-4: "cac" không khớp tiền tố thuật ngữ nào (CAC ≠ "các") → None;
        // freq-scan cũng không có từ "cac*" trong từ điển EN.
        assert_eq!(complete_word("cac", "cac", &[]), None);
        // Hoàn tất yêu cầu tiền tố nghiêm ngặt — gõ đủ "kpi" không tự kéo dài.
        assert_eq!(complete_word("kpi", "kpi", &[]), None);
    }

    /// R2-4/R2-56 (vòng 37): tiền tố là âm tiết Việt KHÔNG dấu không được quét
    /// freq/tech thành từ EN — "cac" (các thiếu dấu) KHÔNG được → "cache" dù
    /// "cache" đã nằm trong freq-list domain; "dat" (đắt/đất) không → "date";
    /// "bac" (bạc/bách) không → "back". Chặn tái diễn cả họ lỗi FPR này.
    #[test]
    fn r2_4_vn_ascii_prefix_not_completed_to_en() {
        assert_eq!(complete_word("cac", "cac", &[]), None);
        assert_eq!(complete_word("dat", "dat", &[]), None);
        assert_eq!(complete_word("bac", "bac", &[]), None);
        // Đối chứng dương: tiền tố KHÔNG phải âm tiết Việt vẫn hoàn tất bình
        // thường ("gith" → "github" — từ domain tech mới của vòng 37).
        assert_eq!(
            complete_word("gith", "gith", &[]),
            Some("github".to_string())
        );
        // Từ EN đã biết (trong freq/en_common) không bị guard chặn tiền tố.
        assert_eq!(complete_word("lot", "lot", &[]), Some("lots".to_string()));
    }

    /// R2-58: `đ` trong token không nguyên âm (giá tiền, chữ viết tắt) giữ nguyên.
    #[test]
    fn stroke_without_vowel_is_kept() {
        for (raw, display) in [
            ("dd", "đ"),
            ("000dd", "000đ"),
            ("DDT", "ĐT"),
            ("ddc", "đc"),
            ("DDHQG", "ĐHQG"),
            ("d9", "đ"),
        ] {
            assert!(
                !should_restore(&chars(raw), &chars(display), &[]),
                "`{raw}` → `{display}` phải giữ"
            );
        }
        // Có nguyên âm thì vẫn kiểm cấu trúc như cũ (`add` → `ađ` → restore).
        assert!(should_restore(&chars("add"), &chars("ađ"), &[]));
        // Từ điển cá nhân vẫn thắng.
        assert!(should_restore(
            &chars("ddc"),
            &chars("đc"),
            &["ddc".to_string()]
        ));
    }

    #[test]
    fn no_restore_when_unchanged() {
        assert!(!should_restore(&chars("hello"), &chars("hello"), &[]));
        assert!(!should_restore(&[], &[], &[]));
    }

    #[test]
    fn tu_en_thong_dung_fold_hop_le_duoc_restore() {
        // Từ trong en_common: fold hợp lệ về cấu trúc nhưng KHÔNG phải âm tiết
        // Việt thông dụng → restore (nhánh 2, bật mặc định 2026-10-02).
        for (raw, folded) in [
            ("text", "tẽt"),
            ("see", "sê"),
            ("is", "í"),
            ("yes", "yé"),
            ("saw", "să"),
        ] {
            assert!(
                should_restore(&chars(raw), &chars(folded), &[]),
                "`{raw}` → `{folded}` phải được restore"
            );
        }
    }

    #[test]
    fn cap_mo_ho_giu_tien_g_viet() {
        // Cặp mơ hồ hai chiều: fold là âm tiết Việt thông dụng → GIỮ fold
        // (người gõ tiếng Việt thắng — `cow` là cách gõ Telex của `cơ`).
        // `cow` không nằm trong en_common nên nhánh 2 không bắn; nếu ai thêm
        // vào `english_words` thì nhánh 3 (quyết định tường minh) thắng.
        let user_list = vec!["cow".to_string()];
        assert!(should_restore(&chars("cow"), &chars("cơ"), &user_list));
        // Không khai báo → giữ fold "cơ"
        assert!(!should_restore(&chars("cow"), &chars("cơ"), &[]));
    }

    /// R2-56: âm tiết Việt thật mà từ EN thông dụng fold vào phải được giữ —
    /// `thí`/`hí`/`vơ`/`bõ`/`gá` không có cách gõ Telex nào khác.
    #[test]
    fn am_tiet_viet_that_khong_bi_restore() {
        for raw in [
            "this", "his", "host", "lost", "most", "cost", "best", "vast", "arm", "vow", "row",
            "box", "gas",
        ] {
            let folded = telex(raw);
            let f: String = folded.iter().collect();
            assert!(
                !should_restore(&chars(raw), &folded, &[]),
                "`{raw}` → `{f}` là âm tiết Việt thật — phải giữ"
            );
        }
    }

    /// R2-57: Tab không thay âm tiết Việt thông dụng bằng từ EN (`có`+Tab → `cost`).
    #[test]
    fn tab_khong_cuop_am_tiet_viet_thong_dung() {
        for (typed, folded) in [("cos", "có"), ("bes", "bé"), ("vas", "vá"), ("los", "ló")] {
            assert_eq!(
                complete_word(typed, folded, &[]),
                None,
                "`{typed}` (`{folded}`) + Tab"
            );
        }
        // `có` được bảo vệ cả khi từ điển cá nhân có từ bắt đầu bằng `cos`.
        assert_eq!(complete_word("cos", "có", &["cosplay".to_string()]), None);
    }

    #[test]
    fn danh_sach_nguoi_dung_thang_moi_phong_doan() {
        let list = vec!["Text".to_string()];
        assert!(
            should_restore(&chars("text"), &chars("tẽt"), &list),
            "so khớp phải không phân biệt hoa thường"
        );
        // Từ không ở đâu cả nhưng người dùng khai báo → vẫn restore
        let list2 = vec!["nest".to_string()];
        assert!(should_restore(&chars("nest"), &chars("nết"), &list2));
    }

    #[test]
    fn parse_bo_comment_va_dong_trong() {
        let src = "# chú thích\n\ntext\n  keep  \n#x\n";
        assert_eq!(
            parse_word_list(src),
            vec!["text".to_string(), "keep".to_string()]
        );
    }

    /// Mỗi dòng trong `data/stop_en.txt` phải **thỏa** điều kiện đã ghi trong file:
    /// fold bằng Telex khác gốc và là âm tiết hợp lệ về cấu trúc. Nếu không, mục đó vô
    /// nghĩa (không có gì để restore) và test này đỏ.
    #[test]
    fn data_stop_en_file_is_self_consistent() {
        let words = default_english_words();
        assert!(words.len() >= 20, "danh sách mẫu quá ngắn: {}", words.len());
        for w in &words {
            assert!(
                w.chars().all(|c| c.is_ascii_alphabetic()),
                "`{w}` phải là chữ cái ASCII không khoảng trắng"
            );
            let folded = telex(w);
            let f: String = folded.iter().collect();
            assert_ne!(
                &f, w,
                "`{w}` fold ra chính nó → không có biến đổi để restore"
            );
            assert!(
                is_valid_word(&folded),
                "`{w}` fold thành `{f}` — không phải âm tiết hợp lệ → mục vô nghĩa"
            );
        }
        // không trùng lặp
        let mut uniq = words.clone();
        uniq.sort();
        let before = uniq.len();
        uniq.dedup();
        assert_eq!(before, uniq.len(), "danh sách có từ trùng");
    }

    /// Ca ngược lại: những mục trong danh sách mà fold ra **từ Việt thật** là ca mơ hồ —
    /// phải được nêu rõ trong file data để người dùng biết trước khi bật.
    #[test]
    fn canh_bao_cai_muc_mo_ho() {
        let words = default_english_words();
        // `test` -> `tết` và `list` -> `lít` là từ Việt có nghĩa thật.
        for ambiguous in ["test", "list"] {
            assert!(
                words.iter().any(|w| w == ambiguous),
                "file data phải giữ `{ambiguous}` (minh hoạ ca mơ hồ) + có ghi chú cảnh báo"
            );
        }
    }

    /// Oracle cho `en_common.txt`: MỌI mục phải **bắn** — fold khác chính nó,
    /// là âm tiết Việt hợp lệ, và KHÔNG nằm trong vn_common (nếu không, mục
    /// vô nghĩa: nhánh cấu trúc đã bắt, hoặc tiếng Việt đã thắng). Báo TẤT CẢ
    /// vi phạm trong một lần chạy để chỉnh data nhanh.
    #[test]
    fn data_en_common_file_fires() {
        // Từ nhánh cấu trúc đã restore (R2-62: âm cuối tắc không sắc/nặng) nhưng vẫn giữ
        // làm gợi ý Tab — bỏ đi thì `tex`+Tab ra `texts`.
        const TAB_ONLY: [&str; 4] = ["text", "next", "art", "cart"];
        for w in TAB_ONLY {
            assert!(!is_valid_word(&telex(w)), "{w}: không còn là TAB_ONLY");
        }
        let words = parse_word_list(EN_COMMON_DATA);
        // Ngưỡng chỉ chặn lỡ tay xoá trắng file: R2-56 đã bỏ các từ fold ra âm tiết
        // Việt thật (`this`→`thí`, `host`→`hót`…), danh sách còn ~20 mục.
        assert!(
            words.len() >= 15,
            "danh sách EN thông dụng quá ngắn: {}",
            words.len()
        );
        let mut uniq = words.clone();
        uniq.sort();
        uniq.dedup();
        assert_eq!(uniq.len(), words.len(), "en_common có từ trùng");
        let mut bad: Vec<String> = Vec::new();
        for w in &words {
            if !w.chars().all(|c| c.is_ascii_alphabetic()) {
                bad.push(format!("{w}: phải là chữ cái ASCII"));
                continue;
            }
            let folded = telex(w);
            let f: String = folded.iter().collect();
            if &f == w {
                bad.push(format!("{w}: fold ra chính nó — bỏ đi"));
            } else if !is_valid_word(&folded) && !TAB_ONLY.contains(&w.as_str()) {
                bad.push(format!(
                    "{w}: fold `{f}` KHÔNG hợp lệ — nhánh cấu trúc đã restore, bỏ đi"
                ));
            } else if vn_common_contains(&f) {
                bad.push(format!(
                    "{w}: fold `{f}` là âm tiết Việt thông dụng — GIỮ tiếng Việt, bỏ khỏi en_common"
                ));
            }
        }
        assert!(
            bad.is_empty(),
            "en_common có {} mục vô nghĩa:
{}",
            bad.len(),
            bad.join(
                "
"
            )
        );
    }

    /// Oracle cho `vn_common.txt`: mỗi mục phải là âm tiết Việt hợp lệ.
    #[test]
    fn data_vn_common_file_is_valid() {
        let syllables = parse_word_list(VN_COMMON_DATA);
        assert!(
            syllables.len() >= 30,
            "danh sách âm tiết Việt quá ngắn: {}",
            syllables.len()
        );
        let mut uniq = syllables.clone();
        uniq.sort();
        uniq.dedup();
        assert_eq!(uniq.len(), syllables.len(), "vn_common có mục trùng");
        let bad: Vec<String> = syllables
            .iter()
            .filter(|s| !is_valid_word(&chars(s)))
            .map(|s| format!("{s}: không phải âm tiết Việt hợp lệ"))
            .collect();
        assert!(
            bad.is_empty(),
            "vn_common có {} mục sai:
{}",
            bad.len(),
            bad.join(
                "
"
            )
        );
    }

    #[test]
    fn tab_goi_y_hoan_tu_tieng_anh() {
        // Tiền tố nghiêm ngặt → từ ngắn nhất khớp ("tes" đang fold thành "té")
        assert_eq!(complete_word("tes", "té", &[]), Some("test".to_string()));
        // Fold là âm tiết Việt thông dụng → không gợi ý (ưu tiên tiếng Việt)
        assert_eq!(complete_word("cow", "cơ", &[]), None);
        // Đã là từ đầy đủ, không mục nào dài hơn trong từ điển → không gợi ý
        assert_eq!(complete_word("hex", "hẽ", &[]), None);
        // Tiền tố nghiêm ngặt → từ ngắn nhất khớp ("see" là tiền tố của "seem"
        // trong danh sách 2026-10-18)
        assert_eq!(complete_word("see", "sê", &[]), Some("seem".to_string()));
        // Quá ngắn
        assert_eq!(complete_word("s", "s", &[]), None);
        // Danh sách người dùng tham gia ứng viên
        assert_eq!(
            complete_word("vn", "vn", &["vnexpress".to_string()]),
            Some("vnexpress".to_string())
        );
        // Không có ứng viên → None
        assert_eq!(complete_word("zzz", "zzz", &[]), None);
        // Từ điển cá nhân viết hoa vẫn khớp tiền tố gõ thường
        assert_eq!(
            complete_word("vn", "vn", &["VnExpress".to_string()]),
            Some("vnexpress".to_string())
        );
    }

    #[test]
    fn e61_extra_and_tech_tables_strictly_sorted() {
        assert!(
            EN_EXTRA_LEXICON_WORDS.windows(2).all(|w| w[0] < w[1]),
            "EN_EXTRA_LEXICON_WORDS phải sắp xếp tăng dần nghiêm ngặt cho binary_search"
        );
        assert!(
            TECH_UNDO_WORDS.windows(2).all(|w| w[0] < w[1]),
            "TECH_UNDO_WORDS phải sắp xếp tăng dần nghiêm ngặt cho binary_search"
        );
    }
}
