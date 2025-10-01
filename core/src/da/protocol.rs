/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2025 Shomy
*/
use crate::connection::Connection;
use crate::connection::port::ConnectionType;
use tokio::io::Error;

#[async_trait::async_trait]
pub trait DAProtocol: Send {
    // Main helpers
    async fn upload_da(&mut self) -> Result<bool, Error>;
    async fn boot_to(&mut self, addr: u32, data: &[u8]) -> Result<bool, Error>;
    async fn send(&mut self, data: u32, datatype: u32) -> Result<bool, Error>;
    async fn send_data(&mut self, data: &[u8]) -> Result<bool, Error>;
    async fn get_status(&mut self) -> Result<u32, Error>;
    // FLASH operations
    // fn read_partition(&mut self, name: &str) -> Result<Vec<u8>, Error>;
    async fn read_flash(
        &mut self,
        addr: u64,
        size: usize,
        progress: &mut (dyn FnMut(usize, usize) + Send),
    ) -> Result<Vec<u8>, Error>;

    async fn write_flash(
        &mut self,
        addr: u64,
        size: usize,
        data: &[u8],
        progress: &mut (dyn FnMut(usize, usize) + Send),
    ) -> Result<(), Error>;

    // Memory
    async fn read32(&mut self, addr: u32) -> Result<u32, Error>;
    async fn write32(&mut self, addr: u32, value: u32) -> Result<(), Error>;

    async fn get_usb_speed(&mut self) -> Result<u32, Error>;
    // fn set_usb_speed(&mut self, speed: u32) -> Result<(), Error>;

    // Connection
    fn get_connection(&self) -> &Connection;
    fn set_connection_type(&mut self, conn_type: ConnectionType) -> Result<(), Error>;
}
