// SPDX-License-Identifier: GPL-3.0-or-later
//! L9 — lặp từ ở ranh giới câu ("... đi học. Học xong.").
//!
//! **VÔ HIỆU HOÁ từ vòng 32 (audit sâu 2026-10-25)**: luật cũ bêu "lặp" của
//! một từ chức năng ở cuối câu trước + đầu câu sau — nhưng trong tiếng Việt
//! đó là văn phạm ĐÚNG và rất phổ biến trong thư từ công sở:
//!
//! - "Em gửi báo cáo cho anh. **Anh** xem giúp em nhé." (đại từ làm chủ ngữ
//!   câu mới)
//! - "Tôi không. **Không** sao cả." / "Cô ấy cũng. **Cũng** vậy."
//!
//! Toàn bộ từ chức năng có thể đứng tự nhiên ở cả 2 vị trí ⇒ luật sinh FP
//! dồi dào trong khi corpus lỗi thật (49 câu, P4-12) **không có** row nào
//! đo được recall — đánh đổi sai chiều bar FPR=0 ("không bao giờ bêu chữ
//! đúng"). `IssueKind::SentenceRepeat` giữ nguyên cho ABI/kind-table (mọi
//! nền tảng map kind theo số); rule chỉ ngừng phát issue. Nếu sau này có
//! dữ liệu lỗi thật cho dạng này, bật lại bằng closed-list riêng.

use super::Ctx;
use crate::{Issue, IssueKind};

/// Kiểm tra lặp từ ở ranh giới câu — **no-op** (xem doc module).
pub fn check(_ctx: &Ctx, _out: &mut Vec<Issue>) {
    // Vô hiệu hoá — lý do ở doc module (vòng 32: FPR trên văn phạm đúng).
    // Chữ ký giữ nguyên để dispatch trong lib.rs không đổi; `IssueKind`
    // được tham chiếu ở đây để cargo không báo unused-import trên enum ABI.
    let _ = std::mem::discriminant(&IssueKind::SentenceRepeat);
}

#[cfg(test)]
mod tests {
    use crate::check_sentence;

    /// Vòng 32: văn phạm đúng ở ranh giới câu KHÔNG được bêu
    /// (FP từng được enshrined vào corpus — xem doc module).
    #[test]
    fn ranh_gioi_cau_van_pham_dung_khong_bao() {
        for s in [
            "Em gửi báo cáo cho anh. Anh xem giúp em nhé.",
            "Tôi không. Không sao cả.",
            "Cô ấy cũng. Cũng vậy.",
            "Tôi đã gọi rồi. Rồi sao nữa?",
        ] {
            assert!(
                check_sentence(s).is_empty(),
                "\"{s}\" là tiếng Việt đúng — không được bêu SentenceRepeat"
            );
        }
    }
}
