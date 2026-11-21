import io

p = 'math/src/lexicon.rs'
s = io.open(p, encoding='utf-8').read()

# Replace si_row + si_table with item-level si_group approach
start = s.find('macro_rules! si_row')
end = s.find('si_table! {')
assert 0 < start < end
si_end = s.find('/// Alias ASCII')
assert si_end > end

macro_block = r'''macro_rules! si_group {
    ($pid:ident, $psym:literal, $plat:literal, $pen:literal, $pvi:literal, $units:tt) => {
        const $pid: &[MathEntry] = &[ si_group!(@rows $psym, $plat, $pen, $pvi, $units) ];
    };
    (@rows $psym:literal, $plat:literal, $pen:literal, $pvi:literal, [ $( ($uid:ident, $usym:literal, $ulat:literal, $uen:literal, $uvi:literal) ),* $(,)? ]) => {
        $( e!(
            concat!($psym, $usym), [], PhysicsMech,
            concat!("\\mathrm{", $plat, $ulat, "}"),
            concat!($psym, $usym),
            concat!($psym, $usym),
            concat!("Đơn vị ", $pvi, " ", $uvi),
            concat!($pen, " ", $uen),
            60
        ) ),*
    };
}

macro_rules! si_tests_row {
    ($pid:ident, $psym:literal, $units:tt) => {
        mod $pid {
            use super::super::{lookup_exact, Discipline};
            si_tests_row!(@inner $pid, $psym, $units);
        }
    };
    (@inner $pid:ident, $psym:literal, [ $( ($uid:ident, $usym:literal, $ulat:literal, $uen:literal, $uvi:literal) ),* $(,)? ]) => {
        $( #[test]
        fn $uid() {
            let trig = concat!($psym, $usym);
            let en = lookup_exact(trig).unwrap_or_else(|| panic!("missing {trig}"));
            assert_eq!(en.trigger, trig);
            assert!(!en.unicode.is_empty(), "{trig} unicode rỗng");
            assert!(en.latex.starts_with("\\mathrm{"), "{trig} latex sai định dạng");
            assert!(en.latex.ends_with('}'));
            assert_eq!(en.discipline, Discipline::PhysicsMech);
            assert!(!en.description_vi.is_empty());
            assert!(!en.description_en.is_empty());
            assert!(en.base_weight > 0);
        } )*
    };
}

macro_rules! si_table {
    (
        prefixes: [ $( ($pid:ident, $psym:literal, $plat:literal, $pen:literal, $pvi:literal) ),* $(,)? ],
        units: $units:tt
    ) => {
        $( si_group!($pid, $psym, $plat, $pen, $pvi, $units); )*
        pub const SI_TABLES: &[&[MathEntry]] = &[ $( &$pid ),* ];
        #[cfg(test)]
        mod si_tests {
            use super::SI_TABLES;
            #[test]
            fn si_total_560() {
                let n: usize = SI_TABLES.iter().map(|g| g.len()).sum();
                assert_eq!(n, 20 * 28, "20 tiền tố × 28 đơn vị");
            }
            $( si_tests_row!($pid, $psym, $units); )*
        }
    };
}

'''
s = s[:start] + macro_block + s[end:]

# ALL_ENTRIES: append SI_TABLES via all_slices helper
old_all = '''/// Tổng hợp toàn bộ nhóm — thứ tự: groups thuần → SI → elements → còn lại.
pub const ALL_ENTRIES: &[&[MathEntry]] = &[
    MATH_SYMBOLS,
    GREEK,
    SUPSUB,
    SI_ENTRIES,
    SI_MICRO_ASCII,
    UNITS_MISC,
    ELEMENTS,
    CHEMISTRY,
    PHYSICS,
    ELECTRICAL,
    CS_AI,
    QUANT,
    FORMULAS,
    TEMPLATE_LEX,
    NOTATION,
];'''
new_all = '''/// Tổng hợp toàn bộ nhóm (trừ SI) — SI nằm trong `SI_TABLES`.
pub const ALL_ENTRIES: &[&[MathEntry]] = &[
    MATH_SYMBOLS,
    GREEK,
    SUPSUB,
    SI_MICRO_ASCII,
    UNITS_MISC,
    ELEMENTS,
    CHEMISTRY,
    PHYSICS,
    ELECTRICAL,
    CS_AI,
    QUANT,
    FORMULAS,
    TEMPLATE_LEX,
    NOTATION,
];

/// Toàn bộ slice entry (nhóm thường + 20 nhóm SI).
pub fn all_slices() -> impl Iterator<Item = &'static [MathEntry]> {
    ALL_ENTRIES.iter().copied().chain(SI_TABLES.iter().copied())
}'''
assert old_all in s, 'ALL_ENTRIES block not found'
s = s.replace(old_all, new_all, 1)

# Replace ALL usages of ALL_ENTRIES.iter().flat_map(|s| s.iter()) with all_slices().flat_map(...)
s = s.replace('ALL_ENTRIES\n            .iter()\n            .flat_map(|s| s.iter())', 'all_slices()\n            .flat_map(|s| s.iter())')
s = s.replace('ALL_ENTRIES.iter().flat_map(|s| s.iter())', 'all_slices().flat_map(|s| s.iter())')

# total_entries uses ALL_ENTRIES — update
s = s.replace('''pub fn total_entries() -> usize {
    ALL_ENTRIES.iter().map(|s| s.len()).sum()
}''', '''pub fn total_entries() -> usize {
    all_slices().map(|s| s.len()).sum()
}''')

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('si_group restructure done')

# MathConfig PartialEq
p = 'math/src/config.rs'
s = io.open(p, encoding='utf-8').read()
s = s.replace('/// Cấu hình math (`config.math`) — mọi field có default nên config cũ\n/// không có section `math` vẫn parse được (tương thích ngược).\n#[derive(Debug, Clone, Serialize, Deserialize)]\n#[serde(default)]\npub struct MathConfig {',
              '/// Cấu hình math (`config.math`) — mọi field có default nên config cũ\n/// không có section `math` vẫn parse được (tương thích ngược).\n#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(default)]\npub struct MathConfig {')
s = s.replace('/// Tổ hợp phím toàn cục do TextVN quản lý (user remap được).\n#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct HotkeySpec {',
              '/// Tổ hợp phím toàn cục do TextVN quản lý (user remap được).\n#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct HotkeySpec {')
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('MathConfig PartialEq done')
