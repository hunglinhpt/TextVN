// SPDX-License-Identifier: GPL-3.0-or-later
#![windows_subsystem = "windows"]
//! Tiny 32-bit test host for the TextVN x86 TIP (`textvn-tsf-x86.dll` — Zalo/Office 32-bit):
//! one multiline RichEdit50W (TSF-aware, the path Chromium/Zalo take) or EDIT (IMM32 via
//! CUAS — WinForms TextBox / Delphi TEdit family) control. `x86probe.exe rich|edit`.
//! Build: `cargo build --release --target i686-pc-windows-msvc --manifest-path
//! tools/win/x86probe/Cargo.toml` — used by `tools/win/e2e-assist-apps.ps1 -X86Probe`.
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::*;

static mut EDIT: isize = 0;

fn main() -> Result<()> {
    unsafe {
        let mode = std::env::args().nth(1).unwrap_or_else(|| "rich".into());
        let class_name: PCWSTR = if mode == "edit" {
            w!("EDIT")
        } else {
            let _ = LoadLibraryW(w!("msftedit.dll"));
            w!("RICHEDIT50W")
        };
        let hinst = GetModuleHandleW(None)?;
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: hinst.into(),
            lpszClassName: w!("TextVNX86Probe"),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        };
        RegisterClassW(&wc);
        let title = if mode == "edit" { w!("TextVN x86 probe EDIT") } else { w!("TextVN x86 probe RICH") };
        let main = CreateWindowExW(
            WINDOW_EX_STYLE(0), w!("TextVNX86Probe"), title,
            WS_OVERLAPPEDWINDOW | WS_VISIBLE, 200, 200, 1000, 500,
            None, None, Some(hinst.into()), None,
        )?;
        let style = WS_CHILD.0 | WS_VISIBLE.0 | WS_VSCROLL.0 | (ES_MULTILINE as u32) | (ES_AUTOVSCROLL as u32);
        let edit = CreateWindowExW(
            WINDOW_EX_STYLE(0), class_name, w!(""), WINDOW_STYLE(style),
            10, 10, 960, 420, Some(main), None, Some(hinst.into()), None,
        )?;
        let font = CreateFontW(-28, 0, 0, 0, 400, 0, 0, 0, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, DEFAULT_PITCH.0 as u32, w!("Segoe UI"));
        SendMessageW(edit, WM_SETFONT, Some(WPARAM(font.0 as usize)), Some(LPARAM(1)));
        EDIT = edit.0 as isize;
        let _ = SetFocus(Some(edit));
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}

extern "system" fn wndproc(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match m {
            WM_SETFOCUS => {
                let _ = SetFocus(Some(HWND(EDIT as *mut _)));
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(h, m, w, l),
        }
    }
}
