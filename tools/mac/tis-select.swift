// SPDX-License-Identifier: GPL-3.0-or-later
// tools/mac/tis-select.swift — ARM-M2: đăng ký + bật + chọn input source TextVN qua
// Text Input Sources (TIS) để harness IMK (`smoke-imk.sh`) gõ thật trên runner/máy Mac.
//
// Dùng: swift tools/mac/tis-select.swift <đường dẫn TextVN-IM.app> <bundle id>
//   vd: swift tools/mac/tis-select.swift "$HOME/Library/Input Methods/TextVN-IM.app" vn.textvn.im
//
// Exit: 0 = input source của bundle đang được chọn; 1 = TIS từ chối/không thấy nguồn
// (lỗi THẬT — harness không được coi là SKIP); 2 = sai cú pháp.
// Không đọc/ghi nội dung gõ (G8) — chỉ in ID nguồn nhập.

import Carbon
import Foundation

func fail(_ msg: String, code: Int32 = 1) -> Never {
    FileHandle.standardError.write(Data("[tis-select] FAIL: \(msg)\n".utf8))
    exit(code)
}

func stringProperty(_ source: TISInputSource, _ key: CFString) -> String? {
    guard let raw = TISGetInputSourceProperty(source, key) else { return nil }
    return Unmanaged<CFString>.fromOpaque(raw).takeUnretainedValue() as String
}

func boolProperty(_ source: TISInputSource, _ key: CFString) -> Bool {
    guard let raw = TISGetInputSourceProperty(source, key) else { return false }
    return CFBooleanGetValue(Unmanaged<CFBoolean>.fromOpaque(raw).takeUnretainedValue())
}

let args = CommandLine.arguments
guard args.count == 3 else {
    fail("cú pháp: swift tis-select.swift <TextVN-IM.app> <bundle id>", code: 2)
}
let appPath = args[1]
let bundleID = args[2]
guard FileManager.default.fileExists(atPath: appPath) else {
    fail("không thấy bundle \(appPath)")
}

let registerStatus = TISRegisterInputSource(URL(fileURLWithPath: appPath) as CFURL)
print("[tis-select] TISRegisterInputSource(\(appPath)) = \(registerStatus)")
if registerStatus != noErr {
    fail("TISRegisterInputSource trả \(registerStatus)")
}

func allSources() -> [TISInputSource] {
    guard let list = TISCreateInputSourceList(nil, true)?.takeRetainedValue() else { return [] }
    return (list as NSArray).map { $0 as! TISInputSource }
}

/// Nguồn của bundle: lọc theo `kTISPropertyBundleID`, dự phòng theo tiền tố ID nguồn
/// (`<bundle>` hoặc `vn.textvn.`) vì vài bản macOS chưa gán BundleID cho mode vừa đăng ký.
func sources() -> [TISInputSource] {
    let filter = [kTISPropertyBundleID as String: bundleID] as CFDictionary
    if let list = TISCreateInputSourceList(filter, true)?.takeRetainedValue(),
       (list as NSArray).count > 0 {
        return (list as NSArray).map { $0 as! TISInputSource }
    }
    return allSources().filter { src in
        let id = stringProperty(src, kTISPropertyInputSourceID) ?? ""
        return id.hasPrefix(bundleID) || id.hasPrefix("vn.textvn.")
    }
}

// Nguồn vừa đăng ký có thể xuất hiện trễ trong danh sách hệ thống (đo trên runner: > 5 s).
var found = sources()
var tries = 0
while found.isEmpty && tries < 120 {
    Thread.sleep(forTimeInterval: 0.25)
    found = sources()
    tries += 1
}
if found.isEmpty {
    // Chẩn đoán: hệ TIS có hoạt động không (có nguồn hệ thống) và có gì mang tên TextVN.
    let all = allSources()
    print("[tis-select] chẩn đoán: \(all.count) input source trong hệ thống")
    for src in all {
        let id = stringProperty(src, kTISPropertyInputSourceID) ?? "?"
        let bid = stringProperty(src, kTISPropertyBundleID) ?? "-"
        if id.lowercased().contains("textvn") || bid.lowercased().contains("textvn") {
            print("[tis-select]   liên quan: \(id) (bundle \(bid))")
        }
    }
    let sample = all.prefix(5).map { stringProperty($0, kTISPropertyInputSourceID) ?? "?" }
    print("[tis-select]   mẫu nguồn hệ thống: \(sample.joined(separator: ", "))")
    fail("TIS không có input source nào cho bundle \(bundleID) sau 30 s kể từ khi đăng ký")
}

var selected = false
for source in found {
    let id = stringProperty(source, kTISPropertyInputSourceID) ?? "?"
    if !boolProperty(source, kTISPropertyInputSourceIsEnabled) {
        let st = TISEnableInputSource(source)
        print("[tis-select] enable \(id) = \(st)")
        if st != noErr {
            fail("TISEnableInputSource(\(id)) trả \(st)")
        }
    }
    if !selected && boolProperty(source, kTISPropertyInputSourceIsSelectCapable) {
        let st = TISSelectInputSource(source)
        print("[tis-select] select \(id) = \(st)")
        if st == noErr {
            selected = true
        }
    }
}
if !selected {
    fail("không chọn được input source nào của \(bundleID) (không nguồn nào select-capable)")
}

// Xác minh nguồn hiện hành thật sự là của TextVN (TISSelect có thể trả noErr mà không đổi).
Thread.sleep(forTimeInterval: 0.5)
let current = TISCopyCurrentKeyboardInputSource().takeRetainedValue()
let currentBundle = stringProperty(current, kTISPropertyBundleID) ?? "?"
let currentID = stringProperty(current, kTISPropertyInputSourceID) ?? "?"
print("[tis-select] nguồn hiện hành: \(currentID) (bundle \(currentBundle))")
if currentBundle != bundleID {
    fail("nguồn hiện hành là \(currentBundle), không phải \(bundleID)")
}
print("[tis-select] OK")
