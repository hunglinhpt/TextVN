// SPDX-License-Identifier: GPL-3.0-or-later
//! Tokenizer cho cau tieng Viet (P4-0 §3) - thuan, khong I/O.
//!
//! Offset la **ky tu Unicode** (char), khong phai byte: adapter chuyen sang
//! UTF-16 (TSF) hoac byte khi can.

/// Loai token trong mot cau.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// Tu (chu cai + dau tieng Viet).
    Word,
    /// Khong trang.
    Space,
    /// Dau cau / dau ngoac / dau trich dan.
    Punct,
    /// Ky tu khac (chu so, ky tu la).
    Other,
}

/// Mot token trong cau.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    /// Loai token.
    pub kind: TokenKind,
    /// Offset bat dau (ky tu Unicode, inclusive).
    pub start: usize,
    /// Offset ket thuc (exclusive).
    pub end: usize,
}

impl Token {
    /// Do dai token theo ky tu Unicode.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Token rong hay khong.
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

/// Phan loai mot ky tu.
fn classify(ch: char) -> TokenKind {
    if ch.is_whitespace() {
        TokenKind::Space
    } else if ch.is_alphabetic() {
        TokenKind::Word
    } else if ch.is_numeric() {
        TokenKind::Other
    } else {
        TokenKind::Punct
    }
}

/// Tach cau thanh token.
///
/// - Chu `Word` gom cac ky tu chu cai lien tiep (tieng Viet co dau van la
///   `is_alphabetic`).
/// - Khoang trang gom cac ky tu trang lien tiep thanh mot token.
/// - Ky tu khac (`Other`) tach tung ky tu.
pub fn tokenize(text: &str) -> Vec<Token> {
    tokenize_iter(text.chars())
}

/// Tach mang ky tu `chars` thanh token (tranh duyet UTF-8 2 lan khi caller da co `&[char]`).
pub fn tokenize_chars(chars: &[char]) -> Vec<Token> {
    tokenize_iter(chars.iter().copied())
}

fn tokenize_iter(iter: impl Iterator<Item = char>) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut cur_kind: Option<TokenKind> = None;
    let mut cur_start = 0usize;
    let mut cur_end = 0usize;
    let mut char_idx = 0usize;

    for ch in iter {
        let kind = classify(ch);
        let end = char_idx + 1;
        match cur_kind {
            Some(k) if k == kind && kind != TokenKind::Other => {
                cur_end = end;
            }
            Some(k) => {
                tokens.push(Token {
                    kind: k,
                    start: cur_start,
                    end: cur_end,
                });
                cur_kind = Some(kind);
                cur_start = char_idx;
                cur_end = end;
            }
            None => {
                cur_kind = Some(kind);
                cur_start = char_idx;
                cur_end = end;
            }
        }
        char_idx = end;
    }

    if let Some(k) = cur_kind {
        tokens.push(Token {
            kind: k,
            start: cur_start,
            end: cur_end,
        });
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tach_dung_ba_token() {
        let toks = tokenize("Tôi đi, nhé!");
        let kinds: Vec<TokenKind> = toks.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Word,
                TokenKind::Space,
                TokenKind::Word,
                TokenKind::Punct,
                TokenKind::Space,
                TokenKind::Word,
                TokenKind::Punct,
            ]
        );
    }

    #[test]
    fn offset_la_ky_tu_unicode() {
        // "đ" la 2 byte UTF-8 nhung 1 ky tu Unicode.
        let toks = tokenize("đ");
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].start, 0);
        assert_eq!(toks[0].end, 1);
    }

    #[test]
    fn chu_so_la_token_khac() {
        let toks = tokenize("3.5");
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[0].kind, TokenKind::Other);
        assert_eq!(toks[1].kind, TokenKind::Punct);
        assert_eq!(toks[2].kind, TokenKind::Other);
    }
}
