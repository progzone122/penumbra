#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
    // Taken from https://github.com/bkerler/mtkclient/blob/main/mtkclient/Library/DA/xflash/xflash_param.py
    // SPDX-License-Identifier: GPL-3.0-or-later
    // Copyright (C) 2018-2024 bkerler

    Magic = 0xFEEEEEEF,
    SyncSignal = 0x434E5953,

    Unknown = 0x010000,
    Download = 0x010001,
    Upload = 0x010002,
    Format = 0x010003,
    WriteData = 0x010004,
    ReadData = 0x010005,
    FormatPartition = 0x010006,
    Shutdown = 0x010007,
    BootTo = 0x010008,
    DeviceCtrl = 0x010009,
    InitExtRam = 0x01000A,
    SwitchUsbSpeed = 0x01000B,
    ReadOtpZone = 0x01000C,
    WriteOtpZone = 0x01000D,
    WriteEfuse = 0x01000E,
    ReadEfuse = 0x01000F,
    NandBmtRemark = 0x010010,

    SetupEnvironment = 0x010100,
    SetupHwInitParams = 0x010101,

    SetBmtPercentage = 0x020001,
    SetBatteryOpt = 0x020002,
    SetChecksumLevel = 0x020003,
    SetResetKey = 0x020004,
    SetHostInfo = 0x020005,
    SetMetaBootMode = 0x020006,
    SetEmmcHwresetPin = 0x020007,
    SetGenerateGpx = 0x020008,
    SetRegisterValue = 0x020009,
    SetExternalSig = 0x02000A,
    SetRemoteSecPolicy = 0x02000B,
    SetAllInOneSig = 0x02000C,
    SetRscInfo = 0x02000D,
    SetUpdateFw = 0x020010,
    SetUfsConfig = 0x020011,

    GetEmmcInfo = 0x040001,
    GetNandInfo = 0x040002,
    GetNorInfo = 0x040003,
    GetUfsInfo = 0x040004,
    GetDaVersion = 0x040005,
    GetExpireData = 0x040006,
    GetPacketLength = 0x040007,
    GetRandomId = 0x040008,
    GetPartitionTblCata = 0x040009,
    GetConnectionAgent = 0x04000A,
    GetUsbSpeed = 0x04000B,
    GetRamInfo = 0x04000C,
    GetChipId = 0x04000D,
    GetOtpLockStatus = 0x04000E,
    GetBatteryVoltage = 0x04000F,
    GetRpmbStatus = 0x040010,
    GetExpireDate = 0x040011,
    GetDramType = 0x040012,
    GetDevFwInfo = 0x040013,
    GetHrid = 0x040014,
    GetErrorDetail = 0x040015,
    SlaEnabledStatus = 0x040016,

    StartDlInfo = 0x080001,
    EndDlInfo = 0x080002,
    ActLockOtpZone = 0x080003,
    DisableEmmcHwresetPin = 0x080004,
    CcOptionalDownloadAct = 0x080005,
    DaStorLifeCycleCheck = 0x080007,

    UnknownCtrlCode = 0x0E0000,
    CtrlStorageTest = 0x0E0001,
    CtrlRamTest = 0x0E0002,
    DeviceCtrlReadRegister = 0x0E0003,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    ProtocolFlow = 1,
    Message = 2,
}
