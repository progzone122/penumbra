/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2025 Roger Ortiz <me@r0rt1z2.com>

    Derived from:
    https://github.com/R0rt1z2/moto-experiments/blob/main/src/commands.py
*/
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    // Commands used by the Preloader / BROM protocol
    GetHwSwVer = 0xFC,
    GetHwCode = 0xFD,
    GetPlVer = 0xFE,
    GetBrVer = 0xFF,

    LegacyWrite = 0xA1,
    LegacyRead = 0xA2,

    I2cInit = 0xB0,
    I2cDeinit = 0xB1,
    I2cWrite8 = 0xB2,
    I2cRead8 = 0xB3,
    I2cSetSpeed = 0xB4,

    PwrInit = 0xC4,
    PwrDeinit = 0xC5,
    PwrRead16 = 0xC6,
    PwrWrite16 = 0xC7,

    Read16 = 0xD0,
    Read32 = 0xD1,
    Write16 = 0xD2,
    Write16NoEcho = 0xD3,
    Write32 = 0xD4,
    JumpDa = 0xD5,
    JumpBl = 0xD6,
    SendDa = 0xD7,
    GetTargetConfig = 0xD8,
    Uart1LogEn = 0xDB,

    SendCert = 0xE0,
    GetMeId = 0xE1,
    SendAuth = 0xE2,
    SlaChallenge = 0xE3,
    GetSocId = 0xE7,

    Zeroization = 0xF0,
    GetPlCap = 0xF1,
}
