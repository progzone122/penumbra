/*
    SPDX-License-Identifier: GPL-3.0-or-later
    SPDX-FileCopyrightText: 2025 Shomy

    Derived from:
    https://github.com/bkerler/mtkclient/blob/main/mtkclient/Library/Hardware/hwcrypto.py
    Original SPDX-License-Identifier: GPL-3.0-or-later
    Original SPDX-FileCopyrightText: 2018–2024 bkerler

    This file remains under the GPL-3.0-or-later license.
    However, as part of a larger project licensed under the AGPL-3.0-or-later,
    the combined work is subject to the networking terms of the AGPL-3.0-or-later,
    as for term 13 of the GPL-3.0-or-later license.
*/
#[async_trait::async_trait(?Send)]
pub trait CryptoIO {
    async fn read32(&mut self, addr: u32) -> u32;
    async fn write32(&mut self, addr: u32, val: u32);
}

pub struct CryptoConfig<'a> {
    pub sej_base: u32,
    pub io: &'a mut dyn CryptoIO,
}

impl<'a> CryptoConfig<'a> {
    pub fn new(sej_base: u32, io: &'a mut dyn CryptoIO) -> Self {
        Self { sej_base, io }
    }
    pub async fn read32(&mut self, addr: u32) -> u32 {
        self.io.read32(addr).await
    }
    pub async fn write32(&mut self, addr: u32, val: u32) {
        self.io.write32(addr, val).await
    }
}
