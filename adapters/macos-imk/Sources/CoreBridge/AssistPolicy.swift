// SPDX-License-Identifier: GPL-3.0-or-later
//! AssistPolicy — SF-5 (P0-6 §5 assist): luật quyết định của Trợ lý do `textvn-session` quyết
//! (C ABI `ime_assist_*`, G22). Adapter IMK chỉ đưa phím (vk canonical, ch, mods).

import CTextVNFFI
import Foundation

public enum AssistPolicy {
    /// R2-154/155: phím này hẹn lại timer idle 1 s (chữ/dấu câu/xoá khi tiếng Việt bật)?
    public static func idleArms(vk: UInt32, ch: UInt32, mods: UInt32, viEnabled: Bool) -> Bool {
        ime_assist_idle_arms(vk, ch, mods, viEnabled ? 1 : 0) == 1
    }
}
