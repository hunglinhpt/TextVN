// SPDX-License-Identifier: GPL-3.0-or-later
//! main.swift — TextVN-IM.app entry (P2-1 §3).
//!
//! IMK server: system spawn khi user chọn input source; controller class khai
//! trong Resources/Info.plist (`InputMethodServerControllerClass`). Không đăng
//! ký observer nào ngoài IMK (P2-1 §3 — tránh retain cycle giữ process sống).

import AppKit
import IMKLib
import InputMethodKit

Diagnostics.start()
Diagnostics.log("TextVN-IM launch (pid \(ProcessInfo.processInfo.processIdentifier))")

let app = NSApplication.shared
// IMK process không hiện Dock icon; không activate chính nó.
app.setActivationPolicy(.prohibited)

let server = IMKServer(
    name: "TextVN-IM_Connection",        // = Info.plist InputMethodConnectionName
    bundleIdentifier: "vn.textvn.im"
)
if server == nil {
    Diagnostics.log("IMKServer init failed — exiting")
    Diagnostics.stop()
    exit(1)
}

// CRashReport engine hook: counter cho status item (P2-1 §12).
NSSetUncaughtExceptionHandler { exception in
    Diagnostics.recordCrash()
    Diagnostics.log("uncaught exception: \(exception.name.rawValue)")
}

app.run()
Diagnostics.stop()
