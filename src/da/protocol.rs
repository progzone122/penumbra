/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2025 Shomy
*/
use crate::da::DA;
use std::io::Error;

pub trait DAProtocol {
    fn upload_da(&mut self) -> Result<bool, Error>;
    fn boot_to(&mut self, addr: u32, data: &[u8]) -> Result<bool, Error>;
    fn send(&mut self, data: u32, datatype: u32) -> Result<bool, Error>;
    fn send_data(&mut self, data: &[u8]) -> Result<bool, Error>;
    fn get_status(&mut self) -> Result<u32, Error>;
}
