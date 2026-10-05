/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2025-2026 Shomy
*/

#[allow(dead_code)]
pub const CONN_BR: u8 = 0;
#[allow(dead_code)]
pub const CONN_PL: u8 = 1;
pub const CONN_DA: u8 = 2;

/// A trait for providing metadata for CLI commands.
/// This trait can be implemented by command structs to give additional info
pub trait CommandMetadata {
    fn aliases() -> &'static [&'static str] {
        &[]
    }
    fn visible_aliases() -> &'static [&'static str] {
        &[]
    }
    fn about() -> &'static str {
        ""
    }
    fn long_about() -> &'static str {
        ""
    }
    fn hide() -> bool {
        false
    }
}
