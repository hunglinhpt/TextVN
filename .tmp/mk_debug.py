import io
content = r'''// SPDX-License-Identifier: GPL-3.0-or-later
//! Scratch debug - xoa sau khi chan doan xong.
use textvn_math::ast;

#[test]
fn debug_left_right() {
    let r = ast::parse_latex("\left(a+b\right)");
    println!("RESULT: {r:?}");
    let r3 = ast::parse_latex("\alpha + \beta");
    println!("RESULT3: {r3:?}");
    let r4 = ast::parse_latex("x^2 + y_1");
    println!("RESULT4: {r4:?}");
    let r5 = ast::parse_latex("12.5 + 3");
    println!("RESULT5: {r5:?}");
    let r6 = ast::parse_chem("SO_4^{2-}");
    println!("RESULT6: {r6:?}");
    let r7 = ast::parse_mathml("<math><mtable><mtr><mtd><mi>a</mi></mtd><mtd><mi>b</mi></mtd></mtr></mtable></math>");
    println!("RESULT7: {r7:?}");
}
'''
io.open('math/tests/debug_left.rs', 'w', encoding='utf-8', newline='\n').write(content)
print('written')
