/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2026 Shomy
*/
use std::path::PathBuf;

use anyhow::{Result, anyhow, bail};
use clap::Args;
use clap_num::maybe_hex;
use log::{error, info};
use penumbra::hacc::{Preloader, TryRead};
use penumbra::port::ConnectionType;
use penumbra::{Device, MtkPort, PlProtocol};

use crate::cli::DeviceCommand;
use crate::cli::common::CommandMetadata;
use crate::cli::state::PersistedDeviceState;

#[derive(Args, Debug)]
pub struct BootPlArgs {
    /// Path to the preloader file
    file: PathBuf,
    /// Force the jump address
    #[clap(value_parser=maybe_hex::<u32>)]
    address: Option<u32>,
    /// Send the file as raw data instead of a preloader file
    #[arg(long, default_value_t = false)]
    raw: bool,
}

impl BootPlArgs {
    fn address(&self, file_address: u32) -> Result<u32> {
        match (self.raw, self.address) {
            (true, Some(address)) => Ok(address),
            (true, None) => Err(anyhow!("--address is required when --raw is used")),
            (false, Some(address)) => Ok(address),
            (false, None) => Ok(file_address),
        }
    }
}

impl CommandMetadata for BootPlArgs {
    fn aliases() -> &'static [&'static str] {
        &["plstage"]
    }

    fn about() -> &'static str {
        "Boot a custom preloader in BROM."
    }

    fn long_about() -> &'static str {
        "Temporarily boots a custom preloader in BootROM, without verification.
        Allows to also boot a raw binary at a specified address.
        "
    }
}

impl DeviceCommand for BootPlArgs {
    fn run<P: MtkPort>(
        &self,
        dev: &mut Device<P>,
        _state: &mut PersistedDeviceState,
    ) -> Result<()> {
        if dev.get_connection_type() != ConnectionType::Brom {
            error!("You can only run this command in Brom.");
            error!("Please reboot the device into Brom mode and try again.");
            return Ok(());
        };

        if self.raw && self.address.is_none() {
            bail!("--address is required when --raw is used");
        }

        let data = std::fs::read(self.file.clone())?;
        let (data, address) = if self.raw {
            let address = self.address.unwrap();
            info!("Raw data size: 0x{:X}", data.len());
            (data.as_slice(), address)
        } else {
            let preloader = Preloader::try_read(&data)?;

            let gfh_file_info = preloader.gfh().file_info();
            // Load address is what bootrom uses to load the PL including the GFH header,
            // while jump offset is the offset from the load address where the actual
            // entry point is.
            let pl_jump_addr = gfh_file_info.load_addr() + gfh_file_info.jump_offset();
            let address = self.address(pl_jump_addr)?;

            info!("Parsed preloader content size: 0x{:X}", preloader.content().len());

            (preloader.content(), address)
        };

        let mut pl = PlProtocol::new(dev.port_mut());

        pl.exploit()?;

        info!("Sending data to address 0x{address:08X}...");

        pl.send_da(data, data.len() as u32, address, 0)?;
        pl.jump_da(address)?;

        info!("Jumped to address 0x{address:08X}.");

        Ok(())
    }
}
