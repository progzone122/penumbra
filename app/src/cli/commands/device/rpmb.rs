/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2026 Shomy
*/

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use clap::Args;
use log::info;
use penumbra::{Device, MtkPort, RpmbRegion, Storage};

use crate::cli::DeviceCommand;
use crate::cli::common::{CONN_DA, CommandMetadata};
use crate::cli::helpers::AntumbraProgress;
use crate::cli::state::PersistedDeviceState;

#[derive(Debug, Args)]
pub struct RpmbReadArgs {
    /// RPMB region to use.
    #[arg(long, default_value_t = 0)]
    pub region: u8,
    /// Starting sector to read from.
    #[arg(long, default_value_t = 0)]
    pub start_sector: u32,
    /// Number of sectors to read.
    #[arg(short, long)]
    pub num_sectors: Option<u32>,
    /// File to write the read data to.
    pub file: PathBuf,
}

#[derive(Debug, Args)]
pub struct RpmbWriteArgs {
    /// RPMB region to use.
    #[arg(long, default_value_t = 0)]
    pub region: u8,
    /// Starting sector to write to.
    #[arg(long, default_value_t = 0)]
    pub start_sector: u32,
    /// Number of sectors to write.
    #[arg(short, long)]
    pub num_sectors: Option<u32>,
    /// File to read the data from.
    pub file: PathBuf,
}

#[derive(Debug, Args)]
pub struct RpmbEraseArgs {
    /// RPMB region to use.
    #[arg(long, default_value_t = 0)]
    pub region: u8,
    /// Starting sector to erase.
    #[arg(long, default_value_t = 0)]
    pub start_sector: u32,
    /// Number of sectors to erase.
    #[arg(short, long)]
    pub num_sectors: Option<u32>,
}

#[derive(Debug, Args)]
pub struct RpmbAuthArgs {
    /// RPMB region to use.
    #[arg(long, default_value_t = 0)]
    pub region: u8,
    /// The authentication key in hex
    pub key: String,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct RpmbArgs {
    #[command(subcommand)]
    pub command: RpmbCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum RpmbCommand {
    /// Read from RPMB.
    Read(RpmbReadArgs),
    /// Write to RPMB.
    Write(RpmbWriteArgs),
    /// Erase RPMB.
    Erase(RpmbEraseArgs),
    /// Authenticate RPMB.
    Auth(RpmbAuthArgs),
}

impl CommandMetadata for RpmbArgs {
    fn about() -> &'static str {
        "Perform RPMB operations."
    }

    fn long_about() -> &'static str {
        "Perform RPMB operations. DA Extensions must be loaded for this command to work."
    }
}

fn perform_rpmb_io<P: MtkPort>(
    dev: &mut Device<P>,
    region: RpmbRegion,
    start_sector: u32,
    num_sectors: Option<u32>,
    file_path: &PathBuf,
    is_read: bool,
) -> Result<()> {
    let storage =
        dev.get_storage().ok_or_else(|| anyhow!("Failed to retrieve storage information"))?;

    let rpmb_size = storage.get_rpmb_size();
    if rpmb_size == 0 {
        return Err(anyhow!("Device reports 0 RPMB size or RPMB is not supported"));
    }
    let max_sectors = (rpmb_size / 256) as u32;

    let num_sectors = num_sectors.unwrap_or_else(|| max_sectors.saturating_sub(start_sector));
    if start_sector.saturating_add(num_sectors) > max_sectors {
        return Err(anyhow!(
            "RPMB {} out of bounds! Maximum sectors available: {}",
            if is_read { "read" } else { "write" },
            max_sectors
        ));
    }

    info!(
        "{} {} sectors from RPMB starting at sector {} {} {}",
        if is_read { "Reading" } else { "Writing" },
        num_sectors,
        start_sector,
        if is_read { "into" } else { "from" },
        file_path.display()
    );

    let pb = AntumbraProgress::new(num_sectors as u64 * 256);
    let mut progress_callback = pb.get_callback(
        if is_read { "Reading RPMB..." } else { "Writing RPMB..." },
        if is_read { "RPMB Read Complete!" } else { "RPMB Write Complete!" },
    );

    if is_read {
        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);
        dev.read_rpmb(region, start_sector, num_sectors, &mut writer, &mut progress_callback)?;
        writer.flush()?;
    } else {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        dev.write_rpmb(region, start_sector, num_sectors, &mut reader, &mut progress_callback)?;
    }

    Ok(())
}

impl DeviceCommand for RpmbArgs {
    fn run<P: MtkPort>(&self, dev: &mut Device<P>, state: &mut PersistedDeviceState) -> Result<()> {
        dev.enter_da_mode()?;

        state.connection_type = CONN_DA;
        state.flash_mode = 1;

        let region = match &self.command {
            RpmbCommand::Read(args) => RpmbRegion::try_from(args.region).unwrap_or(RpmbRegion::R0),
            RpmbCommand::Write(args) => RpmbRegion::try_from(args.region).unwrap_or(RpmbRegion::R0),
            RpmbCommand::Erase(args) => RpmbRegion::try_from(args.region).unwrap_or(RpmbRegion::R0),
            RpmbCommand::Auth(args) => RpmbRegion::try_from(args.region).unwrap_or(RpmbRegion::R0),
        };

        let rpmb_size = match dev.get_storage() {
            Some(storage) => storage.get_rpmb_size(),
            None => return Err(anyhow!("Failed to retrieve storage information")),
        };

        if rpmb_size == 0 {
            return Err(anyhow!("Device reports 0 RPMB size or RPMB is not supported"));
        }

        let max_sectors = (rpmb_size / 256) as u32;

        match &self.command {
            RpmbCommand::Read(args) => {
                perform_rpmb_io(
                    dev,
                    region,
                    args.start_sector,
                    args.num_sectors,
                    &args.file,
                    true,
                )?;
            }
            RpmbCommand::Write(args) => {
                perform_rpmb_io(
                    dev,
                    region,
                    args.start_sector,
                    args.num_sectors,
                    &args.file,
                    false,
                )?;
            }
            RpmbCommand::Erase(args) => {
                let num_sectors = args
                    .num_sectors
                    .unwrap_or_else(|| max_sectors.saturating_sub(args.start_sector));

                if args.start_sector.saturating_add(num_sectors) > max_sectors {
                    return Err(anyhow!(
                        "RPMB erase out of bounds! Maximum sectors available: {}",
                        max_sectors
                    ));
                }

                info!(
                    "Erasing {} sectors from RPMB starting at sector {}",
                    num_sectors, args.start_sector
                );

                let pb = AntumbraProgress::new(num_sectors as u64 * 256);
                let mut progress_callback =
                    pb.get_callback("Erasing RPMB...", "RPMB Erase Complete!");

                dev.erase_rpmb(region, args.start_sector, num_sectors, &mut progress_callback)?;
            }
            RpmbCommand::Auth(args) => {
                info!("Authenticating RPMB using provided key...");
                let key = hex::decode(&args.key)?;
                dev.auth_rpmb(region, &key)?;
                info!("Authentication was successful!");
            }
        }

        Ok(())
    }
}
