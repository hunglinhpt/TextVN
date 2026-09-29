// SPDX-License-Identifier: GPL-3.0-or-later
// main.swift — Entry point for TextVN macOS Menu Bar App (P2-4 §1, PLAN §3.6)

import Cocoa
import TextVNAppLib

let app = NSApplication.shared
let delegate = AppDelegate.shared
app.delegate = delegate
app.setActivationPolicy(.accessory)
_ = NSApplicationMain(CommandLine.argc, CommandLine.unsafeArgv)
