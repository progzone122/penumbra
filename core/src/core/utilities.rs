/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2025 Shomy
*/

pub fn find_pattern(data: &[u8], to_find: &[u8], offset: usize) -> Option<usize> {
    if data.is_empty() || data.len() < to_find.len() || offset >= data.len() {
        return None;
    }

    data[offset..]
        .windows(to_find.len())
        .position(|chunk| chunk == to_find)
        .map(|index| index + offset)
}
