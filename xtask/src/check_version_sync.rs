// SPDX-License-Identifier: GPL-3.0-or-later
//! `cargo xtask check-version-sync` — mọi chỗ còn phải ghi version bằng tay phải
//! khớp `[workspace.package] version` trong Cargo.toml (review R3 F3-10 + blocker 1).
//!
//! Các file dưới đây không đọc được Cargo.toml lúc build/runtime (plist bundle
//! macOS, fallback Swift khi chạy `swift test`, manifest Win32, component IBus,
//! cask Homebrew, AppStream, fallback `.iss`). Bump chỉ Cargo.toml mà quên chúng
//! → artifact tên vX nhưng app/handshake báo vY. PE VersionInfo (`*.rc`) không
//! nằm trong danh sách: `build.rs` đã patch từ `CARGO_PKG_VERSION`.

use std::path::Path;

/// Một chỗ ghi version: tìm `anchor`, rồi lấy chuỗi giữa `open` … `close` đầu
/// tiên sau anchor. `suffix` nối vào version mong đợi (manifest Win32 = 4 số).
struct Site {
    path: &'static str,
    anchor: &'static str,
    open: &'static str,
    close: &'static str,
    suffix: &'static str,
}

const fn site(
    path: &'static str,
    anchor: &'static str,
    open: &'static str,
    close: &'static str,
) -> Site {
    Site {
        path,
        anchor,
        open,
        close,
        suffix: "",
    }
}

const PLIST_KEY: &str = "<key>CFBundleShortVersionString</key>";
const SWIFT_KEY: &str = "\"CFBundleShortVersionString\")";

const SITES: &[Site] = &[
    site(
        "packaging/macos/Info-App.plist",
        PLIST_KEY,
        "<string>",
        "</string>",
    ),
    site(
        "packaging/macos/Info-IM.plist",
        PLIST_KEY,
        "<string>",
        "</string>",
    ),
    site(
        "adapters/macos-imk/Resources/Info.plist",
        PLIST_KEY,
        "<string>",
        "</string>",
    ),
    site(
        "adapters/macos-app/Sources/TextVNAppLib/AppInfo.swift",
        SWIFT_KEY,
        "?? \"",
        "\"",
    ),
    site(
        "adapters/macos-app/Sources/TextVNAppLib/IpcServer.swift",
        SWIFT_KEY,
        "?? \"",
        "\"",
    ),
    site(
        "adapters/macos-imk/Sources/IMKLib/IpcClient.swift",
        SWIFT_KEY,
        "?? \"",
        "\"",
    ),
    site(
        "installer/windows/TextVN-setup.iss",
        "#define MyAppVersion",
        "\"",
        "\"",
    ),
    Site {
        suffix: ".0",
        ..site(
            "installer/windows/app.manifest",
            "<assemblyIdentity",
            "version=\"",
            "\"",
        )
    },
    Site {
        suffix: ".0",
        ..site(
            "tray/tray.manifest",
            "<assemblyIdentity",
            "version=\"",
            "\"",
        )
    },
    Site {
        suffix: ".0",
        ..site("cli/cli.manifest", "<assemblyIdentity", "version=\"", "\"")
    },
    Site {
        suffix: ".0",
        ..site(
            "adapters/windows-hook/hook.manifest",
            "<assemblyIdentity",
            "version=\"",
            "\"",
        )
    },
    site(
        "packaging/linux/ibus/textvn.xml",
        "<component>",
        "<version>",
        "</version>",
    ),
    site(
        "packaging/homebrew/textvn.rb",
        "cask \"textvn\"",
        "version \"",
        "\"",
    ),
    // Bản release mới nhất đứng đầu <releases>.
    site(
        "packaging/linux/appstream/io.github.hunglinhpt.textvn.metainfo.xml",
        "<releases>",
        "version=\"",
        "\"",
    ),
];

/// `version = "x.y.z"` đầu tiên ở đầu dòng = `[workspace.package]`.
fn workspace_version(cargo_toml: &str) -> Option<&str> {
    cargo_toml.lines().find_map(|l| {
        l.strip_prefix("version = \"")
            .and_then(|rest| rest.split('"').next())
            .filter(|v| !v.is_empty())
    })
}

fn extract<'a>(text: &'a str, anchor: &str, open: &str, close: &str) -> Option<&'a str> {
    let after_anchor = &text[text.find(anchor)? + anchor.len()..];
    let after_open = &after_anchor[after_anchor.find(open)? + open.len()..];
    Some(after_open[..after_open.find(close)?].trim())
}

/// Kiểm 1 file (thuần — test không đụng filesystem). `None` = khớp.
fn check_site(site: &Site, text: &str, version: &str) -> Option<String> {
    let expected = format!("{version}{}", site.suffix);
    match extract(text, site.anchor, site.open, site.close) {
        Some(found) if found == expected => None,
        Some(found) => Some(format!(
            "{}: version `{found}` ≠ `{expected}` (Cargo.toml)",
            site.path
        )),
        None => Some(format!(
            "{}: không tìm thấy version sau `{}`",
            site.path, site.anchor
        )),
    }
}

pub fn run() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("không xác định được thư mục gốc workspace")?;
    let read =
        |rel: &str| std::fs::read_to_string(root.join(rel)).map_err(|e| format!("đọc {rel}: {e}"));
    let cargo = read("Cargo.toml")?;
    let version = workspace_version(&cargo).ok_or("Cargo.toml: thiếu `version = \"…\"`")?;

    let mut problems = Vec::new();
    for s in SITES {
        match read(s.path) {
            Ok(text) => problems.extend(check_site(s, &text, version)),
            Err(e) => problems.push(e),
        }
    }
    if problems.is_empty() {
        println!(
            "ok  {} chỗ ghi version khớp Cargo.toml = {version}",
            SITES.len()
        );
        Ok(())
    } else {
        Err(format!(
            "lệch version ({}):\n  {}",
            problems.len(),
            problems.join("\n  ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_version_lay_dong_dau_tien() {
        let toml = "[workspace.package]\nedition = \"2021\"\nversion = \"0.2.0\"\n\n[dependencies]\nserde = { version = \"1\" }\n";
        assert_eq!(workspace_version(toml), Some("0.2.0"));
        assert_eq!(workspace_version("[package]\nname = \"x\"\n"), None);
    }

    #[test]
    fn plist_va_manifest_bo_qua_version_xml() {
        let plist = "<?xml version=\"1.0\"?><dict><key>CFBundleShortVersionString</key>\n\t<string>0.2.0</string></dict>";
        assert_eq!(check_site(&SITES[0], plist, "0.2.0"), None);
        assert!(check_site(&SITES[0], plist, "0.3.0").is_some());

        // `<?xml version="1.0"` đứng trước assemblyIdentity không được bắt nhầm.
        let manifest = "<?xml version=\"1.0\"?>\n<assemblyIdentity\n    version=\"0.2.0.0\"\n    name=\"TextVN\"/>";
        let m = SITES
            .iter()
            .find(|s| s.path == "tray/tray.manifest")
            .unwrap();
        assert_eq!(check_site(m, manifest, "0.2.0"), None);
        assert!(check_site(m, &manifest.replace("0.2.0.0", "0.1.0.0"), "0.2.0").is_some());
    }

    #[test]
    fn swift_fallback_iss_va_thieu_anchor() {
        let swift = "// comment\nstatic let v: String =\n    Bundle.main.object(forInfoDictionaryKey: \"CFBundleShortVersionString\") as? String\n    ?? \"0.2.0\"\n";
        assert_eq!(check_site(&SITES[3], swift, "0.2.0"), None);

        let iss = SITES.iter().find(|s| s.path.ends_with(".iss")).unwrap();
        let text = "#ifndef MyAppVersion\n  #define MyAppVersion \"0.1.0\"\n#endif\n";
        let err = check_site(iss, text, "0.2.0").expect("0.1.0 phải bị bắt");
        assert!(err.contains("0.1.0"), "{err}");
        assert!(check_site(iss, "; không có define\n", "0.2.0")
            .unwrap()
            .contains("không tìm thấy"));
    }

    #[test]
    fn repo_hien_tai_dong_bo() {
        run().expect("mọi chỗ ghi version phải khớp Cargo.toml");
    }
}
