use serialport::{SerialPort, SerialPortInfo, SerialPortType, ClearBuffer};
use log::{info, error};
use std::io::{Read, Write, Result};

pub const KNOWN_PORTS: &[(u16, u16)] = &[
    (0x0e8d, 0x0003), // Mediatek USB Port (BROM)
    (0x0e8d, 0x2000), // Mediatek USB Port (Preloader)
    (0x0e8d, 0x2001), // Mediatek USB Port (DA)
];

#[derive(Debug)]
pub enum ConnectionType {
    Brom,
    Preloader,
    Da,
}

pub struct Connection {
    pub port: Box<dyn SerialPort>,
    pub connection_type: ConnectionType,
    pub baudrate: u32,
}

pub fn find_mtk_port() -> Vec<SerialPortInfo> {
    match serialport::available_ports() {
        Ok(ports) => ports.into_iter()
            .filter(|p| match &p.port_type {
                SerialPortType::UsbPort(usb_info) => {
                    KNOWN_PORTS.iter().any(|(vid, pid)| usb_info.vid == *vid && usb_info.pid == *pid)
                },
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
                error!("Unknown MTK port type: {:04x}:{:04x}", usb_info.vid, usb_info.pid);
                return None;
            }
        },
        _ => {
            error!("Port is not a USB port");
            return None;
        }
    };

    let baudrate: u32 = match connection_type {
        ConnectionType::Brom => 115_200,
        ConnectionType::Preloader | ConnectionType::Da => 921_600,
    };

    let port = serialport::new(&serial_port.port_name, baudrate)
        .timeout(std::time::Duration::from_millis(1000))
        .open()
        .ok()?;

    println!(
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
    pub fn write(&mut self, data: &[u8], size: usize) -> Result<Vec<u8>> {
        self.port.write_all(data)?;
        let mut buf = vec![0u8; size];
        self.port.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn check(&self, data: &[u8], expected_data: &[u8]) -> Result<()> {
        if data == expected_data {
            Ok(())
        } else {
            error!("Data mismatch. Expected: {:x?}, Got: {:x?}", expected_data, data);
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Data mismatch"))
        }
    }

    pub fn handshake(&mut self) -> Result<()> {
        loop {
            self.port.write_all(&[0xA0])?;
            let mut response = [0u8; 1];
            match self.port.read_exact(&mut response) {
                Ok(()) if response[0] == 0x5F => break,
                Ok(()) | Err(_) => {
                    let _ = self.port.clear(serialport::ClearBuffer::Input);
                }
            }
        }

        let first_response = self.write(&[0x0A], 1)?;
        self.check(&first_response, &[0xF5])?;

        let second_response = self.write(&[0x50], 1)?;
        self.check(&second_response, &[0xAF])?;

        let third_response = self.write(&[0x05], 1)?;
        self.check(&third_response, &[0xFA])?;

        println!("Handshake completed!");
        Ok(())
    }
}