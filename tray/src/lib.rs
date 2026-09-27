// SPDX-License-Identifier: GPL-3.0-or-later
//! Crate `vietime-tray` — Ứng dụng khay hệ thống, IPC server và điều phối runtime cho VietIME (Windows).
//!
//! Bao gồm các module chức năng:
//! - [`svc`]: Service layer in-process quản lý trạng thái (`state.json`) và cấu hình (`config.json`).
//! - [`ipc_server`]: Named Pipe server `\\.\pipe\vietime-ipc-v1`, broadcast cấu hình và giám sát hook.
//! - [`menu`]: Menu ngữ cảnh khay hệ thống (9 mục chuẩn Win32).
//! - [`autostart`]: Quản lý registry key tự khởi động `HKCU\...\Run\VietIME` (per-user).

pub mod autostart;
pub mod ipc_server;
pub mod menu;
pub mod svc;

pub use autostart::{disable_autostart, enable_autostart, is_autostart_enabled};
pub use ipc_server::IpcServer;
pub use menu::TrayMenu;
pub use svc::{StateData, SvcManager};
