import io

# ast.rs: "(" -> "(".to_string() và E0382 name moved
p = 'math/src/ast.rs'
s = io.open(p, encoding='utf-8').read()
s = s.replace('''                        nodes.push(MathNode::Delimited {
                            open: "(",
                            close: ")".into(),
                            body: Box::new(a),
                        });''', '''                        nodes.push(MathNode::Delimited {
                            open: "(".to_string(),
                            close: ")".to_string(),
                            body: Box::new(a),
                        });''')
# E0382: parse_command match name moved — name.as_str() borrowed then name used in fallthrough
s = s.replace('''fn parse_command(
    name: String,
    toks: &[LTok],
    pos: &mut usize,
    st: &mut PState,
) -> PResult<Option<MathNode>> {
    match name.as_str() {''', '''fn parse_command(
    name: String,
    toks: &[LTok],
    pos: &mut usize,
    st: &mut PState,
) -> PResult<Option<MathNode>> {
    let name_borrow: &str = &name;
    match name_borrow {''')
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('ast fixed')

# clipboard/mod.rs: CachedFormula derive Eq removed + parse_any return unwrap
p = 'math/src/clipboard/mod.rs'
s = io.open(p, encoding='utf-8').read()
s = s.replace('''/// Kết quả chuyển đổi cache theo chuỗi nguồn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedFormula {''', '''/// Kết quả chuyển đổi cache theo chuỗi nguồn.
#[derive(Debug, Clone, PartialEq)]
pub struct CachedFormula {''')
s = s.replace('''        // Parse + convert (budget < 2 ms p99 cho payload ≤ 2 KB)
        let node = match crate::ast::parse_any(&ev.text, hint) {
            n => n,
        };
        let latex = crate::converter::to_latex(&node);''', '''        // Parse + convert (budget < 2 ms p99 cho payload ≤ 2 KB)
        let node = crate::ast::parse_any(&ev.text, hint);
        let latex = crate::converter::to_latex(&node);''')
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('clipboard mod fixed')

# lib.rs: ClipboardBundle export + format string braces
p = 'math/src/lib.rs'
s = io.open(p, encoding='utf-8').read()
s = s.replace('pub use lexicon::MathEntry;', 'pub use clipboard::ClipboardBundle;\npub use lexicon::MathEntry;')
# format string "\\alph"... line 419 assert with braces? find "{saved}" missing arg -> the drain test uses got {saved} WITH arg; find the failing one
s = s.replace('assert!(e.word_len() <= 64, "word cap 64");', 'assert!(e.word_len() <= 64, "word cap 64");')
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('lib fixed')
