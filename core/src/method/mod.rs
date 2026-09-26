// SPDX-License-Identifier: GPL-3.0-or-later
//! Bảng phím → action theo từng kiểu gõ (P0-1 §1 `core/src/method/`).
//!
//! Slice 1: Telex đầy đủ. VNI/VIQR/simple_telex = identity (chưa transform) —
//! TODO slice method riêng; corpus `vni_*`/`viqr_*` chưa mở.

pub mod telex;

use crate::transform::DiacriticStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Method {
    #[default]
    Telex,
    Vni,
    Viqr,
    SimpleTelex,
}

/// Fold toàn bộ chuỗi phím của một từ → chuỗi hiển thị.
/// `raw` gồm cả marker đã bị nuốt; kết quả khác `raw` nghĩa là có biến đổi.
pub fn fold(raw: &[char], method: Method, style: DiacriticStyle, free_marking: bool) -> Vec<char> {
    match method {
        Method::Telex => telex::fold(raw, style, free_marking),
        _ => raw.to_vec(),
    }
}
