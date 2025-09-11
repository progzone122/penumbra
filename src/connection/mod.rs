/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2025 Shomy
*/
mod command;
use crate::connection::command::Command;
use log::{debug, error, info};
// use serialport::{ClearBuffer, SerialPort, SerialPortInfo, SerialPortType};
// use std::io::{Read, Result, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt, Result};
use tokio_serial::{
    SerialPort, SerialPortBuilderExt, SerialPortInfo, SerialPortType, SerialStream,
};

pub const KNOWN_PORTS: &[(u16, u16)] = &[
    (0x0e8d, 0x0003), // Mediatek USB Port (BROM)
    (0x0e8d, 0x2000), // Mediatek USB Port (Preloader)
    (0x0e8d, 0x2001), // Mediatek USB Port (DA)
];

#[derive(Debug, PartialEq)]
pub enum ConnectionType {
    Brom,
    Preloader,
    Da,
}

#[derive(Debug)]
pub struct Connection {
    pub port: SerialStream,
    pub connection_type: ConnectionType,
    pub baudrate: u32,
}

pub fn find_mtk_port() -> Vec<SerialPortInfo> {
    match serialport::available_ports() {
        Ok(ports) => ports
            .into_iter()
            .filter(|p| match &p.port_type {
                SerialPortType::UsbPort(usb_info) => KNOWN_PORTS
                    .iter()
                    .any(|(vid, pid)| usb_info.vid == *vid && usb_info.pid == *pid),
                _ => false,
            })
            .collect(),
        Err(e) => {
            error!("Error listing serial ports: {}", e);
            vec![]
        }
    }
}

pub fn get_mtk_port_connection(serial_port: &SerialPortInfo) -> Option<Connection> {
    let connection_type = match &serial_port.port_type {
        SerialPortType::UsbPort(usb_info) => match (usb_info.vid, usb_info.pid) {
            (0x0e8d, 0x0003) => ConnectionType::Brom,
            (0x0e8d, 0x2000) => ConnectionType::Preloader,
            (0x0e8d, 0x2001) => ConnectionType::Da,
            _ => {
                error!(
                    "Unknown MTK port type: {:04x}:{:04x}",
                    usb_info.vid, usb_info.pid
                );
                return None;
            }
        },
        _ => {
            error!("");
            return None;
        }
    };
    debug!("Detected connection type: {:?}", connection_type);
    let baudrate: u32 = match connection_type {
        ConnectionType::Brom => 115_200,
        ConnectionType::Preloader | ConnectionType::Da => 921_600,
    };

    let port = tokio_serial::new(&serial_port.port_name, baudrate)
        .timeout(std::time::Duration::from_millis(1000))
        .open_native_async()
        .ok()?;

    info!(
        "Opened MTK port: {} with baudrate {}",
        serial_port.port_name, baudrate
    );
    Some(Connection {
        connection_type,
        port,
        baudrate,
    })
}

impl Connection {
    pub async fn write(&mut self, data: &[u8], size: usize) -> Result<Vec<u8>> {
        self.port.write_all(data).await?;
        let mut buf = vec![0u8; size];
        self.port.read_exact(&mut buf).await?;
        Ok(buf)
    }

    pub fn check(&self, data: &[u8], expected_data: &[u8]) -> Result<()> {
        if data == expected_data {
            Ok(())
        } else {
            error!(
                "Data mismatch. Expected: {:x?}, Got: {:x?}",
                expected_data, data
            );
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Data mismatch",
            ))
        }
    }

    pub async fn echo(&mut self, data: &[u8], size: usize) -> Result<()> {
        self.port.write_all(data).await?;
        let mut buf = vec![0u8; size];
        self.port.read_exact(&mut buf).await?;
        return self.check(&buf, data);
    }

    pub async fn handshake(&mut self) -> Result<()> {
        loop {
            self.port.write_all(&[0xA0]).await?;
            let mut response = [0u8; 1];
            match self.port.read_exact(&mut response).await {
                Ok(_) if response[0] == 0x5F => break,
                Ok(_) | Err(_) => {
                    let _ = self.port.clear(tokio_serial::ClearBuffer::Input);
                }
            }
        }

        let first_response = self.write(&[0x0A], 1).await?;
        self.check(&first_response, &[0xF5])?;

        let second_response = self.write(&[0x50], 1).await?;
        self.check(&second_response, &[0xAF])?;

        let third_response = self.write(&[0x05], 1).await?;
        self.check(&third_response, &[0xFA])?;

        info!("Handshake completed!");
        Ok(())
    }

    pub async fn jump_da(&mut self, address: u32) -> Result<()> {
        debug!("Jump to DA at 0x{:08X}", address);

        self.echo(&[Command::JumpDa as u8], 1).await?;
        self.echo(&address.to_le_bytes(), 4).await?;

        let mut status = [0u8; 2];
        self.port.read_exact(&mut status).await?;

        let status_val = u16::from_le_bytes(status);
        if status_val != 0 {
            error!("JumpDA failed with status: {:04X}", status_val);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "JumpDA failed").into());
        }

        Ok(())
    }

    pub async fn send_da(
        &mut self,
        da_data: &[u8],
        da_len: u32,
        address: u32,
        sig_len: u32,
    ) -> Result<()> {
        debug!("Sending DA, size: {}", da_data.len());
        self.echo(&[Command::SendDa as u8], 1).await?;
        self.echo(&address.to_be_bytes(), 4).await?;
        self.echo(&(da_len).to_be_bytes(), 4).await?;
        self.echo(&sig_len.to_be_bytes(), 4).await?;

        let mut status = [0u8; 2];
        self.port.read_exact(&mut status).await?;
        let status_val = u16::from_be_bytes(status);
        debug!("Received status: 0x{:04X}", status_val);

        if status_val != 0 {
            error!("SendDA command failed with status: {:04X}", status_val);
            return Err(
                std::io::Error::new(std::io::ErrorKind::Other, "SendDA command failed").into(),
            );
        }

        self.port.write_all(da_data).await?;

        debug!("DA sent!");

        let mut checksum = [0u8; 2];
        self.port.read_exact(&mut checksum).await?;
        debug!("Received checksum: {:02X}{:02X}", checksum[0], checksum[1]);

        let mut status = [0u8; 2];
        self.port.read_exact(&mut status).await?;

        let status_val = u16::from_be_bytes(status);
        debug!("Received final status: 0x{:04X}", status_val);
        if status_val != 0 {
            error!(
                "SendDA data transfer failed with status: {:04X}",
                status_val
            );
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "SendDA data transfer failed",
            )
            .into());
        }

        Ok(())
    }

    pub async fn get_hw_code(&mut self) -> Result<u32> {
        self.echo(&[Command::GetHwCode as u8], 1).await?;

        let mut hw_code = [0u8; 2];
        let mut status = [0u8; 2];

        self.port.read_exact(&mut hw_code).await?;
        self.port.read_exact(&mut status).await?;

        let status_val = u16::from_le_bytes(status);
        if status_val != 0 {
            error!("GetHwCode failed with status: {:04X}", status_val);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "GetHwCode failed").into());
        }

        Ok(u16::from_le_bytes(hw_code) as u32)
    }

    pub async fn get_hw_sw_ver(&mut self) -> Result<(u16, u16, u16)> {
        self.echo(&[Command::GetHwSwVer as u8], 1).await?;

        let mut hw_sub_code = [0u8; 2];
        let mut hw_ver = [0u8; 2];
        let mut sw_ver = [0u8; 2];
        let mut status = [0u8; 2];

        self.port.read_exact(&mut hw_sub_code).await?;
        self.port.read_exact(&mut hw_ver).await?;
        self.port.read_exact(&mut sw_ver).await?;
        self.port.read_exact(&mut status).await?;

        let status_val = u16::from_le_bytes(status);
        if status_val != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Status is 0x{:04X}", status_val),
            ));
        }

        Ok((
            u16::from_le_bytes(hw_sub_code),
            u16::from_le_bytes(hw_ver),
            u16::from_le_bytes(sw_ver),
        ))
    }

    pub async fn get_soc_id(&mut self) -> Result<Vec<u8>> {
        self.echo(&[Command::GetSocId as u8], 1).await?;

        let mut length_bytes = [0u8; 4];
        self.port.read_exact(&mut length_bytes).await?;
        let length = u32::from_be_bytes(length_bytes) as usize;

        let mut soc_id = vec![0u8; length];
        self.port.read_exact(&mut soc_id).await?;

        let mut status_bytes = [0u8; 2];
        self.port.read_exact(&mut status_bytes).await?;
        let status = u16::from_le_bytes(status_bytes);

        if status != 0 {
            error!("GetSocId failed with status: 0x{:04X}", status);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "GetSocId failed").into());
        }

        Ok(soc_id)
    }

    pub async fn get_meid(&mut self) -> Result<Vec<u8>> {
        self.echo(&[Command::GetMeId as u8], 1).await?;

        let mut length_bytes = [0u8; 4];
        self.port.read_exact(&mut length_bytes).await?;
        let length = u32::from_be_bytes(length_bytes) as usize;

        let mut meid = vec![0u8; length];
        self.port.read_exact(&mut meid).await?;

        let mut status_bytes = [0u8; 2];
        self.port.read_exact(&mut status_bytes).await?;
        let status = u16::from_le_bytes(status_bytes);

        if status != 0 {
            error!("GetMeid failed with status: 0x{:04X}", status);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "GetMeid failed").into());
        }

        Ok(meid)
    }
}
