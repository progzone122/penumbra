/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2025 Shomy
*/
mod cmds;
mod exts;
pub mod flash;
use crate::connection::Connection;
use crate::connection::ConnectionType;
use crate::core::device::DeviceInfo;
use crate::da::xflash::cmds::*;
use crate::da::xflash::exts::{boot_extensions, read32_ext, write32_ext};
use crate::da::{DAProtocol, DA};
use crate::exploit::carbonara::Carbonara;
use crate::exploit::Exploit;
use log::{debug, info, warn};
use std::cell::RefCell;
use std::io::{Error, ErrorKind, Read, Write};
use std::rc::Rc;
use std::time::Duration;

pub struct XFlash {
    pub conn: Connection,
    pub da: DA,
    pub dev_info: Rc<RefCell<DeviceInfo>>,
    using_exts: bool,
}

impl DAProtocol for XFlash {
    fn upload_da(&mut self) -> Result<bool, Error> {
        let (da1addr, da1length, da1data, da1sig_len) = match self.da.get_da1() {
            Some(da1) => (da1.addr, da1.length, da1.data.clone(), da1.sig_len),
            None => return Err(Error::new(ErrorKind::NotFound, "DA1 region not found")),
        };

        self.upload_stage1(da1addr, da1length, da1data, da1sig_len)
            .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to upload DA1: {}", e)))?;

        let da2 = match self.da.get_da2() {
            Some(da2) => da2.clone(),
            None => return Err(Error::new(ErrorKind::NotFound, "DA2 region not found")),
        };
        let da2addr = da2.addr;

        // TODO: Patch DA2 with Carbonara
        let carbonara_da = Rc::new(RefCell::new(self.da.clone()));
        let mut carbonara = Carbonara::new(carbonara_da, self as &mut dyn DAProtocol);

        let da2data = match Exploit::run(&mut carbonara) {
            Ok(_) => match carbonara.get_patched_da2() {
                Some(patched_da2) => patched_da2.data.clone(),
                None => da2.data,
            },
            Err(_) => da2.data,
        };

        match self.boot_to(da2addr, &da2data) {
            Ok(true) => {
                info!("[Penumbra] Successfully uploaded and executed DA2");
                self.boot_extensions()?;
                Ok(true)
            }
            Ok(false) => Err(Error::new(ErrorKind::Other, "Failed to execute DA2")),
            Err(e) => Err(Error::new(
                ErrorKind::Other,
                format!("Error uploading DA2: {}", e),
            )),
        }
    }

    fn boot_to(&mut self, addr: u32, data: &[u8]) -> Result<bool, Error> {
        info!(
            "[Penumbra] Sending BOOT_TO command to address 0x{:08X} with {} bytes",
            addr,
            data.len()
        );

        self.send_cmd(Cmd::BootTo)?;

        let status = self.get_status()?;
        if status != 0 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("BOOT_TO command failed with status: 0x{:08X}", status),
            ));
        }

        // Addr (LE) | Padding | Length (LE) | Padding
        // 00000040000000002c83050000000000 -> addr=0x4000000, len=0x0005832c
        let mut param = Vec::new();
        param.extend_from_slice(&addr.to_le_bytes());
        param.extend_from_slice(&[0, 0, 0, 0]);
        param.extend_from_slice(&(data.len() as u32).to_le_bytes());
        param.extend_from_slice(&[0, 0, 0, 0]);

        // TODO: Use send_data instead of reconstructing header manually
        let mut hdr = [0u8; 12];
        hdr[0..4].copy_from_slice(&(Cmd::Magic as u32).to_le_bytes());
        hdr[4..8].copy_from_slice(&(DataType::ProtocolFlow as u32).to_le_bytes());
        hdr[8..12].copy_from_slice(&(param.len() as u32).to_le_bytes());

        debug!(
            "[TX] Parameter Header: {:02X?}, Data Length: {}",
            hdr,
            param.len()
        );

        self.conn.port.write_all(&hdr)?;
        self.conn.port.write_all(&param)?;
        self.conn.port.flush()?;

        // We just need to change the data size,
        // so let us just reuse what we've got already ;P
        hdr[8..12].copy_from_slice(&(data.len() as u32).to_le_bytes());
        debug!(
            "[TX] DA2 Data Header: {:02X?}, Data Length: {}",
            hdr,
            data.len()
        );

        self.conn.port.write_all(&hdr)?;

        // Chunks of 1KB
        let chunk_size = 1024;
        let mut pos = 0;
        while pos < data.len() {
            let end = std::cmp::min(pos + chunk_size, data.len());
            self.conn.port.write_all(&data[pos..end])?;
            pos = end;

            if pos % (chunk_size * 20) == 0 && pos > 0 {
                debug!("[TX] Progress: {}/{} bytes sent", pos, data.len());
            }
        }

        self.conn.port.flush()?;
        debug!("[TX] Completed sending {} bytes", data.len());

        self.conn.port.set_timeout(Duration::from_millis(500))?;

        let status = self.get_status()?;
        if status != 0 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("BOOT_TO status1 is not 0: 0x{:08X}", status),
            ));
        }

        // It needs to receive the SYNC signal as well
        let status = self.get_status()?;
        if status != Cmd::SyncSignal as u32 && status != 0 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("BOOT_TO status2 is not SYNC: 0x{:08X}", status),
            ));
        }

        info!("[Penumbra] Successfully booted to DA2");
        Ok(true)
    }

    fn send_data(&mut self, data: &[u8]) -> Result<bool, Error> {
        let mut hdr = [0u8; 12];

        // MAGIC | DataType (1) | Data Length
        hdr[0..4].copy_from_slice(&(Cmd::Magic as u32).to_le_bytes());
        hdr[4..8].copy_from_slice(&(DataType::ProtocolFlow as u32).to_le_bytes());
        hdr[8..12].copy_from_slice(&(data.len() as u32).to_le_bytes());

        debug!(
            "[TX] Data Header: {:02X?}, Data Length: {}",
            hdr,
            data.len()
        );

        self.conn.port.write_all(&hdr)?;

        let mut pos = 0;
        while pos < data.len() {
            let end = std::cmp::min(pos + 64, data.len());
            let chunk = &data[pos..end];
            debug!("[TX] Sending chunk ({} bytes): {:02X?}", chunk.len(), chunk);
            self.conn.port.write_all(chunk)?;
            pos += chunk.len();
        }

        self.conn.port.flush()?;

        let status = self.get_status()?;
        if status != 0 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Data send failed with status: 0x{:08X}", status),
            ));
        }

        Ok(true)
    }

    fn get_status(&mut self) -> Result<u32, Error> {
        let mut hdr = [0u8; 12];
        self.conn.port.read_exact(&mut hdr)?;

        let magic = u32::from_le_bytes(hdr[0..4].try_into().unwrap());
        let len = u32::from_le_bytes(hdr[8..12].try_into().unwrap());

        if magic != Cmd::Magic as u32 {
            return Err(Error::new(ErrorKind::Other, "Invalid magic"));
        }

        let mut data = vec![0u8; len as usize];
        self.conn.port.read_exact(&mut data)?;

        let status = match len {
            2 => u16::from_le_bytes(data[0..2].try_into().unwrap()) as u32,
            4 => {
                let val = u32::from_le_bytes(data[0..4].try_into().unwrap());
                if val == Cmd::Magic as u32 {
                    0
                } else {
                    val
                }
            }
            _ if data.len() >= 4 => u32::from_le_bytes(data[0..4].try_into().unwrap()),
            _ if !data.is_empty() => data[0] as u32,
            _ => 0xFFFFFFFF,
        };

        debug!("[RX] Status: 0x{:08X}", status);
        Ok(status)
    }

    fn send(&mut self, data: u32, datatype: u32) -> Result<bool, Error> {
        let data_bytes = data.to_le_bytes();

        let mut hdr = [0u8; 12];

        // efeeeefe | 010000000 | 04000000
        hdr[0..4].copy_from_slice(&(Cmd::Magic as u32).to_le_bytes());
        hdr[4..8].copy_from_slice(&(datatype as u32).to_le_bytes());
        hdr[8..12].copy_from_slice(&4u32.to_le_bytes());

        debug!("[TX] Header: {:02X?}, Payload: 0x{:08X}", hdr, data);

        self.conn.port.write_all(&hdr)?;
        self.conn.port.write_all(&data_bytes)?;

        self.conn.port.flush()?;

        Ok(true)
    }

    fn read_flash(&mut self, addr: u64, size: usize) -> Result<Vec<u8>, Error> {
        flash::read_flash(self, addr, size)
    }

    fn write_flash(&mut self, addr: u64, size: usize, data: &[u8]) -> Result<(), Error> {
        flash::write_flash(self, addr, size, data)
    }

    fn get_usb_speed(&mut self) -> Result<u32, Error> {
        let usb_speed = self.devctrl(Cmd::GetUsbSpeed, None)?;
        let status = self.get_status()?;
        if status != 0 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Device returned error status: {:#X}", status),
            ));
        }
        debug!("USB Speed Data: {:?}", usb_speed);
        Ok(u32::from_le_bytes(usb_speed[0..4].try_into().unwrap()))
    }

    fn get_connection(&self) -> &Connection {
        &self.conn
    }

    fn set_connection_type(&mut self, conn_type: ConnectionType) -> Result<(), Error> {
        self.conn.connection_type = conn_type;
        Ok(())
    }

    fn read32(&mut self, addr: u32) -> Result<u32, Error> {
        if self.using_exts {
            return read32_ext(self, addr);
        }
        debug!("Reading 32-bit register at address 0x{:08X}", addr);
        let param = addr.to_le_bytes();
        let resp = self.devctrl(Cmd::DeviceCtrlReadRegister, Some(&param))?;
        debug!("[RX] Read Register Response: {:02X?}", resp);
        if resp.len() < 4 {
            debug!("Short read: expected 4 bytes, got {}", resp.len());
            return Err(Error::new(std::io::ErrorKind::Other, "Short register read"));
        }
        Ok(u32::from_le_bytes(resp[0..4].try_into().unwrap()))
    }

    fn write32(&mut self, addr: u32, value: u32) -> Result<(), Error> {
        if self.using_exts {
            return write32_ext(self, addr, value);
        }
        let mut param = Vec::new();
        param.extend_from_slice(&addr.to_le_bytes());
        param.extend_from_slice(&value.to_le_bytes());
        debug!(
            "[TX] Writing 32-bit value 0x{:08X} to address 0x{:08X}",
            value, addr
        );
        self.devctrl(Cmd::SetRegisterValue, Some(&param))?;
        Ok(())
    }
}

impl XFlash {
    fn send_cmd(&mut self, cmd: Cmd) -> Result<bool, Error> {
        self.send(cmd as u32, DataType::ProtocolFlow as u32)
    }

    pub fn new(conn: Connection, da: DA, dev_info: Rc<RefCell<DeviceInfo>>) -> Self {
        XFlash {
            conn,
            da,
            dev_info,
            using_exts: false,
        }
    }

    fn devctrl(&mut self, cmd: Cmd, param: Option<&[u8]>) -> Result<Vec<u8>, Error> {
        self.send_cmd(Cmd::DeviceCtrl)?;

        let status = self.get_status()?;
        if status != 0 {
            return Err(Error::new(
                ErrorKind::Other,
                format!(
                    "Device control command failed with status: 0x{:08X}",
                    status
                ),
            ));
        }

        self.send_cmd(cmd)?;
        let status = self.get_status()?;
        if status != 0 {
            return Err(Error::new(
                ErrorKind::Other,
                format!(
                    "Device control sub-command failed with status: 0x{:08X}",
                    status
                ),
            ));
        }

        if let Some(p) = param {
            self.send_data(p)?;
            return Ok(Vec::new());
        }

        self.read_data()
    }

    fn read_data(&mut self) -> Result<Vec<u8>, Error> {
        let mut hdr = [0u8; 12];
        self.conn.port.read_exact(&mut hdr)?;

        let magic = u32::from_le_bytes(hdr[0..4].try_into().unwrap());
        let len = u32::from_le_bytes(hdr[8..12].try_into().unwrap());

        if magic != Cmd::Magic as u32 {
            return Err(Error::new(ErrorKind::Other, "Invalid magic"));
        }

        let mut data = vec![0u8; len as usize];
        self.conn.port.read_exact(&mut data)?;

        Ok(data)
    }

    fn upload_stage1(
        &mut self,
        addr: u32,
        length: u32,
        data: Vec<u8>,
        sig_len: u32,
    ) -> Result<bool, Error> {
        info!(
            "[Penumbra] Uploading DA1 region to address 0x{:08X} with length {}",
            addr, length
        );

        self.conn.send_da(&data, length, addr, sig_len)?;
        info!("[Penumbra] Sent DA1, jumping to address 0x{:08X}...", addr);
        self.conn.jump_da(addr)?;

        // Without this, it timed out during my tests, so leave it here for now
        // self.conn.port.set_timeout(Duration::from_secs(10))?;

        let sync_byte = {
            let mut sync_buf = [0u8; 1];
            match self.conn.port.read_exact(&mut sync_buf) {
                Ok(_) => sync_buf[0],
                Err(e) if e.kind() == ErrorKind::TimedOut => {
                    return Err(Error::new(
                        ErrorKind::TimedOut,
                        "Timeout waiting for DA sync byte",
                    ));
                }
                Err(e) => return Err(e),
            }
        };

        info!("[Penumbra] Received sync byte");

        if sync_byte != 0xC0 {
            return Err(Error::new(ErrorKind::Other, "Incorrect sync byte received"));
        }

        self.send_cmd(Cmd::SyncSignal)?;
        self.send_cmd(Cmd::SetupEnvironment)?;

        let mut env_param = Vec::new();
        env_param.extend_from_slice(&2u32.to_le_bytes()); // da_log_level = 2 (UART)
        env_param.extend_from_slice(&1u32.to_le_bytes()); // log_channel = 1
        env_param.extend_from_slice(&1u32.to_le_bytes()); // system_os = 1 (OS_LINUX)
        env_param.extend_from_slice(&0u32.to_le_bytes()); // ufs_provision = 0
        env_param.extend_from_slice(&0u32.to_le_bytes()); // ...

        self.send_data(&env_param)?;
        self.send_cmd(Cmd::SetupHwInitParams)?;
        let hw_param = [0x00, 0x00, 0x00, 0x00];
        self.send_data(&hw_param)?;

        let (magic, dtype, len) = {
            let mut sync_hdr = [0u8; 12];
            match self.conn.port.read_exact(&mut sync_hdr) {
                Ok(_) => {}
                Err(e) => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Failed to read sync header: {}", e),
                    ))
                }
            }

            (
                u32::from_le_bytes(sync_hdr[0..4].try_into().unwrap()),
                u32::from_le_bytes(sync_hdr[4..8].try_into().unwrap()),
                u32::from_le_bytes(sync_hdr[8..12].try_into().unwrap()),
            )
        };

        if magic != Cmd::Magic as u32 || dtype != DataType::ProtocolFlow as u32 || len != 4 {
            return Err(Error::new(ErrorKind::Other, "DA sync header mismatch"));
        }

        let sync_signal_value = {
            let mut sync_signal_buf = [0u8; 4];
            match self.conn.port.read_exact(&mut sync_signal_buf) {
                Ok(_) => {}
                Err(e) => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Failed to read sync payload: {}", e),
                    ))
                }
            }
            u32::from_le_bytes(sync_signal_buf)
        };

        if sync_signal_value != Cmd::SyncSignal as u32 {
            return Err(Error::new(
                ErrorKind::Other,
                "Expected SYNC SIGNAL after setup",
            ));
        }

        info!("[Penumbra] Received DA1 sync signal.");
        Ok(true)
    }

    fn boot_extensions(&mut self) -> Result<bool, Error> {
        if self.using_exts {
            warn!("DA extensions already in use, skipping re-upload");
            return Ok(true);
        }
        info!("Booting DA extensions...");
        boot_extensions(self)?;

        self.using_exts = true;
        Ok(true)
    }
}
