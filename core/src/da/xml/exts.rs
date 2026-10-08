/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2025-2026 Shomy
*/
use std::io::{Cursor, Read, Write};

use log::{debug, info};
use xmlcmd_derive::XmlCommand;

use crate::core::storage::{RPMB_FRAME_DATA_SZ, RpmbRegion, Storage, StorageType};
use crate::da::DownloadProtocol;
use crate::da::xml::Xml;
use crate::da::xml::cmds::{XmlCmdLifetime, XmlCommand};
use crate::da::xml::patch::to_arch;
use crate::error::{Error, Result};
use crate::exploit::get_v6_payload;
use crate::utilities::analysis::create_analyzer;
use crate::utilities::patching::{bytes_to_hex, patch_pattern_str};
use crate::utilities::xml::get_tag;

const DA_EXT: &[u8] = include_bytes!("../../../payloads/da_xml.bin");

#[derive(XmlCommand)]
pub struct ExtAck;

#[derive(XmlCommand)]
pub struct ExtDaCtx {
    #[xml(tag = "sej_base", fmt = "0x{sej_base:X}")]
    sej_base: u32,
    #[xml(tag = "tzcc_base", fmt = "0x{tzcc_base:X}")]
    tzcc_base: u32,
    #[xml(tag = "ssr_base", fmt = "0x{ssr_base:X}")]
    ssr_base: u32,
    #[xml(tag = "da2_base", fmt = "0x{da2_base:X}")]
    da2_base: u32,
    #[xml(tag = "da2_size", fmt = "0x{da2_size:X}")]
    da2_size: u32,
    #[xml(tag = "storage")]
    storage: String,
    #[xml(tag = "usb_log")]
    usb_log: String,
}

#[derive(XmlCommand)]
pub struct ExtReadMem {
    #[xml(tag = "address", fmt = "0x{address:X}")]
    address: u32,
    #[xml(tag = "length", fmt = "0x{length:X}")]
    length: usize,
}

#[derive(XmlCommand)]
pub struct ExtWriteMem {
    #[xml(tag = "address", fmt = "0x{address:X}")]
    address: u32,
    #[xml(tag = "length", fmt = "0x{length:X}")]
    length: usize,
}

#[derive(XmlCommand)]
pub struct ExtKeyDerive {
    #[xml(tag = "key_type")]
    key_type: String,
}

#[derive(XmlCommand)]
pub struct ExtSej {
    #[xml(tag = "encrypt")]
    encrypt: String,
    #[xml(tag = "legacy")]
    legacy: String,
    #[xml(tag = "ac")]
    anti_clone: String,
    #[xml(tag = "length", fmt = "0x{length:X}")]
    length: u32,
}

#[derive(XmlCommand)]
pub struct ExtRpmbInit {
    #[xml(tag = "partition", fmt = "{partition}")]
    partition: u32,
    #[xml(tag = "key")]
    key: String,
}

#[derive(XmlCommand)]
pub struct ExtRpmbRead {
    #[xml(tag = "partition", fmt = "{partition}")]
    partition: u32,
    #[xml(tag = "start_sector", fmt = "{start_sector}")]
    start_sector: u32,
    #[xml(tag = "sectors_count", fmt = "{sectors_count}")]
    sectors_count: u32,
}

#[derive(XmlCommand)]
pub struct ExtRpmbWrite {
    #[xml(tag = "partition", fmt = "{partition}")]
    partition: u32,
    #[xml(tag = "start_sector", fmt = "{start_sector}")]
    start_sector: u32,
    #[xml(tag = "sectors_count", fmt = "{sectors_count}")]
    sectors_count: u32,
}

pub fn boot_extensions(xml: &mut Xml) -> Result<bool> {
    let ext_data = match prepare_extensions(xml) {
        Some(data) => data,
        None => {
            debug!("Failed to prepare XML extensions. Continuing without.");
            return Ok(false);
        }
    };

    debug!("Trying booting XML extensions...");

    let ext_addr = 0x68000000;
    let ext_size = DA_EXT.len() as u32;

    info!("Uploading XML extensions to 0x{:08X} (0x{:X} bytes)", ext_addr, ext_size);

    let boot_to_resp = xml.boot_to(ext_addr, &ext_data).unwrap_or(false);
    if !boot_to_resp {
        info!("Failed to upload XML extensions, continuing without extensions");
        return Ok(false);
    }

    if xmlcmd!(xml, ExtAck).is_err() {
        info!("Extensions did not reply, continuing without extensions");
        return Ok(false);
    }

    let response = match xml.get_upload_file_resp() {
        Ok(resp) => resp,
        Err(_) => {
            xml.lifetime_ack(XmlCmdLifetime::CmdEnd)?;
            info!("Failed to get extension ack response, continuing without extensions");
            return Ok(false);
        }
    };

    xml.lifetime_ack(XmlCmdLifetime::CmdEnd)?;

    let ack: String = get_tag(&response, "status")?;
    if ack != "OK" {
        info!("DA extensions failed to start: {}", ack);
        return Ok(false);
    }

    let sej_base = xml.chip().sej_base();
    let tzcc_base = xml.chip().tzcc_base();
    let ssr_base = xml.chip().ssr_base();
    let da2_base = xml.da.get_da2().map(|da2| da2.addr).unwrap_or(0);
    let da2_size = xml.da.get_da2().map(|da2| da2.data.len() as u32).unwrap_or(0);
    let storage = match xml.get_storage() {
        Some(s) => match s.kind() {
            StorageType::Emmc => "EMMC",
            StorageType::Ufs => "UFS",
            StorageType::Unknown => "Unknown",
        },
        None => "Unknown",
    };
    let usb_log = if xml.usb_log_channel { "yes" } else { "no" };

    xmlcmd_e!(xml, ExtDaCtx, sej_base, tzcc_base, ssr_base, da2_base, da2_size, storage, usb_log)?;

    info!("Successfully booted XML extensions");

    Ok(true)
}

fn prepare_extensions(xml: &Xml) -> Option<Vec<u8>> {
    let da2address = xml.da.get_da2()?.addr;
    let da2data = &xml.da.get_da2()?.data;

    let is_arm64 = xml.da.is_arm64();
    let mut da_ext_data = get_v6_payload(DA_EXT, is_arm64).ok()?.to_vec();

    let analyzer = create_analyzer(da2data.clone(), da2address as u64, to_arch(is_arm64));

    let off = analyzer.find_string_xref("CMD:REBOOT")?;
    let bl_off = analyzer.get_next_bl_from_off(off)?;
    let reg_cmd_addr = analyzer.get_bl_target(bl_off)? as u32;

    debug!("Reg CMD function at VA 0x{:X}", reg_cmd_addr);

    let off = analyzer.va_to_offset(reg_cmd_addr as u64)?;
    let bl_off = analyzer.get_next_bl_from_off(off)?;
    let malloc_addr = analyzer.get_bl_target(bl_off)? as u32;

    debug!("Malloc function at VA 0x{:X}", malloc_addr);

    let off = analyzer.find_string_xref("Bad %s")?;
    let bl1 = analyzer.get_next_bl_from_off(off)?;
    let bl2 = analyzer.get_next_bl_from_off(bl1 + 4)?;
    let free_addr = analyzer.get_bl_target(bl2)? as u32;

    debug!("Free function at VA 0x{:X}", free_addr);

    let load_string_off = analyzer.find_function_start_from_off(off)?;
    let load_str_addr = analyzer.offset_to_va(load_string_off)? as u32;

    debug!("mxml_load_string function at VA 0x{:X}", load_str_addr);
    let off = analyzer.find_string_xref("runtime_switchable_config/magic")?;
    let bl_off = analyzer.get_next_bl_from_off(off)?;
    let gettext_addr = analyzer.get_bl_target(bl_off)? as u32;

    debug!("gettext function at VA 0x{:X}", gettext_addr);

    let off = analyzer.find_function_from_string("mmc_switch_part")?;
    let bl_off = analyzer.get_next_bl_from_off(off)?;
    let mmc_get_card = analyzer.get_bl_target(bl_off)? as u32;

    debug!("mmc_get_card function at VA 0x{:X}", mmc_get_card);

    let uart_base = xml.chip().uart();

    debug!("UART base address at 0x{:X}", uart_base);

    patch_pattern_str(&mut da_ext_data, "11111111", &bytes_to_hex(&reg_cmd_addr.to_le_bytes()))?;
    patch_pattern_str(&mut da_ext_data, "22222222", &bytes_to_hex(&malloc_addr.to_le_bytes()))?;
    patch_pattern_str(&mut da_ext_data, "33333333", &bytes_to_hex(&free_addr.to_le_bytes()))?;
    patch_pattern_str(&mut da_ext_data, "44444444", &bytes_to_hex(&gettext_addr.to_le_bytes()))?;
    patch_pattern_str(&mut da_ext_data, "55555555", &bytes_to_hex(&load_str_addr.to_le_bytes()))?;
    patch_pattern_str(&mut da_ext_data, "66666666", &bytes_to_hex(&mmc_get_card.to_le_bytes()))?;
    patch_pattern_str(&mut da_ext_data, "00200011", &bytes_to_hex(&uart_base.to_le_bytes()))?;

    Some(da_ext_data)
}

pub fn peek<W, F>(xml: &mut Xml, addr: u32, length: usize, writer: W, progress: F) -> Result<()>
where
    W: Write,
    F: FnMut(usize, usize) + Send,
{
    xmlcmd!(xml, ExtReadMem, addr, length)?;

    xml.upload_file(writer, progress)?;

    xml.lifetime_ack(XmlCmdLifetime::CmdEnd)?;

    Ok(())
}

pub fn poke<R, F>(xml: &mut Xml, addr: u32, length: usize, reader: R, progress: F) -> Result<()>
where
    R: Read,
    F: FnMut(usize, usize) + Send,
{
    xmlcmd!(xml, ExtWriteMem, addr, length)?;

    xml.download_file(length, reader, progress)?;

    xml.lifetime_ack(XmlCmdLifetime::CmdEnd)?;

    Ok(())
}

pub fn sej(
    xml: &mut Xml,
    data: &[u8],
    encrypt: bool,
    legacy: bool,
    anti_clone: bool,
    _xor: bool,
) -> Result<Vec<u8>> {
    let length = data.len() as u32;

    // yes or no
    let encrypt_str = if encrypt { "yes" } else { "no" };
    let legacy_str = if legacy { "yes" } else { "no" };
    let anti_clone_str = if anti_clone { "yes" } else { "no" };
    xmlcmd!(xml, ExtSej, encrypt_str, legacy_str, anti_clone_str, length)?;

    let mut buf = data.to_vec();
    let mut cursor = Cursor::new(&mut buf);
    let progress = |_: usize, _: usize| {};

    xml.download_file(length as usize, &mut cursor, progress)?;
    cursor.set_position(0);
    xml.upload_file(&mut cursor, progress)?;

    xml.lifetime_ack(XmlCmdLifetime::CmdEnd)?;

    Ok(buf)
}

fn init_rpmb(xml: &mut Xml, region: RpmbRegion) -> Result<()> {
    // Derive RPMB key (0 = RPMB)
    xmlcmd!(xml, ExtKeyDerive, "RPMB")?;
    let resp = xml.get_upload_file_resp()?;
    let key: String = get_tag(&resp, "result")?;
    xml.lifetime_ack(XmlCmdLifetime::CmdEnd)?;

    // If the RPMB is already initialized (even with another key), this will succeed
    // without actually changing the key.
    xmlcmd_e!(xml, ExtRpmbInit, region as u32, key)?;

    Ok(())
}

pub fn read_rpmb<W, F>(
    xml: &mut Xml,
    region: RpmbRegion,
    start_sector: u32,
    sectors_count: u32,
    writer: W,
    progress: F,
) -> Result<()>
where
    W: Write + Send,
    F: FnMut(usize, usize) + Send,
{
    init_rpmb(xml, region)?;

    let storage = match xml.get_storage() {
        Some(s) => s,
        None => {
            return Err(Error::penumbra("Failed to get storage information for RPMB read"));
        }
    };

    let rpmb_size = storage.get_rpmb_size();
    let max_sectors = (rpmb_size / RPMB_FRAME_DATA_SZ as u64) as u32;
    if start_sector.checked_add(sectors_count).is_none_or(|end| end > max_sectors) {
        return Err(Error::penumbra("Requested RPMB read range is out of bounds"));
    }

    xmlcmd!(xml, ExtRpmbRead, region as u32, start_sector, sectors_count)?;
    xml.upload_file(writer, progress)?;
    xml.lifetime_ack(XmlCmdLifetime::CmdEnd)?;

    Ok(())
}

pub fn write_rpmb<R, F>(
    xml: &mut Xml,
    region: RpmbRegion,
    start_sector: u32,
    sectors_count: u32,
    reader: R,
    progress: F,
) -> Result<()>
where
    R: Read + Send,
    F: FnMut(usize, usize) + Send,
{
    init_rpmb(xml, region)?;

    let storage = match xml.get_storage() {
        Some(s) => s,
        None => {
            return Err(Error::penumbra("Failed to get storage information for RPMB write"));
        }
    };

    let rpmb_size = storage.get_rpmb_size();
    let max_sectors = (rpmb_size / RPMB_FRAME_DATA_SZ as u64) as u32;
    if start_sector.checked_add(sectors_count).is_none_or(|end| end > max_sectors) {
        return Err(Error::penumbra("Requested RPMB write range is out of bounds"));
    }

    let data_len = sectors_count as usize * RPMB_FRAME_DATA_SZ;

    xmlcmd!(xml, ExtRpmbWrite, region as u32, start_sector, sectors_count)?;
    xml.download_file(data_len, reader, progress)?;
    xml.lifetime_ack(XmlCmdLifetime::CmdEnd)?;

    Ok(())
}

pub fn auth_rpmb(xml: &mut Xml, region: RpmbRegion, key: &[u8]) -> Result<()> {
    let key = bytes_to_hex(key);
    xmlcmd_e!(xml, ExtRpmbInit, region as u32, key)?;

    Ok(())
}
