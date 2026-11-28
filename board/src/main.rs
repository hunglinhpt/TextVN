// SPDX-License-Identifier: GPL-3.0-or-later
//! Entry point cho binary `cargo-board` — `cargo board <lệnh>` (alias cargo của
//! `cargo xtask board`, BOARD-1/E83). Toàn bộ logic nằm ở `textvn_board::cli`.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = textvn_board::cli(&args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
