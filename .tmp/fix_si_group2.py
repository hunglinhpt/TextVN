import io

p = 'math/src/lexicon.rs'
s = io.open(p, encoding='utf-8').read()

start = s.find('macro_rules! si_group')
end = s.find('si_table! {')
si_end = s.find('/// Alias ASCII')
assert 0 < start < end < si_end, (start, end, si_end)

macro_block = r'''macro_rules! si_group {
    ($pid:ident, $psym:literal, $plat:literal, $pen:literal, $pvi:literal, $units:tt) => {
        mod $pid {
            use super::super::{MathEntry, Discipline};
            si_group!(@consts $psym, $plat, $pen, $pvi, $units);
        }
    };
    (@consts $psym:literal, $plat:literal, $pen:literal, $pvi:literal, [ $( ($uid:ident, $usym:literal, $ulat:literal, $uen:literal, $uvi:literal) ),* $(,)? ]) => {
        $( pub const $uid: MathEntry = e!(
            concat!($psym, $usym), [], PhysicsMech,
            concat!("\\mathrm{", $plat, $ulat, "}"),
            concat!($psym, $usym),
            concat!($psym, $usym),
            concat!("Đơn vị ", $pvi, " ", $uvi),
            concat!($pen, " ", $uen),
            60
        ); )*
        pub const ENTRIES: &[MathEntry] = &[ $($uid),* ];
        si_group!(@tests $psym, [ $( ($uid, $usym) ),* ]);
    };
    (@tests $psym:literal, [ $( ($uid:ident, $usym:literal) ),* $(,)? ]) => {
        #[cfg(test)]
        mod tests {
            use super::super::super::{lookup_exact, Discipline};
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
        }
    };
}

macro_rules! si_table {
    (
        prefixes: [ $( ($pid:ident, $psym:literal, $plat:literal, $pen:literal, $pvi:literal) ),* $(,)? ],
        units: $units:tt
    ) => {
        $( si_group!($pid, $psym, $plat, $pen, $pvi, $units); )*
        pub const SI_TABLES: &[&[MathEntry]] = &[ $( &$pid::ENTRIES ),* ];
        #[cfg(test)]
        mod si_tests {
            use super::SI_TABLES;
            #[test]
            fn si_total_560() {
                let n: usize = SI_TABLES.iter().map(|g| g.len()).sum();
                assert_eq!(n, 20 * 28, "20 tiền tố × 28 đơn vị");
            }
        }
    };
}

'''
s = s[:start] + macro_block + s[end:]
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('si_group item-level done')
