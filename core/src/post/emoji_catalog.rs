// SPDX-License-Identifier: GPL-3.0-or-later
//! Danh mục Emoji nhẹ (P12) — bộ ~280 emoji quen thuộc chuẩn hệ điều hành / MS Teams / iOS.
//!
//! Thiết kế nhẹ & sạch tuyệt đối (zero-dep, không I/O, không tải bộ dữ liệu MB):
//! - Mảng hằng `CATALOG` biên dịch tĩnh (~22 KB), phủ đủ 7 nhóm quen thuộc + nhóm 0 "Gần đây".
//! - Tìm kiếm song ngữ Việt–Anh **không phân biệt dấu** (`cuoi` / `cười` / `smile` đều khớp).
//! - Hỗ trợ mở rộng nhanh `:shortcode:` ngay trong ô gõ (`:smile:`, `:fire:`, `:ok:`, `:tim:`…)
//!   với rào chống kích hoạt nhầm trong URL / giờ (`12:30:00`) / C++/Rust (`std::fmt`).
//! - MRU giới hạn cứng 32 phần tử (`EmojiMru`) — G8: chỉ giữ mã glyph thuộc `CATALOG`, không lưu
//!   bất kỳ nội dung gõ hay chuỗi tự do nào của người dùng.

/// Số lượng tối đa emoji trong danh sách "Gần đây" (MRU).
pub const EMOJI_MRU_CAP: usize = 32;

/// Mã nhóm emoji (0 = Gần đây, 1..=7 = 7 nhóm quen thuộc).
pub const CAT_RECENT: u8 = 0;
pub const CAT_SMILEYS: u8 = 1;
pub const CAT_GESTURES: u8 = 2;
pub const CAT_SYMBOLS: u8 = 3;
pub const CAT_NATURE: u8 = 4;
pub const CAT_FOOD: u8 = 5;
pub const CAT_WORK: u8 = 6;
pub const CAT_ACTIVITIES: u8 = 7;

/// Một mục emoji trong danh mục tĩnh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmojiEntry {
    pub glyph: &'static str,
    pub shortcode: &'static str,
    pub category: u8,
    /// Từ khóa tiếng Việt (chữ thường, đã có cả bản không dấu trong từ khóa để tìm nhanh).
    pub vi: &'static str,
    /// Từ khóa tiếng Anh (chữ thường).
    pub en: &'static str,
}

macro_rules! em {
    ($g:expr, $sc:expr, $cat:expr, $vi:expr, $en:expr) => {
        EmojiEntry {
            glyph: $g,
            shortcode: $sc,
            category: $cat,
            vi: $vi,
            en: $en,
        }
    };
}

/// ~280 emoji quen thuộc nhất trên iOS / Windows / MS Teams / Zalo / Slack.
pub static CATALOG: &[EmojiEntry] = &[
    // =================== 1. Smileys & Emotion (Cảm xúc) ===================
    em!(
        "😀",
        "grin",
        CAT_SMILEYS,
        "cuoi tuoi vui",
        "grinning smile happy"
    ),
    em!(
        "😃",
        "smiley",
        CAT_SMILEYS,
        "cuoi rong vui ve",
        "smiley happy joy"
    ),
    em!(
        "😄",
        "smile",
        CAT_SMILEYS,
        "cuoi hanh phuc vui",
        "smile happy grin"
    ),
    em!(
        "😁",
        "beaming",
        CAT_SMILEYS,
        "cuoi nhe rang",
        "beaming grin teeth"
    ),
    em!(
        "😆",
        "laughing",
        CAT_SMILEYS,
        "cuoi lon haha",
        "laughing haha xd"
    ),
    em!(
        "😅",
        "sweat_smile",
        CAT_SMILEYS,
        "cuoi guong mo hoi ngai",
        "sweat smile nervous relief"
    ),
    em!(
        "😂",
        "joy",
        CAT_SMILEYS,
        "cuoi ra nuoc mat haha",
        "joy tears laugh lol"
    ),
    em!(
        "🤣",
        "rofl",
        CAT_SMILEYS,
        "cuoi lan lon haha",
        "rofl rolling laugh"
    ),
    em!(
        "😊",
        "blush",
        CAT_SMILEYS,
        "cuoi hien do mat vui",
        "blush warm smile"
    ),
    em!(
        "😇",
        "innocent",
        CAT_SMILEYS,
        "ngay tho thien than",
        "innocent angel halo"
    ),
    em!(
        "🙂",
        "slight_smile",
        CAT_SMILEYS,
        "cuoi nhe",
        "slightly smiling"
    ),
    em!(
        "🙃",
        "upside_down",
        CAT_SMILEYS,
        "lon nguoc cham biem",
        "upside down sarcasm"
    ),
    em!("😉", "wink", CAT_SMILEYS, "nhay mat", "wink flirt"),
    em!(
        "😌",
        "relieved",
        CAT_SMILEYS,
        "nhe nhom yen tam",
        "relieved calm peace"
    ),
    em!(
        "😍",
        "heart_eyes",
        CAT_SMILEYS,
        "mat trai tim me man yeu",
        "heart eyes love crush"
    ),
    em!(
        "🥰",
        "smiling_hearts",
        CAT_SMILEYS,
        "yeu thuong am ap",
        "smiling hearts adore love"
    ),
    em!(
        "😘",
        "kiss",
        CAT_SMILEYS,
        "hon gio yeu",
        "kiss blowing heart"
    ),
    em!(
        "😋",
        "yum",
        CAT_SMILEYS,
        "ngon them an",
        "yum delicious tasty"
    ),
    em!(
        "😛",
        "tongue",
        CAT_SMILEYS,
        "le luoi treu",
        "tongue playful"
    ),
    em!(
        "😜",
        "wink_tongue",
        CAT_SMILEYS,
        "nhay mat le luoi",
        "winking tongue crazy"
    ),
    em!(
        "🤪",
        "zany",
        CAT_SMILEYS,
        "nhang nhit hai huoc",
        "zany goofy silly"
    ),
    em!(
        "🤨",
        "raised_eyebrow",
        CAT_SMILEYS,
        "nhuong may nghi ngo",
        "raised eyebrow skeptical"
    ),
    em!(
        "🧐",
        "monocle",
        CAT_SMILEYS,
        "soi xet suy ngam kinh",
        "monocle inspect curious"
    ),
    em!(
        "🤓",
        "nerd",
        CAT_SMILEYS,
        "mot sach deo kinh",
        "nerd geek glasses"
    ),
    em!(
        "😎",
        "sunglasses",
        CAT_SMILEYS,
        "ngau kinh ram chat",
        "sunglasses cool awesome"
    ),
    em!(
        "🤩",
        "star_struck",
        CAT_SMILEYS,
        "mat sao nguong mo",
        "star struck amazed wow"
    ),
    em!(
        "🥳",
        "partying",
        CAT_SMILEYS,
        "tiec tung chuc mung sinh nhat",
        "partying celebrate birthday"
    ),
    em!(
        "😏",
        "smirk",
        CAT_SMILEYS,
        "cuoi khay tu tin",
        "smirk sly smug"
    ),
    em!(
        "😒",
        "unamused",
        CAT_SMILEYS,
        "chan nan khong vui",
        "unamused meh"
    ),
    em!(
        "😞",
        "disappointed",
        CAT_SMILEYS,
        "that vong buon",
        "disappointed sad"
    ),
    em!(
        "😔",
        "pensive",
        CAT_SMILEYS,
        "tram tu buon",
        "pensive sad sorry"
    ),
    em!("😟", "worried", CAT_SMILEYS, "lo lang", "worried concerned"),
    em!(
        "😕",
        "confused",
        CAT_SMILEYS,
        "boi roi kho hieu",
        "confused puzzled"
    ),
    em!("🙁", "frown", CAT_SMILEYS, "buon xiu", "frowning sad"),
    em!(
        "😣",
        "persevere",
        CAT_SMILEYS,
        "co gang kho khan",
        "persevere struggle"
    ),
    em!(
        "😖",
        "confounded",
        CAT_SMILEYS,
        "kho xu nhuc dau",
        "confounded frustrated"
    ),
    em!(
        "😫",
        "tired",
        CAT_SMILEYS,
        "met moi kiet suc",
        "tired exhausted"
    ),
    em!(
        "😩",
        "weary",
        CAT_SMILEYS,
        "met qua than tho",
        "weary exhausted"
    ),
    em!(
        "🥺",
        "pleading",
        CAT_SMILEYS,
        "nan ni dang thuong",
        "pleading puppy eyes"
    ),
    em!(
        "😢",
        "cry",
        CAT_SMILEYS,
        "khoc buon nuoc mat",
        "cry tear sad"
    ),
    em!(
        "😭",
        "sob",
        CAT_SMILEYS,
        "khoc lon khoc oa",
        "sob crying loudly"
    ),
    em!(
        "😤",
        "triumph",
        CAT_SMILEYS,
        "tuc gian phun khoi",
        "triumph huff steam"
    ),
    em!("😠", "angry", CAT_SMILEYS, "gian du buc minh", "angry mad"),
    em!(
        "😡",
        "rage",
        CAT_SMILEYS,
        "rat gian phan no",
        "rage furious red"
    ),
    em!(
        "🤬",
        "cursing",
        CAT_SMILEYS,
        "chui the cuc tuc",
        "cursing swearing"
    ),
    em!(
        "🤯",
        "mind_blown",
        CAT_SMILEYS,
        "no nao bat ngo",
        "mind blown exploding head"
    ),
    em!(
        "😳",
        "flushed",
        CAT_SMILEYS,
        "ngai ngung do mat soc",
        "flushed embarrassed"
    ),
    em!("🥵", "hot", CAT_SMILEYS, "nong buc", "hot face heat"),
    em!("🥶", "cold", CAT_SMILEYS, "lanh cong", "cold face freezing"),
    em!(
        "😱",
        "scream",
        CAT_SMILEYS,
        "het hoang so hai",
        "scream fear shocked"
    ),
    em!(
        "😨",
        "fearful",
        CAT_SMILEYS,
        "so hai lo so",
        "fearful scared"
    ),
    em!(
        "😰",
        "anxious",
        CAT_SMILEYS,
        "lo au mo hoi",
        "anxious sweat nervous"
    ),
    em!(
        "😥",
        "sad_relieved",
        CAT_SMILEYS,
        "buon nhe nhom",
        "sad relieved sweat"
    ),
    em!(
        "😓",
        "downcast_sweat",
        CAT_SMILEYS,
        "ap luc mo hoi",
        "downcast sweat hard"
    ),
    em!(
        "🤗",
        "hug",
        CAT_SMILEYS,
        "om ap than thien",
        "hugging hug warm"
    ),
    em!(
        "🤔",
        "thinking",
        CAT_SMILEYS,
        "suy nghi thac mac",
        "thinking think hmm"
    ),
    em!(
        "🤭",
        "hand_over_mouth",
        CAT_SMILEYS,
        "che mieng cuoi tham",
        "oops giggle"
    ),
    em!(
        "🤫",
        "shush",
        CAT_SMILEYS,
        "suyt im lang bi mat",
        "shush quiet secret"
    ),
    em!(
        "🤥",
        "lying",
        CAT_SMILEYS,
        "noi doi mui dai",
        "lying pinocchio"
    ),
    em!(
        "😶",
        "no_mouth",
        CAT_SMILEYS,
        "im lang khong noi",
        "speechless silent"
    ),
    em!(
        "😐",
        "neutral",
        CAT_SMILEYS,
        "binh thuong mat do",
        "neutral poker face"
    ),
    em!(
        "😑",
        "expressionless",
        CAT_SMILEYS,
        "vo cam chan",
        "expressionless blank"
    ),
    em!(
        "😬",
        "grimacing",
        CAT_SMILEYS,
        "nhan nho ngai",
        "grimacing awkward"
    ),
    em!(
        "🙄",
        "rolling_eyes",
        CAT_SMILEYS,
        "dao mat ngao ngan",
        "rolling eyes whatever"
    ),
    em!(
        "😯",
        "hushed",
        CAT_SMILEYS,
        "ngac nhien o",
        "hushed surprised"
    ),
    em!(
        "😮",
        "open_mouth",
        CAT_SMILEYS,
        "ha hoc mom bat ngo",
        "open mouth wow"
    ),
    em!(
        "😲",
        "astonished",
        CAT_SMILEYS,
        "kinh ngac soc",
        "astonished shocked"
    ),
    em!(
        "🥱",
        "yawn",
        CAT_SMILEYS,
        "ngap buon ngu",
        "yawning sleepy bored"
    ),
    em!(
        "😴",
        "sleeping",
        CAT_SMILEYS,
        "ngu ngon zzz",
        "sleeping zzz tired"
    ),
    em!(
        "🤤",
        "drooling",
        CAT_SMILEYS,
        "chay nuoc mieng them",
        "drooling yum"
    ),
    em!("😪", "sleepy", CAT_SMILEYS, "buon ngu", "sleepy tired"),
    em!(
        "😵",
        "dizzy",
        CAT_SMILEYS,
        "chong mat choang",
        "dizzy knocked out"
    ),
    em!(
        "🤐",
        "zipper_mouth",
        CAT_SMILEYS,
        "khoa mieng giu bi mat",
        "zipper mouth secret"
    ),
    em!(
        "🥴",
        "woozy",
        CAT_SMILEYS,
        "say xin choang vang",
        "woozy drunk tipsy"
    ),
    em!(
        "🤢",
        "nauseated",
        CAT_SMILEYS,
        "buon non ghe",
        "nauseated sick green"
    ),
    em!(
        "🤮",
        "vomiting",
        CAT_SMILEYS,
        "non mua",
        "vomiting puke sick"
    ),
    em!(
        "🤧",
        "sneezing",
        CAT_SMILEYS,
        "hat hoi cam cum",
        "sneezing tissue cold"
    ),
    em!(
        "😷",
        "mask",
        CAT_SMILEYS,
        "khau trang om",
        "mask medical sick"
    ),
    em!(
        "🤒",
        "thermometer",
        CAT_SMILEYS,
        "sot om benh",
        "thermometer fever sick"
    ),
    em!(
        "🤕",
        "head_bandage",
        CAT_SMILEYS,
        "dau dau bang bo",
        "bandage hurt injured"
    ),
    em!(
        "🤑",
        "money_mouth",
        CAT_SMILEYS,
        "tien bac giau",
        "money mouth rich"
    ),
    em!("🤠", "cowboy", CAT_SMILEYS, "cao boi mu", "cowboy hat"),
    em!(
        "😈",
        "smiling_imp",
        CAT_SMILEYS,
        "tinh nghich ac quy",
        "devil imp mischief"
    ),
    em!("🤡", "clown", CAT_SMILEYS, "chu he", "clown face"),
    em!(
        "👻",
        "ghost",
        CAT_SMILEYS,
        "con ma halloween",
        "ghost boo halloween"
    ),
    em!(
        "💀",
        "skull",
        CAT_SMILEYS,
        "dau lau chet cuoi",
        "skull dead skeleton"
    ),
    em!(
        "👽",
        "alien",
        CAT_SMILEYS,
        "nguoi ngoai hanh tinh",
        "alien ufo"
    ),
    em!("🤖", "robot", CAT_SMILEYS, "ro bot ai may", "robot bot ai"),
    em!("💩", "poop", CAT_SMILEYS, "cuc phan hai huoc", "poop pile"),
    em!(
        "🙈",
        "see_no_evil",
        CAT_SMILEYS,
        "khi che mat ngai",
        "see no evil monkey shy"
    ),
    em!(
        "🙉",
        "hear_no_evil",
        CAT_SMILEYS,
        "khi biet tai",
        "hear no evil monkey"
    ),
    em!(
        "🙊",
        "speak_no_evil",
        CAT_SMILEYS,
        "khi che mieng lo loi",
        "speak no evil monkey oops"
    ),
    // =================== 2. People & Gestures (Cử chỉ & Người) ===================
    em!(
        "👋",
        "wave",
        CAT_GESTURES,
        "vay tay xin chao tam biet",
        "wave hello hi bye"
    ),
    em!(
        "🤚",
        "raised_back_of_hand",
        CAT_GESTURES,
        "gio tay",
        "raised back hand"
    ),
    em!(
        "✋",
        "raised_hand",
        CAT_GESTURES,
        "dung lai gio tay",
        "raised hand stop high five"
    ),
    em!(
        "👌",
        "ok",
        CAT_GESTURES,
        "ok duoc tuyet voi",
        "ok hand perfect"
    ),
    em!(
        "🤌",
        "pinched_fingers",
        CAT_GESTURES,
        "tinh te y",
        "pinched fingers chef kiss"
    ),
    em!(
        "🤏",
        "pinching_hand",
        CAT_GESTURES,
        "chut xiu nho",
        "pinching tiny small"
    ),
    em!(
        "✌️",
        "victory",
        CAT_GESTURES,
        "chien thang hoa binh hai ngon",
        "victory peace v"
    ),
    em!(
        "🤞",
        "crossed_fingers",
        CAT_GESTURES,
        "cau may hy vong",
        "crossed fingers luck hope"
    ),
    em!(
        "🤟",
        "love_you",
        CAT_GESTURES,
        "yeu ban ky hieu",
        "love you gesture"
    ),
    em!("🤘", "rock", CAT_GESTURES, "rock ngau", "rock metal horns"),
    em!(
        "🤙",
        "call_me",
        CAT_GESTURES,
        "goi cho toi nha",
        "call me shaka"
    ),
    em!("👈", "point_left", CAT_GESTURES, "chi trai", "point left"),
    em!("👉", "point_right", CAT_GESTURES, "chi phai", "point right"),
    em!("👆", "point_up", CAT_GESTURES, "chi len tren", "point up"),
    em!(
        "👇",
        "point_down",
        CAT_GESTURES,
        "chi xuong duoi",
        "point down"
    ),
    em!(
        "☝️",
        "index_up",
        CAT_GESTURES,
        "luu y mot dieu",
        "index pointing up note"
    ),
    em!(
        "👍",
        "thumbsup",
        CAT_GESTURES,
        "thich ok dong y tuyet",
        "thumbs up like yes +1"
    ),
    em!(
        "👎",
        "thumbsdown",
        CAT_GESTURES,
        "khong thich phan doi",
        "thumbs down dislike no -1"
    ),
    em!(
        "✊",
        "fist",
        CAT_GESTURES,
        "quyet tam nam dam",
        "raised fist power"
    ),
    em!(
        "👊",
        "punch",
        CAT_GESTURES,
        "cung tay dam",
        "fist bump punch"
    ),
    em!(
        "👏",
        "clap",
        CAT_GESTURES,
        "vo tay chuc mung gioi",
        "clap applause bravo"
    ),
    em!(
        "🙌",
        "raised_hands",
        CAT_GESTURES,
        "hoan ho tuyet voi",
        "raised hands hooray yay"
    ),
    em!("👐", "open_hands", CAT_GESTURES, "mo tay", "open hands"),
    em!(
        "🤝",
        "handshake",
        CAT_GESTURES,
        "bat tay hop tac dong y",
        "handshake deal agree"
    ),
    em!(
        "🙏",
        "pray",
        CAT_GESTURES,
        "cam on lam on cau nguyen",
        "pray thanks please namaste"
    ),
    em!(
        "✍️",
        "writing_hand",
        CAT_GESTURES,
        "viet ghi chu",
        "writing hand note"
    ),
    em!(
        "💪",
        "muscle",
        CAT_GESTURES,
        "manh me co bap co len",
        "muscle strong flex"
    ),
    em!(
        "🧠",
        "brain",
        CAT_GESTURES,
        "nao thong minh tu duy",
        "brain smart think"
    ),
    em!(
        "👀",
        "eyes",
        CAT_GESTURES,
        "doi mat xem nhin",
        "eyes look watch see"
    ),
    em!("👁️", "eye", CAT_GESTURES, "con mat", "eye"),
    em!(
        "👨‍💻",
        "technologist_man",
        CAT_GESTURES,
        "lap trinh vien dev it",
        "developer coder man"
    ),
    em!(
        "👩‍💻",
        "technologist_woman",
        CAT_GESTURES,
        "nu lap trinh vien dev it",
        "developer coder woman"
    ),
    em!(
        "🎓",
        "grad_cap",
        CAT_GESTURES,
        "tot nghiep dai hoc hoc tap",
        "graduation cap degree"
    ),
    em!(
        "🏃",
        "runner",
        CAT_GESTURES,
        "chay bo voi va",
        "running run sprint"
    ),
    em!(
        "💃",
        "dancer",
        CAT_GESTURES,
        "nhay mua vui",
        "dancer party dance"
    ),
    em!(
        "🕺",
        "man_dancing",
        CAT_GESTURES,
        "khieu vu",
        "man dancing disco"
    ),
    em!(
        "🧘",
        "meditate",
        CAT_GESTURES,
        "thien thu gian yoga",
        "meditate yoga zen"
    ),
    em!(
        "🤷",
        "shrug",
        CAT_GESTURES,
        "nhun vai khong biet",
        "shrug dunno idk"
    ),
    em!(
        "🤦",
        "facepalm",
        CAT_GESTURES,
        "vo tran bo tay",
        "facepalm sigh"
    ),
    em!(
        "🙋",
        "raising_hand",
        CAT_GESTURES,
        "gio tay phat bieu",
        "raising hand question"
    ),
    // =================== 3. Hearts & Symbols (Trái tim & Biểu tượng) ===================
    em!(
        "❤️",
        "heart",
        CAT_SYMBOLS,
        "trai tim do yeu thuong",
        "red heart love"
    ),
    em!(
        "🧡",
        "orange_heart",
        CAT_SYMBOLS,
        "trai tim cam",
        "orange heart"
    ),
    em!(
        "💛",
        "yellow_heart",
        CAT_SYMBOLS,
        "trai tim vang",
        "yellow heart"
    ),
    em!(
        "💚",
        "green_heart",
        CAT_SYMBOLS,
        "trai tim xanh la",
        "green heart"
    ),
    em!(
        "💙",
        "blue_heart",
        CAT_SYMBOLS,
        "trai tim xanh duong",
        "blue heart"
    ),
    em!(
        "💜",
        "purple_heart",
        CAT_SYMBOLS,
        "trai tim tim",
        "purple heart"
    ),
    em!(
        "🖤",
        "black_heart",
        CAT_SYMBOLS,
        "trai tim den",
        "black heart"
    ),
    em!(
        "🤍",
        "white_heart",
        CAT_SYMBOLS,
        "trai tim trang",
        "white heart"
    ),
    em!(
        "💔",
        "broken_heart",
        CAT_SYMBOLS,
        "trai tim tan vo buon",
        "broken heart sad"
    ),
    em!(
        "💕",
        "two_hearts",
        CAT_SYMBOLS,
        "hai trai tim yeu",
        "two hearts love"
    ),
    em!(
        "💖",
        "sparkling_heart",
        CAT_SYMBOLS,
        "trai tim lap lanh",
        "sparkling heart"
    ),
    em!(
        "💘",
        "cupid",
        CAT_SYMBOLS,
        "mui ten tinh yeu",
        "cupid heart arrow"
    ),
    em!("💌", "love_letter", CAT_SYMBOLS, "thu tinh", "love letter"),
    em!(
        "💯",
        "100",
        CAT_SYMBOLS,
        "tram diem hoan hao chuan",
        "100 hundred perfect"
    ),
    em!("💢", "anger", CAT_SYMBOLS, "tuc gian", "anger symbol"),
    em!("💥", "boom", CAT_SYMBOLS, "bung no", "boom collision bang"),
    em!("💫", "dizzy_star", CAT_SYMBOLS, "sao xoay", "dizzy star"),
    em!("💤", "zzz", CAT_SYMBOLS, "ngu zzz", "zzz sleep"),
    em!(
        "💬",
        "speech",
        CAT_SYMBOLS,
        "binh luan nhan tin",
        "speech balloon comment chat"
    ),
    em!(
        "💭",
        "thought",
        CAT_SYMBOLS,
        "suy nghi y tuong",
        "thought balloon"
    ),
    em!(
        "✨",
        "sparkles",
        CAT_SYMBOLS,
        "lap lanh moi dep",
        "sparkles shine magic new"
    ),
    em!(
        "⭐",
        "star",
        CAT_SYMBOLS,
        "ngoi sao danh gia",
        "star favorite"
    ),
    em!(
        "🌟",
        "glowing_star",
        CAT_SYMBOLS,
        "sao sang noi bat",
        "glowing star"
    ),
    em!(
        "⚡",
        "zap",
        CAT_SYMBOLS,
        "tia set nhanh nang luong",
        "zap lightning fast power"
    ),
    em!(
        "🔥",
        "fire",
        CAT_SYMBOLS,
        "lua chay hot dinh",
        "fire flame hot lit"
    ),
    em!("🌈", "rainbow", CAT_SYMBOLS, "cau vong", "rainbow"),
    em!(
        "☀️",
        "sun",
        CAT_SYMBOLS,
        "mat troi nang",
        "sun sunny bright"
    ),
    em!(
        "🌤️",
        "partly_sunny",
        CAT_SYMBOLS,
        "nang nhe may",
        "partly sunny"
    ),
    em!("🌧️", "rain", CAT_SYMBOLS, "mua", "rain cloud"),
    em!("⛈️", "storm", CAT_SYMBOLS, "bao sam set", "storm thunder"),
    em!(
        "❄️",
        "snowflake",
        CAT_SYMBOLS,
        "tuyet lanh",
        "snowflake winter cold"
    ),
    em!("💧", "droplet", CAT_SYMBOLS, "giot nuoc", "droplet water"),
    em!("🌊", "wave_water", CAT_SYMBOLS, "song bien", "ocean wave"),
    em!(
        "✅",
        "check",
        CAT_SYMBOLS,
        "xong hoan thanh dung tick",
        "check mark done ok yes"
    ),
    em!(
        "❌",
        "x",
        CAT_SYMBOLS,
        "sai xoa huy loi",
        "cross mark wrong no fail"
    ),
    em!("⭕", "o", CAT_SYMBOLS, "vong tron dung", "heavy circle"),
    em!(
        "❗",
        "exclamation",
        CAT_SYMBOLS,
        "chu y quan trong cham than",
        "exclamation alert important"
    ),
    em!(
        "❓",
        "question",
        CAT_SYMBOLS,
        "cau hoi thac mac",
        "question mark"
    ),
    em!(
        "⚠️",
        "warning",
        CAT_SYMBOLS,
        "canh bao chu y",
        "warning caution alert"
    ),
    em!(
        "🚫",
        "prohibited",
        CAT_SYMBOLS,
        "cam chan",
        "prohibited banned no"
    ),
    em!("♻️", "recycle", CAT_SYMBOLS, "tai che xanh", "recycle"),
    em!("➕", "plus", CAT_SYMBOLS, "dau cong them", "plus add"),
    em!("➖", "minus", CAT_SYMBOLS, "dau tru bot", "minus subtract"),
    em!("♾️", "infinity", CAT_SYMBOLS, "vo cuc", "infinity forever"),
    em!("🆕", "new", CAT_SYMBOLS, "moi", "new badge"),
    em!("🆗", "ok_box", CAT_SYMBOLS, "ok duoc", "ok button"),
    em!("🆘", "sos", CAT_SYMBOLS, "cuu khan cap", "sos help urgent"),
    em!("⬆️", "arrow_up", CAT_SYMBOLS, "mui ten len", "arrow up"),
    em!(
        "⬇️",
        "arrow_down",
        CAT_SYMBOLS,
        "mui ten xuong",
        "arrow down"
    ),
    em!(
        "⬅️",
        "arrow_left",
        CAT_SYMBOLS,
        "mui ten trai",
        "arrow left"
    ),
    em!(
        "➡️",
        "arrow_right",
        CAT_SYMBOLS,
        "mui ten phai",
        "arrow right"
    ),
    em!(
        "🔄",
        "refresh",
        CAT_SYMBOLS,
        "tai lai dong bo",
        "refresh sync arrows"
    ),
    em!(
        "🔔",
        "bell",
        CAT_SYMBOLS,
        "chuong thong bao",
        "bell notification alert"
    ),
    em!(
        "🔕",
        "no_bell",
        CAT_SYMBOLS,
        "tat chuong im lang",
        "muted bell"
    ),
    em!(
        "🎵",
        "musical_note",
        CAT_SYMBOLS,
        "not nhac am nhac",
        "music note song"
    ),
    em!(
        "🎶",
        "notes",
        CAT_SYMBOLS,
        "am nhac giai dieu",
        "musical notes"
    ),
    // =================== 4. Animals & Nature (Động vật & Thiên nhiên) ===================
    em!("🐶", "dog", CAT_NATURE, "cho cun cung", "dog puppy"),
    em!("🐱", "cat", CAT_NATURE, "meo", "cat kitty"),
    em!("🐭", "mouse", CAT_NATURE, "chuot", "mouse"),
    em!("🐰", "rabbit", CAT_NATURE, "tho", "rabbit bunny"),
    em!("🦊", "fox", CAT_NATURE, "cao", "fox"),
    em!("🐻", "bear", CAT_NATURE, "gau", "bear"),
    em!("🐼", "panda", CAT_NATURE, "gau truc", "panda"),
    em!("🐨", "koala", CAT_NATURE, "gau tui", "koala"),
    em!("🐯", "tiger", CAT_NATURE, "ho cop", "tiger"),
    em!("🦁", "lion", CAT_NATURE, "su tu", "lion"),
    em!("🐷", "pig", CAT_NATURE, "heo lon", "pig"),
    em!("🐸", "frog", CAT_NATURE, "ech", "frog"),
    em!("🐵", "monkey", CAT_NATURE, "khi", "monkey"),
    em!("🐔", "chicken", CAT_NATURE, "ga", "chicken"),
    em!(
        "🐧",
        "penguin",
        CAT_NATURE,
        "chim canh cut linux",
        "penguin linux"
    ),
    em!("🦅", "eagle", CAT_NATURE, "dai bang", "eagle"),
    em!("🦉", "owl", CAT_NATURE, "cu meo", "owl"),
    em!("🦄", "unicorn", CAT_NATURE, "ky lan startup", "unicorn"),
    em!("🐝", "bee", CAT_NATURE, "con ong", "bee honey"),
    em!(
        "🐛",
        "bug",
        CAT_NATURE,
        "sau bo loi bug",
        "bug insect software"
    ),
    em!("🦋", "butterfly", CAT_NATURE, "buom", "butterfly"),
    em!("🐢", "turtle", CAT_NATURE, "rua cham", "turtle slow"),
    em!("🐍", "snake", CAT_NATURE, "ran python", "snake python"),
    em!("🐙", "octopus", CAT_NATURE, "bach tuoc", "octopus"),
    em!("🐬", "dolphin", CAT_NATURE, "ca heo", "dolphin"),
    em!("🐳", "whale", CAT_NATURE, "ca voi docker", "whale docker"),
    em!("🦈", "shark", CAT_NATURE, "ca map", "shark"),
    em!(
        "🌸",
        "cherry_blossom",
        CAT_NATURE,
        "hoa anh dao hoa dao",
        "cherry blossom flower"
    ),
    em!(
        "💐",
        "bouquet",
        CAT_NATURE,
        "bo hoa tang",
        "bouquet flowers"
    ),
    em!("🌹", "rose", CAT_NATURE, "hoa hong tang yeu", "rose flower"),
    em!(
        "🌻",
        "sunflower",
        CAT_NATURE,
        "hoa huong duong",
        "sunflower"
    ),
    em!(
        "🌱",
        "seedling",
        CAT_NATURE,
        "mam cay phat trien",
        "seedling sprout grow"
    ),
    em!(
        "🌲",
        "evergreen",
        CAT_NATURE,
        "cay thong rung",
        "evergreen tree"
    ),
    em!(
        "🍀",
        "four_leaf_clover",
        CAT_NATURE,
        "co bon la may man",
        "clover lucky"
    ),
    em!(
        "🍁",
        "maple_leaf",
        CAT_NATURE,
        "la phong mua thu",
        "maple leaf autumn"
    ),
    em!(
        "🌍",
        "earth",
        CAT_NATURE,
        "trai dat toan cau",
        "earth globe world"
    ),
    em!(
        "🌙",
        "moon",
        CAT_NATURE,
        "mat trang dem",
        "moon night crescent"
    ),
    // =================== 5. Food & Drink (Đồ ăn & Thức uống) ===================
    em!("🍎", "apple", CAT_FOOD, "tao do", "apple fruit"),
    em!("🍊", "orange", CAT_FOOD, "cam quyt", "orange tangerine"),
    em!("🍋", "lemon", CAT_FOOD, "chanh vang", "lemon"),
    em!("🍌", "banana", CAT_FOOD, "chuoi", "banana"),
    em!("🍉", "watermelon", CAT_FOOD, "dua hau", "watermelon"),
    em!("🍇", "grapes", CAT_FOOD, "nho", "grapes"),
    em!("🍓", "strawberry", CAT_FOOD, "dau tay", "strawberry"),
    em!("🥭", "mango", CAT_FOOD, "xoai", "mango"),
    em!("🥑", "avocado", CAT_FOOD, "qua bo", "avocado"),
    em!("🌶️", "pepper", CAT_FOOD, "ot cay", "hot pepper spicy"),
    em!("🥐", "croissant", CAT_FOOD, "banh sung bo", "croissant"),
    em!("🥖", "baguette", CAT_FOOD, "banh mi", "baguette bread"),
    em!(
        "🍳",
        "cooking",
        CAT_FOOD,
        "trung op la nau an",
        "cooking egg breakfast"
    ),
    em!(
        "🍔",
        "burger",
        CAT_FOOD,
        "banh mi kep burger",
        "hamburger burger"
    ),
    em!("🍟", "fries", CAT_FOOD, "khoai tay chien", "french fries"),
    em!("🍕", "pizza", CAT_FOOD, "banh pizza", "pizza"),
    em!("🍜", "ramen", CAT_FOOD, "pho mi bun", "ramen pho noodles"),
    em!("🍣", "sushi", CAT_FOOD, "sushi", "sushi"),
    em!("🥟", "dumpling", CAT_FOOD, "banh bao ha cao", "dumpling"),
    em!("🍚", "rice", CAT_FOOD, "com trang", "rice bowl"),
    em!("🍲", "stew", CAT_FOOD, "lau canh", "pot stew hotpot"),
    em!(
        "🍰",
        "cake",
        CAT_FOOD,
        "banh kem ngot",
        "shortcake cake dessert"
    ),
    em!(
        "🎂",
        "birthday",
        CAT_FOOD,
        "banh sinh nhat",
        "birthday cake"
    ),
    em!("🍫", "chocolate", CAT_FOOD, "so co la", "chocolate"),
    em!("🍩", "donut", CAT_FOOD, "banh vong", "doughnut donut"),
    em!("🍪", "cookie", CAT_FOOD, "banh quy", "cookie"),
    em!(
        "☕",
        "coffee",
        CAT_FOOD,
        "ca phe cafe sang",
        "coffee cafe morning"
    ),
    em!("🍵", "tea", CAT_FOOD, "tra xanh", "tea matcha green"),
    em!(
        "🧋",
        "boba",
        CAT_FOOD,
        "tra sua tran chau",
        "bubble tea boba"
    ),
    em!("🍺", "beer", CAT_FOOD, "bia nhau", "beer mug"),
    em!(
        "🍻",
        "beers",
        CAT_FOOD,
        "cung ly bia chuc mung",
        "clinking beers cheers"
    ),
    em!(
        "🥂",
        "champagne",
        CAT_FOOD,
        "cung ly ruou chuc mung",
        "clinking glasses cheers"
    ),
    em!("🍷", "wine", CAT_FOOD, "ruou vang", "wine glass"),
    em!(
        "🍹",
        "tropical_drink",
        CAT_FOOD,
        "nuoc trai cay cocktail",
        "tropical drink cocktail"
    ),
    em!("🥤", "cup_straw", CAT_FOOD, "nuoc ngot", "cup straw soda"),
    // =================== 6. Work, Office & Tech (Công việc & Công nghệ) ===================
    em!(
        "💻",
        "laptop",
        CAT_WORK,
        "may tinh laptop lam viec code",
        "laptop computer work"
    ),
    em!(
        "🖥️",
        "desktop",
        CAT_WORK,
        "may tinh de ban man hinh",
        "desktop monitor pc"
    ),
    em!("⌨️", "keyboard", CAT_WORK, "ban phim go", "keyboard typing"),
    em!(
        "📱",
        "phone",
        CAT_WORK,
        "dien thoai di dong",
        "mobile phone smartphone"
    ),
    em!(
        "📞",
        "telephone",
        CAT_WORK,
        "goi dien thoai",
        "telephone call"
    ),
    em!("🔋", "battery", CAT_WORK, "pin nang luong", "battery power"),
    em!(
        "💡",
        "bulb",
        CAT_WORK,
        "y tuong sang kien bong den",
        "light bulb idea tip"
    ),
    em!(
        "⏰",
        "alarm",
        CAT_WORK,
        "dong ho bao thuc deadline",
        "alarm clock deadline"
    ),
    em!(
        "⏱️",
        "stopwatch",
        CAT_WORK,
        "bam gio toc do",
        "stopwatch timer fast"
    ),
    em!(
        "⏳",
        "hourglass",
        CAT_WORK,
        "cho doi dong ho cat",
        "hourglass waiting loading"
    ),
    em!(
        "📅",
        "calendar",
        CAT_WORK,
        "lich hop ke hoach",
        "calendar schedule date"
    ),
    em!(
        "📝",
        "memo",
        CAT_WORK,
        "ghi chu tai lieu viet",
        "memo note document"
    ),
    em!("✏️", "pencil", CAT_WORK, "but chi sua", "pencil edit write"),
    em!(
        "📌",
        "pushpin",
        CAT_WORK,
        "ghim luu y quan trong",
        "pushpin pin"
    ),
    em!(
        "📍",
        "round_pushpin",
        CAT_WORK,
        "vi tri dia diem",
        "location pin"
    ),
    em!(
        "📎",
        "paperclip",
        CAT_WORK,
        "dinh kem tep",
        "paperclip attach"
    ),
    em!("✂️", "scissors", CAT_WORK, "cai keo cat", "scissors cut"),
    em!("📁", "folder", CAT_WORK, "thu muc ho so", "folder file"),
    em!("📂", "open_folder", CAT_WORK, "mo thu muc", "open folder"),
    em!(
        "📚",
        "books",
        CAT_WORK,
        "sach hoc tap tai lieu",
        "books study docs"
    ),
    em!("🔖", "bookmark", CAT_WORK, "danh dau luu", "bookmark"),
    em!("🔗", "link", CAT_WORK, "lien ket url", "link chain url"),
    em!("📧", "email", CAT_WORK, "thu dien tu email", "email mail"),
    em!(
        "📨",
        "incoming_email",
        CAT_WORK,
        "thu den",
        "incoming envelope"
    ),
    em!("📤", "outbox", CAT_WORK, "gui di", "outbox send"),
    em!("📥", "inbox", CAT_WORK, "hop thu den", "inbox"),
    em!(
        "📦",
        "package",
        CAT_WORK,
        "goi hang ban phat hanh",
        "package box release"
    ),
    em!(
        "📊",
        "chart",
        CAT_WORK,
        "bieu do bao cao thong ke",
        "bar chart report data"
    ),
    em!(
        "📈",
        "chart_up",
        CAT_WORK,
        "tang truong len",
        "chart increasing growth"
    ),
    em!(
        "📉",
        "chart_down",
        CAT_WORK,
        "giam xuong",
        "chart decreasing"
    ),
    em!(
        "📋",
        "clipboard",
        CAT_WORK,
        "danh sach kiem tra checklist",
        "clipboard checklist"
    ),
    em!(
        "🔍",
        "search",
        CAT_WORK,
        "tim kiem kinh lup",
        "magnifying glass search"
    ),
    em!(
        "🔒",
        "lock",
        CAT_WORK,
        "khoa bao mat",
        "locked security private"
    ),
    em!("🔓", "unlock", CAT_WORK, "mo khoa", "unlocked"),
    em!("🔑", "key", CAT_WORK, "chia khoa mat khau", "key password"),
    em!(
        "🔧",
        "wrench",
        CAT_WORK,
        "sua chua cai dat",
        "wrench fix tool"
    ),
    em!(
        "🛠️",
        "tools",
        CAT_WORK,
        "cong cu ky thuat",
        "hammer wrench tools"
    ),
    em!(
        "⚙️",
        "gear",
        CAT_WORK,
        "cai dat cau hinh",
        "gear settings config"
    ),
    em!(
        "🧪",
        "test_tube",
        CAT_WORK,
        "thu nghiem hoa hoc",
        "test tube science experiment"
    ),
    em!(
        "🔬",
        "microscope",
        CAT_WORK,
        "kinh hien vi khoa hoc",
        "microscope research"
    ),
    em!(
        "💼",
        "briefcase",
        CAT_WORK,
        "cong viec van phong",
        "briefcase business work"
    ),
    em!(
        "🏢",
        "office",
        CAT_WORK,
        "cong ty toa nha",
        "office building"
    ),
    em!("🏠", "house", CAT_WORK, "nha o wfh", "house home"),
    em!(
        "💰",
        "moneybag",
        CAT_WORK,
        "tien luong thuong",
        "money bag rich salary"
    ),
    em!(
        "💵",
        "dollar",
        CAT_WORK,
        "tien mat do la",
        "dollar banknote cash"
    ),
    em!(
        "💳",
        "credit_card",
        CAT_WORK,
        "the thanh toan",
        "credit card payment"
    ),
    // =================== 7. Activities, Travel & Celebrations (Hoạt động & Du lịch) ===================
    em!(
        "🎉",
        "party",
        CAT_ACTIVITIES,
        "chuc mung tiec phao giay",
        "tada party celebrate congrats"
    ),
    em!(
        "🎊",
        "confetti",
        CAT_ACTIVITIES,
        "phao hoa chuc mung",
        "confetti ball"
    ),
    em!("🎈", "balloon", CAT_ACTIVITIES, "bong bay", "balloon party"),
    em!("🎁", "gift", CAT_ACTIVITIES, "qua tang", "gift present"),
    em!(
        "🎄",
        "christmas",
        CAT_ACTIVITIES,
        "giang sinh noel",
        "christmas tree"
    ),
    em!(
        "🎃",
        "halloween",
        CAT_ACTIVITIES,
        "bi ngo halloween",
        "jack o lantern halloween"
    ),
    em!(
        "🎆",
        "fireworks",
        CAT_ACTIVITIES,
        "phao hoa nam moi tet",
        "fireworks new year"
    ),
    em!(
        "🧧",
        "red_envelope",
        CAT_ACTIVITIES,
        "li xi tet may man",
        "red envelope lucky money"
    ),
    em!(
        "🏮",
        "lantern",
        CAT_ACTIVITIES,
        "den long trung thu",
        "lantern"
    ),
    em!(
        "🏆",
        "trophy",
        CAT_ACTIVITIES,
        "cup vo dich thanh tich",
        "trophy winner award"
    ),
    em!(
        "🥇",
        "gold_medal",
        CAT_ACTIVITIES,
        "huy chuong vang nhat",
        "first place gold medal"
    ),
    em!(
        "🥈",
        "silver_medal",
        CAT_ACTIVITIES,
        "huy chuong bac nhi",
        "second place silver"
    ),
    em!(
        "🥉",
        "bronze_medal",
        CAT_ACTIVITIES,
        "huy chuong dong ba",
        "third place bronze"
    ),
    em!("⚽", "soccer", CAT_ACTIVITIES, "bong da", "soccer football"),
    em!("🏀", "basketball", CAT_ACTIVITIES, "bong ro", "basketball"),
    em!("🎾", "tennis", CAT_ACTIVITIES, "quan vot tennis", "tennis"),
    em!("🏸", "badminton", CAT_ACTIVITIES, "cau long", "badminton"),
    em!(
        "🎯",
        "dart",
        CAT_ACTIVITIES,
        "muc tieu trung dich",
        "direct hit target goal"
    ),
    em!(
        "🎮",
        "video_game",
        CAT_ACTIVITIES,
        "choi game tro choi",
        "video game controller"
    ),
    em!("🧩", "puzzle", CAT_ACTIVITIES, "ghep hinh", "puzzle piece"),
    em!(
        "🎨",
        "art",
        CAT_ACTIVITIES,
        "ve thiet ke my thuat",
        "artist palette design"
    ),
    em!(
        "🎬",
        "clapper",
        CAT_ACTIVITIES,
        "phim video quay",
        "clapper board movie"
    ),
    em!(
        "🎤",
        "microphone",
        CAT_ACTIVITIES,
        "mic hat thuyet trinh",
        "microphone karaoke"
    ),
    em!(
        "🎧",
        "headphones",
        CAT_ACTIVITIES,
        "tai nghe nghe nhac",
        "headphones audio"
    ),
    em!("🎸", "guitar", CAT_ACTIVITIES, "dan ghi ta", "guitar music"),
    em!("🚗", "car", CAT_ACTIVITIES, "o to xe hoi", "car automobile"),
    em!("🛵", "scooter", CAT_ACTIVITIES, "xe may", "motor scooter"),
    em!("🚲", "bike", CAT_ACTIVITIES, "xe dap", "bicycle bike"),
    em!(
        "✈️",
        "airplane",
        CAT_ACTIVITIES,
        "may bay du lich",
        "airplane flight travel"
    ),
    em!(
        "🚀",
        "rocket",
        CAT_ACTIVITIES,
        "ten lua phong ra mat nhanh",
        "rocket launch ship deploy"
    ),
    em!(
        "🏖️",
        "beach",
        CAT_ACTIVITIES,
        "bai bien nghi mat",
        "beach vacation"
    ),
    em!("🏔️", "mountain", CAT_ACTIVITIES, "nui leo nui", "mountain"),
    em!(
        "🇻🇳",
        "vn",
        CAT_ACTIVITIES,
        "viet nam co do sao vang",
        "vietnam flag vn"
    ),
    em!("🇺🇸", "us", CAT_ACTIVITIES, "my hoa ky", "usa flag us"),
    em!("🇯🇵", "jp", CAT_ACTIVITIES, "nhat ban", "japan flag jp"),
    em!("🇰🇷", "kr", CAT_ACTIVITIES, "han quoc", "korea flag kr"),
    em!("🇬🇧", "gb", CAT_ACTIVITIES, "anh quoc", "uk flag gb"),
    em!("🇫🇷", "fr", CAT_ACTIVITIES, "phap", "france flag fr"),
    em!("🇩🇪", "de", CAT_ACTIVITIES, "duc", "germany flag de"),
];

/// Bí danh ngắn phổ biến (MS Teams / Slack / tiếng Việt) map sang shortcode chuẩn trong `CATALOG`.
static SHORTCODE_ALIASES: &[(&str, &str)] = &[
    ("+1", "thumbsup"),
    ("-1", "thumbsdown"),
    ("like", "thumbsup"),
    ("dislike", "thumbsdown"),
    ("tim", "heart"),
    ("love", "heart"),
    ("cuoi", "smile"),
    ("haha", "joy"),
    ("lol", "joy"),
    ("khoc", "cry"),
    ("buon", "cry"),
    ("gian", "angry"),
    ("ngau", "sunglasses"),
    ("cool", "sunglasses"),
    ("lua", "fire"),
    ("hot", "fire"),
    ("sao", "star"),
    ("tada", "party"),
    ("chucmung", "party"),
    ("camon", "pray"),
    ("thanks", "pray"),
    ("ty", "pray"),
    ("votay", "clap"),
    ("cafe", "coffee"),
    ("caphe", "coffee"),
    ("bia", "beer"),
    ("trasua", "boba"),
    ("pho", "ramen"),
    ("vietnam", "vn"),
    ("done", "check"),
    ("yes", "check"),
    ("no", "x"),
    ("warn", "warning"),
    ("idea", "bulb"),
    ("think", "thinking"),
    ("hmm", "thinking"),
    ("wow", "astonished"),
    ("hi", "wave"),
    ("bye", "wave"),
    ("dev", "laptop"),
    ("ship", "rocket"),
    ("100", "100"),
];

/// Bộ emoji phổ biến mặc định khi danh sách MRU "Gần đây" còn trống.
static DEFAULT_RECENT_SHORTCODES: &[&str] = &[
    "joy",
    "heart",
    "thumbsup",
    "smile",
    "blush",
    "fire",
    "check",
    "pray",
    "party",
    "clap",
    "thinking",
    "sunglasses",
    "heart_eyes",
    "sob",
    "ok",
    "wave",
    "rocket",
    "sparkles",
    "100",
    "coffee",
    "bulb",
    "warning",
    "x",
    "vn",
];

/// Trạng thái MRU per-engine (G8: chỉ lưu chỉ số mục trong `CATALOG`, tối đa `EMOJI_MRU_CAP`).
#[derive(Debug, Clone, Default)]
pub struct EmojiMru {
    /// Các chỉ số `0..CATALOG.len()` xếp theo mới dùng nhất ở đầu.
    indices: Vec<usize>,
}

impl EmojiMru {
    /// Ghi nhận 1 glyph vừa dùng (từ picker hoặc `:shortcode:`). Bỏ qua nếu không nằm trong `CATALOG`.
    pub fn record(&mut self, glyph: &str) -> bool {
        let Some(idx) = CATALOG.iter().position(|e| e.glyph == glyph) else {
            return false;
        };
        if let Some(pos) = self.indices.iter().position(|&i| i == idx) {
            self.indices.remove(pos);
        }
        self.indices.insert(0, idx);
        if self.indices.len() > EMOJI_MRU_CAP {
            self.indices.truncate(EMOJI_MRU_CAP);
        }
        true
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn entries(&self) -> impl Iterator<Item = &'static EmojiEntry> + '_ {
        self.indices.iter().map(|&i| &CATALOG[i])
    }
}

/// Đưa chuỗi tìm kiếm về ASCII chữ thường không dấu (tiếng Việt + Latin) để so khớp mềm.
pub fn fold_search_key(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        let folded = match ch {
            'A'..='Z' => (ch as u8 + 32) as char,
            'a'..='z' | '0'..='9' | '_' | '-' | '+' | ' ' => ch,
            'á' | 'à' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ấ' | 'ầ'
            | 'ẩ' | 'ẫ' | 'ậ' | 'Á' | 'À' | 'Ả' | 'Ã' | 'Ạ' | 'Ă' | 'Ắ' | 'Ằ' | 'Ẳ' | 'Ẵ' | 'Ặ'
            | 'Â' | 'Ấ' | 'Ầ' | 'Ẩ' | 'Ẫ' | 'Ậ' => 'a',
            'đ' | 'Đ' => 'd',
            'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' | 'É' | 'È' | 'Ẻ'
            | 'Ẽ' | 'Ẹ' | 'Ê' | 'Ế' | 'Ề' | 'Ể' | 'Ễ' | 'Ệ' => 'e',
            'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' | 'Í' | 'Ì' | 'Ỉ' | 'Ĩ' | 'Ị' => 'i',
            'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ớ' | 'ờ'
            | 'ở' | 'ỡ' | 'ợ' | 'Ó' | 'Ò' | 'Ỏ' | 'Õ' | 'Ọ' | 'Ô' | 'Ố' | 'Ồ' | 'Ổ' | 'Ỗ' | 'Ộ'
            | 'Ơ' | 'Ớ' | 'Ờ' | 'Ở' | 'Ỡ' | 'Ợ' => 'o',
            'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' | 'Ú' | 'Ù' | 'Ủ'
            | 'Ũ' | 'Ụ' | 'Ư' | 'Ứ' | 'Ừ' | 'Ử' | 'Ữ' | 'Ự' => 'u',
            'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' | 'Ý' | 'Ỳ' | 'Ỷ' | 'Ỹ' | 'Ỵ' => 'y',
            _ => continue,
        };
        out.push(folded);
    }
    out.trim().to_string()
}

/// Tra shortcode chính xác (không có dấu `:`), hỗ trợ cả bí danh VI/EN phổ biến.
pub fn lookup_shortcode(code: &str) -> Option<&'static EmojiEntry> {
    let key = fold_search_key(code);
    if key.is_empty() {
        return None;
    }
    if let Some(entry) = CATALOG.iter().find(|e| e.shortcode == key) {
        return Some(entry);
    }
    if let Some(&(_, canonical)) = SHORTCODE_ALIASES.iter().find(|&&(a, _)| a == key) {
        return CATALOG.iter().find(|e| e.shortcode == canonical);
    }
    None
}

/// Kiểm tra đuôi `recent` (tính cả dấu `:` vừa gõ ở cuối) có khớp `:shortcode:` không.
///
/// Chống dương tính giả:
/// - Độ dài shortcode trong `2..=24` ký tự ASCII `[a-z0-9_+-]` (chữ thường/số).
/// - Ký tự ngay trước dấu `:` mở (nếu có) KHÔNG được là chữ/số/`:`/`/`/`\`
///   (chặn `http://a:b:`, `12:30:00`, `std::ok:`, `foo:ok:`).
///
/// Trả `(delete_len_before_closing_colon, entry)` — số ký tự đã nằm trong document TRƯỚC
/// phím `:` đóng cần xoá khi thay thế bằng `entry.glyph`.
pub fn match_inline_shortcode(
    recent_before_colon: &[char],
) -> Option<(usize, &'static EmojiEntry)> {
    // Tìm dấu `:` mở gần nhất trong 25 ký tự cuối.
    let len = recent_before_colon.len();
    let max_back = len.min(26);
    let mut open_pos = None;
    for back in 1..=max_back {
        let idx = len - back;
        let c = recent_before_colon[idx];
        if c == ':' {
            open_pos = Some(idx);
            break;
        }
        if !(c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '-')) {
            return None;
        }
    }
    let open = open_pos?;
    let code_len = len - open - 1;
    if !(2..=24).contains(&code_len) {
        return None;
    }
    // Ký tự ngay trước `:` mở không được là chữ/số hoặc `:`, `/`, `\`.
    if open > 0 {
        let prev = recent_before_colon[open - 1];
        if prev.is_alphanumeric() || matches!(prev, ':' | '/' | '\\') {
            return None;
        }
    }
    let code: String = recent_before_colon[open + 1..].iter().collect();
    let entry = lookup_shortcode(&code)?;
    // Xoá từ dấu `:` mở tới hết shortcode (= code_len + 1 ký tự đã ở trong document).
    Some((code_len + 1, entry))
}

/// Tìm kiếm emoji cho UI Picker (trả tối đa `limit` mục, xếp hạng theo độ khớp + MRU).
///
/// - `query` rỗng:
///   - `category == 0` (Gần đây): ưu tiên MRU của người dùng, bổ sung `DEFAULT_RECENT_SHORTCODES`.
///   - `category == 1..=7`: trả toàn bộ mục thuộc nhóm đó (ưu tiên MRU lên đầu nhẹ nhàng? Giữ thứ
///     tự nhóm cố định để user dễ nhớ vị trí mắt).
///   - `category == 0xFF` (Tất cả): MRU trước, sau đó phần còn lại của `CATALOG`.
/// - `query` khác rỗng: tìm xuyên toàn bộ `CATALOG` (bỏ qua tab nhóm trừ khi `category` 1..=7
///   được caller yêu cầu lọc? Khi gõ tìm kiếm, tìm xuyên tất cả nhóm giống iOS/Win/Teams!).
pub fn search_emojis(
    query: &str,
    category: u8,
    mru: &EmojiMru,
    limit: usize,
) -> Vec<&'static EmojiEntry> {
    if limit == 0 {
        return Vec::new();
    }
    let q = fold_search_key(query);
    if q.is_empty() {
        let mut out = Vec::with_capacity(limit.min(CATALOG.len()));
        if category == CAT_RECENT {
            for e in mru.entries() {
                if out.len() >= limit {
                    break;
                }
                out.push(e);
            }
            for &sc in DEFAULT_RECENT_SHORTCODES {
                if out.len() >= limit {
                    break;
                }
                if let Some(e) = CATALOG.iter().find(|e| e.shortcode == sc) {
                    if !out.iter().any(|x| x.glyph == e.glyph) {
                        out.push(e);
                    }
                }
            }
            return out;
        }
        if category == 0xFF {
            for e in mru.entries() {
                if out.len() >= limit {
                    break;
                }
                out.push(e);
            }
            for e in CATALOG {
                if out.len() >= limit {
                    break;
                }
                if !out.iter().any(|x| x.glyph == e.glyph) {
                    out.push(e);
                }
            }
            return out;
        }
        for e in CATALOG {
            if e.category == category {
                out.push(e);
                if out.len() >= limit {
                    break;
                }
            }
        }
        return out;
    }

    // Truy vấn có chữ: tìm toàn bộ danh mục và chấm điểm ưu tiên khớp chính xác / tiền tố / MRU.
    let tokens: Vec<&str> = q.split_whitespace().collect();
    let mut scored: Vec<(u32, usize, &'static EmojiEntry)> = Vec::new();
    for (idx, e) in CATALOG.iter().enumerate() {
        let mut score = 0u32;
        if e.shortcode == q {
            score = 1000;
        } else if SHORTCODE_ALIASES
            .iter()
            .any(|&(a, c)| a == q && c == e.shortcode)
        {
            score = 950;
        } else if e.shortcode.starts_with(&q) {
            score = 800;
        } else if tokens
            .iter()
            .all(|t| e.shortcode.contains(t) || e.vi.contains(t) || e.en.contains(t))
        {
            let word_prefix =
                e.vi.split_whitespace()
                    .chain(e.en.split_whitespace())
                    .any(|w| w == q || w.starts_with(&q));
            score = if word_prefix { 600 } else { 400 };
        }
        if score > 0 {
            if let Some(pos) = mru.indices.iter().position(|&i| i == idx) {
                score = score.saturating_add((EMOJI_MRU_CAP.saturating_sub(pos) as u32) * 4);
            }
            scored.push((score, idx, e));
        }
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    scored.into_iter().take(limit).map(|(_, _, e)| e).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalog_invariants_size_unique_shortcodes_and_valid_categories() {
        assert!(
            CATALOG.len() >= 250 && CATALOG.len() <= 350,
            "kích thước danh mục ngoài dải nhẹ: {}",
            CATALOG.len()
        );
        let mut seen_sc = HashSet::new();
        let mut seen_glyph = HashSet::new();
        for e in CATALOG {
            assert!(!e.glyph.is_empty());
            assert!(!e.shortcode.is_empty());
            assert!(
                (CAT_SMILEYS..=CAT_ACTIVITIES).contains(&e.category),
                "category sai: {} ({})",
                e.category,
                e.shortcode
            );
            assert!(
                seen_sc.insert(e.shortcode),
                "trùng shortcode: {}",
                e.shortcode
            );
            assert!(seen_glyph.insert(e.glyph), "trùng glyph: {}", e.glyph);
        }
        for &(alias, canonical) in SHORTCODE_ALIASES {
            assert!(
                CATALOG.iter().any(|e| e.shortcode == canonical),
                "alias `{alias}` trỏ tới shortcode không tồn tại `{canonical}`"
            );
        }
    }

    #[test]
    fn bilingual_diacritic_insensitive_search() {
        let mru = EmojiMru::default();
        let r_vi = search_emojis("cười", 0, &mru, 10);
        let r_ascii = search_emojis("cuoi", 0, &mru, 10);
        assert!(!r_vi.is_empty());
        assert_eq!(r_vi[0].glyph, r_ascii[0].glyph);

        let r_en = search_emojis("fire", 0, &mru, 5);
        assert_eq!(r_en[0].glyph, "🔥");

        let r_tim = search_emojis("trái tim", 0, &mru, 5);
        assert!(r_tim.iter().any(|e| e.glyph == "❤️"));

        let r_vn = search_emojis("việt nam", 0, &mru, 5);
        assert!(r_vn.iter().any(|e| e.glyph == "🇻🇳"));
    }

    #[test]
    fn mru_bounded_and_ranks_first_in_recent() {
        let mut mru = EmojiMru::default();
        assert!(!mru.record("not-in-catalog"));
        for e in CATALOG.iter().take(40) {
            assert!(mru.record(e.glyph));
        }
        assert_eq!(mru.len(), EMOJI_MRU_CAP);
        assert!(mru.record("🚀"));
        let recent = search_emojis("", CAT_RECENT, &mru, 5);
        assert_eq!(recent[0].glyph, "🚀");
    }

    #[test]
    fn inline_shortcode_matches_cleanly_and_blocks_false_positives() {
        let ok: Vec<char> = "chào :fire".chars().collect();
        let (del, e) = match_inline_shortcode(&ok).expect("phải khớp :fire:");
        assert_eq!(del, 5);
        assert_eq!(e.glyph, "🔥");

        let vi_alias: Vec<char> = ":camon".chars().collect();
        let (del2, e2) = match_inline_shortcode(&vi_alias).expect("phải khớp :camon:");
        assert_eq!(del2, 6);
        assert_eq!(e2.glyph, "🙏");

        // Chống dương tính giả: URL, giờ, Rust/C++ ::, dính liền chữ
        for bad in ["http://a:ok", "12:30", "std::ok", "foo:ok", "::fire", ":a"] {
            let chars: Vec<char> = bad.chars().collect();
            assert!(
                match_inline_shortcode(&chars).is_none(),
                "không được kích hoạt trên `{bad}`"
            );
        }
    }
}
