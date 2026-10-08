/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2025-2026 Shomy
*/

use num_enum::{IntoPrimitive, TryFromPrimitive};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Preloader/Brom error: {0}")]
    BrPl(#[from] BrPlError),
    /// An error related to XFlash protocol (and its error codes)
    #[error("XFlash error: {0}")]
    XFlash(#[from] XFlashError),
    #[error("XML error: {0}")]
    Xml(#[from] XmlError),
    /// Generic Protocol error
    #[error("Protocol Error {0}")]
    Protocol(String),
    /// Connection specific error
    #[error("Connection Error: {0}")]
    Connection(String),
    /// Error related to I/O operations
    /// In particular with the connection backends
    #[error("I/O Error: {0}")]
    Io(String),
    /// Error specific related to timeouts.
    /// Use this preferrably over the generic Io error when
    /// dealing with timeouts, so that we can handle them
    /// separately.
    #[error("Timeout")]
    Timeout,
    /// Generic error that happens in Penumbra, can
    /// be used for anything
    #[error("Penumbra Error: {0}")]
    Penumbra(String),
    /// Error that takes a status code and formats it as hex.
    /// When dealing with statuses in general, use
    /// this, unless a more specific implementation
    /// is there (e.g. XFlash)
    #[error("{ctx}: Status is 0x{status:X}")]
    Status { ctx: String, status: u32 },
    #[error(transparent)]
    Read(#[from] wincode::ReadError),
    #[error(transparent)]
    Write(#[from] wincode::WriteError),
    #[error(transparent)]
    HexDecode(#[from] hex::FromHexError),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("String parse error: {0}")]
    StringParseError(String),
    #[error("Invalid UTF-8 string")]
    InvalidUtf8,
    #[error("Invalid UTF-16 string")]
    InvalidUtf16,
    #[error(transparent)]
    PayloadError(#[from] PayloadError),
}

impl Error {
    pub fn io<S: Into<String>>(msg: S) -> Self {
        Error::Io(msg.into())
    }

    pub fn conn<S: Into<String>>(msg: S) -> Self {
        Error::Connection(msg.into())
    }

    pub fn proto<S: Into<String>>(msg: S) -> Self {
        Error::Protocol(msg.into())
    }

    pub fn penumbra<S: Into<String>>(msg: S) -> Self {
        Error::Penumbra(msg.into())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::penumbra(value.to_string())
    }
}

#[cfg(feature = "nusb")]
impl From<nusb::Error> for Error {
    fn from(err: nusb::Error) -> Self {
        Error::io(err.to_string())
    }
}

#[cfg(feature = "libusb")]
impl From<rusb::Error> for Error {
    fn from(err: rusb::Error) -> Self {
        Error::io(err.to_string())
    }
}

/*
    XFlash error codes work as follows:

    There are four severity levels:
    * Success (0 << 30, or 0x00000000)
    * Info    (1 << 30, or 0x40000000)
    * Warning (2 << 30, or 0x80000000)
    * Error   (3 << 30, or 0xC0000000)

    Then, follows the "domain" of this error code
    relates to:
    * Common     (1)
    * Security   (2)
    * Library    (3)
    * Device/HW  (4)
    * Host?      (5)
    * BROM       (6)
    * DA         (7)
    * Preloader  (8)

    Finally, the actual error code (0x01-...)

    Example:
    0xc0070004 => 0xC0000000 (Error) | 7 << 16 (domain) | 0x4 (code)
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq, Error, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum XFlashErrorKind {
    #[error("Generic error")]
    Error = 0xC0010001,
    #[error("Abort")]
    Abort = 0xC0010002,
    #[error("Unsupported command")]
    UnsupportedCommand = 0xC0010003,
    #[error("Unsupported devctrl code")]
    UnsupportedCtrlCode = 0xC0010004,
    #[error("Protocol error")]
    ProtocolError = 0xC0010005,
    #[error("Protocol buffer overflow")]
    ProtocolBufferOverflow = 0xC0010006,
    #[error("Insufficient buffer")]
    InsufficientBuffer = 0xC0010007,
    #[error("USB SCAN error")]
    UsbScanError = 0xC0010008,
    #[error("Invalid hsession")]
    InvalidHSession = 0xC0010009,
    #[error("Invalid session")]
    InvalidSession = 0xC001000A,
    #[error("Invalid stage")]
    InvalidStage = 0xC001000B,
    #[error("Not implemented")]
    NotImplemented = 0xC001000C,
    #[error("File not found")]
    FileNotFound = 0xC001000D,
    #[error("Open file error")]
    OpenFileError = 0xC001000E,
    #[error("Write file error")]
    WriteFileError = 0xC001000F,
    #[error("Read file error")]
    ReadFileError = 0xC0010010,
    #[error("Create File error / Unsupported Version")]
    CreateFileErrorOrUnsupportedVersion = 0xC0010011, // In XML these two errors are separated

    // Security
    #[error("SEC: Rom info not found")]
    RomInfoNotFound = 0xC0020001,
    #[error("SEC: Cust name not found")]
    CustNameNotFound = 0xC0020002,
    #[error("SEC: Device not supported")]
    DeviceNotSupported = 0xC0020003,
    #[error("SEC: Download forbidden (region is not whitelisted)")]
    DlForbidden = 0xC0020004,
    #[error("SEC: Image too large")]
    ImgTooLarge = 0xC0020005,
    #[error("SEC: Preloader verify failed")]
    PlVerifyFailed = 0xC0020006,
    #[error("SEC: Image verify failed")]
    ImageVerifyFailed = 0xC0020007,
    #[error("SEC: Hash operation failed")]
    HashOperationFailed = 0xC0020008,
    #[error("SEC: Hash binding check failed")]
    HashBindingCheckFailed = 0xC0020009,
    #[error("SEC: Invalid buffer")]
    InvalidBuf = 0xC002000A,
    #[error("SEC: Binding hash not available")]
    BindingHashNotAvailable = 0xC002000B,
    #[error("SEC: Write data not allowed (region is not whitelisted)")]
    WriteDataNotAllowed = 0xC002000C,
    #[error("SEC: Format not allowed (region is not whitelisted)")]
    FormatNotAllowed = 0xC002000D,
    #[error("SEC: SV5 public key auth failed")]
    Sv5PubKeyAuthFailed = 0xC002000E,
    #[error("SEC: SV5 hash verify failed")]
    Sv5HashVerifyFailed = 0xC002000F,
    #[error("SEC: SV5 RSA operation failed")]
    Sv5RsaOpFailed = 0xC0020010,
    #[error("SEC: SV5 RSA verify failed")]
    Sv5RsaVerifyFailed = 0xC0020011,
    #[error("SEC: SV5 GFH not found")]
    Sv5GfhNotFound = 0xC0020012,
    #[error("SEC: Invalid cert1")]
    Cert1Invalid = 0xC0020013,
    #[error("SEC: Invalid cert2")]
    Cert2Invalid = 0xC0020014,
    #[error("SEC: Image header invalid")]
    ImghdrInvalid = 0xC0020015,
    #[error("SEC: Signature size invalid")]
    SigSizeInvalid = 0xC0020016,
    #[error("SEC: RSA PSS operation failed")]
    RsaPssOpFailed = 0xC0020017,
    #[error("SEC: Certificate authentication failed")]
    CertAuthFailed = 0xC0020018,
    #[error("SEC: Public key auth mismatch N size")]
    PubKeyAuthMismatchNSize = 0xC0020019,
    #[error("SEC: Public key auth mismatch E size")]
    PubKeyAuthMismatchESize = 0xC002001A,
    #[error("SEC: Public key auth mismatch N")]
    PubKeyAuthMismatchN = 0xC002001B,
    #[error("SEC: Public key auth mismatch E")]
    PubKeyAuthMismatchE = 0xC002001C,
    #[error("SEC: Public key auth mismatch hash")]
    PubKeyAuthMismatchHash = 0xC002001D,
    #[error("SEC: Certificate object not found")]
    CertObjNotFound = 0xC002001E,
    #[error("SEC: Certificate OID not found")]
    CertOidNotFound = 0xC002001F,
    #[error("SEC: Certificate out of range")]
    CertOutOfRange = 0xC0020020,
    #[error("SEC: OID doesn't match")]
    OidDoesntMatch = 0xC0020021,
    #[error("SEC: Length doesn't match")]
    LengthDoesntMatch = 0xC0020022,
    #[error("SEC: ASN1 unknown operation")]
    Asn1UnknownOp = 0xC0020023,
    #[error("SEC: OID index out of range")]
    OidIndexOutOfRange = 0xC0020024,
    #[error("SEC: OID too large")]
    OidTooLarge = 0xC0020025,
    #[error("SEC: Public key size mismatch")]
    PubKeySizeMismatch = 0xC0020026,
    #[error("SEC: SWID mismatch")]
    SwidMismatch = 0xC0020027,
    #[error("SEC: Hash size mismatch")]
    HashSizeMismatch = 0xC0020028,
    #[error("SEC: Image header type mismatch")]
    ImghdrTypeMismatch = 0xC0020029,
    #[error("SEC: Image type mismatch")]
    ImgTypeMismatch = 0xC002002A,
    #[error("SEC: Image header hash verify failed")]
    ImghdrHashVerifyFailed = 0xC002002B,
    #[error("SEC: Image hash verify failed")]
    ImgHashVerifyFailed = 0xC002002C,
    #[error("SEC: Anti rollback violation")]
    AntiRollbackViolation = 0xC002002D,
    #[error("SEC: SECCFG not found")]
    SeccfgNotFound = 0xC002002E,
    #[error("SEC: SECCFG magic is incorrect")]
    SeccfgMagicIncorrect = 0xC002002F,
    #[error("SEC: SECCFG is invalid")]
    SeccfgInvalid = 0xC0020030,
    #[error("SEC: Cipher mode is invalid")]
    CipherModeInvalid = 0xC0020031,
    #[error("SEC: Cipher key is invalid")]
    CipherKeyInvalid = 0xC0020032,
    #[error("SEC: Cipher data unaligned")]
    CipherDataUnaligned = 0xC0020033,
    #[error("SEC: GFH file info not found")]
    GfhFileInfoNotFound = 0xC0020034,
    #[error("SEC: GFH anti clone not found")]
    GfhAntiCloneNotFound = 0xC0020035,
    #[error("SEC: GFH sec config not found")]
    GfhSecCfgNotFound = 0xC0020036,
    #[error("SEC: Unsupported source type")]
    UnsupportedSourceType = 0xC0020037,
    #[error("SEC: Cust name mismatch")]
    CustNameMismatch = 0xC0020038,
    #[error("SEC: Invalid address")]
    InvalidAddress = 0xC0020039,
    #[error("SEC: Certificate version not synced")]
    CertificateVersionNotSynced = 0xC0020040,
    #[error("SEC: Signature not synced")]
    SignatureNotSynced = 0xC0020041,
    #[error("SEC: Ext AllInOne Signature rejected")]
    ExtAllInOneSignatureRejected = 0xC0020042,
    #[error("SEC: Ext AllInOne Signature missing")]
    ExtAllInOneSignatureMissing = 0xC0020043,
    #[error("SEC: Communication key is not set")]
    CommKeyIsNotSet = 0xC0020044,
    #[error("SEC: Device info check failed")]
    DevInfoCheckFailed = 0xC0020045,
    #[error("SEC: Boot image count overflow")]
    BootimgCountOverflow = 0xC0020046,
    #[error("SEC: Signature not found")]
    SignatureNotFound = 0xC0020047,
    #[error("SEC: Boot image special handle")]
    BootimgSpecialHandle = 0xC0020048,
    #[error("SEC: Remote security policy disabled")]
    RemoteSecurityPolicyDisabled = 0xC0020049,
    #[error("SEC: RSA OAEP failed")]
    RsaOaepFailed = 0xC002004A,
    #[error("SEC: Insufficient buffer")]
    InsufficientBuffer2 = 0xC002004B,
    #[error("SEC: DA Anti-Rollback error. DA version less than OTP version.")]
    DaAntiRollbackError = 0xC002004C,
    #[error("SEC: Get OTP value failed")]
    GetOtpValueFailed = 0xC002004D,
    #[error("SEC: Invalid unit size")]
    InvalidUnitSize = 0xC002004E,
    #[error("SEC: Invalid group index")]
    InvalidGroupIdx = 0xC002004F,
    #[error("SEC: Image version overflow")]
    ImgVersionOverflow = 0xC0020050,
    #[error("SEC: OTP table not initialized")]
    OtpTableNotInitialized = 0xC0020051,
    #[error("SEC: Invalid partition name")]
    InvalidPartitionName = 0xC0020052,
    #[error("SEC: DA version Anti-Rollback error")]
    DaVersionAntiRollbackError = 0xC0020053,
    #[error("SEC: Invalid message size")]
    InvalidMsgSize = 0xC0020054,
    #[error("SEC: Security level unsupported")]
    SecurityLevelUnsupported = 0xC0020055,
    #[error("SEC: Security level mismatch")]
    SecurityLevelMismatch = 0xC0020056,
    #[error("SEC: Fault injection error")]
    FaultInjectionError = 0xC0020057,
    #[error("SEC: Public key hash group is invalid.")]
    PubKeyHashGroupInvalid = 0xC0020058,
    #[error("SEC: Security level too large")]
    SecurityLevelTooLarge = 0xC0020059,
    #[error("SEC: Security config is formatted")]
    SecurityConfigIsFormatted = 0xC002005A,
    #[error("SEC: Security config unknown error")]
    SecurityConfigUnknownError = 0xC002005B,
    #[error("SEC: Failed getting seccfg lockstate")]
    LockstateSeccfgFailed = 0xC002005C,
    #[error("SEC: Failed getting custom lockstate")]
    LockstateCustomFailed = 0xC002005D,
    #[error("SEC: Lockstate is inconsistent")]
    LockstateInconsistent = 0xC002005E,

    // Library
    #[error("Library: Scatter file invalid")]
    ScatterFileInvalid = 0xC0030001,
    #[error("Library: DA file invalid")]
    DaFileInvalid = 0xC0030002,
    #[error("Library: DA selection error")]
    DaSelectionError = 0xC0030003,
    #[error("Library: Preloader invalid")]
    PreloaderInvalid = 0xC0030004,
    #[error("Library: EMI header invalid")]
    EmiHdrInvalid = 0xC0030005,
    #[error("Library: Storage mismatch")]
    StorageMismatch = 0xC0030006,
    #[error("Library: Invalid parameters")]
    InvalidParameters = 0xC0030007,
    #[error("Library: Invalid GPT")]
    InvalidGpt = 0xC0030008,
    #[error("Library: Invalid PMT")]
    InvalidPmt = 0xC0030009,
    #[error("Library: Layout changed")]
    LayoutChanged = 0xC003000A,
    #[error("Library: Invalid format parameter")]
    InvalidFormatParam = 0xC003000B,
    #[error("Library: Unknown storage section type")]
    UnknownStorageSectionType = 0xC003000C,
    #[error("Library: Unknown scatter field")]
    UnknownScatterField = 0xC003000D,
    #[error("Library: Partition table doesn't exist")]
    PartitionTblDoesntExist = 0xC003000E,
    #[error("Library: Scatter HW chip ID mismatch")]
    ScatterHwChipIdMismatch = 0xC003000F,
    #[error("Library: SEC certificate file not found")]
    SecCertFileNotFound = 0xC0030010,
    #[error("Library: SEC authentication file not found")]
    SecAuthFileNotFound = 0xC0030011,
    #[error("Library: SEC authentication file needed")]
    SecAuthFileNeeded = 0xC0030012,
    #[error("Library: EMI container file not found")]
    EmiContainerFileNotFound = 0xC0030013,
    #[error("Library: Scatter file not found")]
    ScatterFileNotFound = 0xC0030014,
    #[error("Library: XML file operation error")]
    XmlFileOpError = 0xC0030015,
    #[error("Library: Unsupported page size")]
    UnsupportedPageSize = 0xC0030016,
    #[error("Library: EMI info length offset invalid")]
    EmiInfoLengthOffsetInvalid = 0xC0030017,
    #[error("Library: EMI info length invalid")]
    EmiInfoLengthInvalid = 0xC0030018,

    // Device (Storage, DRAM, eFuses)
    #[error("Device: Unsupported operation")]
    UnsupportedOperation = 0xC0040001,
    #[error("Device: Thread error")]
    ThreadError = 0xC0040002,
    #[error("Device: Checksum error")]
    ChecksumError = 0xC0040003,
    #[error("Device: Unknown sparse image format")]
    UnknownSparse = 0xC0040004,
    #[error("Device: Unknown sparse chunk type")]
    UnknownSparseChunkType = 0xC0040005,
    #[error("Device: Partition not found")]
    PartitionNotFound = 0xC0040006,
    #[error("Device: Failed to read partition table")]
    ReadParttblFailed = 0xC0040007,
    #[error("Device: Exceeded maximum partition number")]
    ExceededMaxPartitionNumber = 0xC0040008,
    #[error("Device: Unknown storage type")]
    UnknownStorageType = 0xC0040009,
    #[error("Device: DRAM test failed")]
    DramTestFailed = 0xC004000A,
    #[error("Device: Exceeded available range")]
    ExceedAvailableRange = 0xC004000B,
    #[error("Device: Failed to write sparse image")]
    WriteSparseImageFailed = 0xC004000C,
    #[error("Device: MMC error")]
    MmcError = 0xC0040030,
    #[error("Device: NAND error")]
    NandError = 0xC0040040,
    #[error("Device: NAND operation in progress")]
    NandInProgress = 0xC0040041,
    #[error("Device: NAND timeout")]
    NandTimeout = 0xC0040042,
    #[error("Device: NAND bad block")]
    NandBadBlock = 0xC0040043,
    #[error("Device: NAND erase failed")]
    NandEraseFailed = 0xC0040044,
    #[error("Device: NAND page program failed")]
    NandPageProgramFailed = 0xC0040045,
    #[error("Device: EMI setting version error")]
    EmiSettingVersionError = 0xC0040050,
    #[error("Device: UFS error")]
    UfsError = 0xC0040060,
    #[error("Device: DA OTP not supported")]
    DaOtpNotSupported = 0xC0040100,
    #[error("Device: DA OTP lock failed")]
    DaOtpLockFailed = 0xC0040102,

    // eFuses
    #[error("eFuse: Unknown error")]
    EfuseUnknownError = 0xC0040200,
    #[error("eFuse: Write timeout without verification")]
    EfuseWriteTimeoutWithoutVerify = 0xC0040201,
    #[error("eFuse: fuse blown")]
    EfuseBlown = 0xC0040202,
    #[error("eFuse: Revert bit is set")]
    EfuseRevertBit = 0xC0040203,
    #[error("eFuse: fuse is partly blown, needs to be blown again")]
    EfuseBlownPartly = 0xC0040204,
    #[error("eFuse: argument is invalid")]
    EfuseInvalidArgument = 0xC0040205,
    #[error("eFuse: fuse value is not zero")]
    EfuseValueIsNotZero = 0xC0040206,
    #[error("eFuse: fuse blown with incorrect data")]
    EfuseBlownIncorrectData = 0xC0040207,
    #[error("eFuse: Fuse is broken")]
    EfuseBroken = 0xC0040208,
    #[error("eFuse: Eror during blow operation")]
    EfuseBlowError = 0xC0040209,
    #[error("eFuse: Error while unlocking BPKEY")]
    EfuseUnlockBpkeyError = 0xC004020A,
    #[error("eFuse: Failed to create list")]
    EfuseCreateListError = 0xC004020B,
    #[error("eFuse: Failed to write to register")]
    EfuseWriteRegisterError = 0xC004020C,
    #[error("eFuse: Padding type mismatch")]
    EfusePaddingTypeMismatch = 0xC004020D,

    // Host commands
    #[error("Host: Device control exception")]
    DeviceCtrlException = 0xC0050001,
    #[error("Host: Shutdown command exception")]
    ShutdownCmdException = 0xC0050002,
    #[error("Host: Download exception")]
    DownloadException = 0xC0050003,
    #[error("Host: Upload exception")]
    UploadException = 0xC0050004,
    #[error("Host: External RAM exception")]
    ExtRamException = 0xC0050005,
    #[error("Host: Notify switch USB speed exception")]
    NotifySwitchUsbSpeedException = 0xC0050006,
    #[error("Host: Read data exception")]
    ReadDataException = 0xC0050007,
    #[error("Host: Write data exception")]
    WriteDataException = 0xC0050008,
    #[error("Host: Format exception")]
    FormatException = 0xC0050009,
    #[error("Host: OTP operation exception")]
    OtpOperationException = 0xC005000A,
    #[error("Host: Switch USB exception")]
    SwitchUsbException = 0xC005000B,
    #[error("Host: Write eFuse exception")]
    WriteEfuseException = 0xC005000C,
    #[error("Host: Read eFuse exception")]
    ReadEfuseException = 0xC005000D,

    // BROM
    #[error("BROM: Start command failed")]
    BromStartCmdFailed = 0xC0060001,
    #[error("BROM: Failed to get BBChip HW version")]
    BromGetBbchipHwVerFailed = 0xC0060002,
    #[error("BROM: Send DA command failed")]
    BromCmdSendDaFailed = 0xC0060003,
    #[error("BROM: Failed to jump to DA")]
    BromCmdJumpDaFailed = 0xC0060004,
    #[error("BROM: Command failed")]
    BromCmdFailed = 0xC0060005,
    #[error("BROM: Stage callback failed")]
    BromStageCallbackFailed = 0xC0060006,

    // DA section
    #[error("DA: Version mismatch")]
    DaVersionMismatch = 0xC0070001,
    #[error("DA: Not found")]
    DaNotFound = 0xC0070002,
    #[error("DA: Section not found")]
    DaSectionNotFound = 0xC0070003,
    #[error("DA: Hash mismatch. DA2 hash does not match hash in DA1")]
    DaHashMismatch = 0xC0070004,
    #[error("DA: Exceeded maximum allowed number")]
    DaExceedMaxNum = 0xC0070005,

    // Extensions
    #[error("Extensions: Download ack is not OK")]
    ExtensionsDownloadAckNotOk = 0xC00E0001,
    #[error("Extensions: Upload ack is not OK")]
    ExtensionsUploadAckNotOk = 0xC00E0002,
    #[error("Extensions: SEJ AES data length exceed max length")]
    ExtensionsSejExceedMaxLen = 0xC00E0003,
    #[error("Extensions: Malloc failed")]
    ExtensionsMallocFailed = 0xC00E0004,
    #[error("Extensions: RPMB not initialized")]
    ExtensionsRpmbNotInit = 0xC00E0005,
    #[error("Extensions: RPMB read failed")]
    ExtensionsRpmbReadFailed = 0xC00E0006,
    #[error("Extensions: RPMB write failed")]
    ExtensionsRpmbWriteFailed = 0xC00E0007,
    #[error("Extensions: RPMB key invalid.")]
    ExtensionsRpmbKeyInvalid = 0xC00E0008,
    #[error("Extensions: RPMB support is not available on this storage type.")]
    ExtensionsRpmbStorageNotSupported = 0xC00E0009,

    #[error("Unknown error")]
    Unknown = 0xFFFFFFFF,
}

#[derive(Debug, Error)]
#[error("{kind} (code: {code:#010x})")]
pub struct XFlashError {
    pub kind: XFlashErrorKind,
    pub code: u32,
}

impl XFlashError {
    pub fn from_code(code: u32) -> Self {
        let kind = XFlashErrorKind::try_from(code).unwrap_or(XFlashErrorKind::Unknown);
        Self { kind, code }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum XmlErrorKind {
    Unknown,
    UnsupportedCmd,
    Cancel,
}

#[derive(Debug, Error)]
#[error("XML Error: {message}")]
pub struct XmlError {
    pub message: String,
    pub kind: XmlErrorKind,
}

impl XmlError {
    pub fn new<S: Into<String>>(msg: S, kind: XmlErrorKind) -> Self {
        XmlError { message: msg.into(), kind }
    }

    pub fn from_message(resp: &[u8]) -> Self {
        let msg = std::str::from_utf8(resp).unwrap_or("Invalid UTF-8");

        let msg = msg.trim_end_matches('\0');

        match msg {
            "ERR!UNSUPPORTED" => XmlError::new("Unsupported command", XmlErrorKind::UnsupportedCmd),
            "ERR!CANCEL" => XmlError::new("Cancelled", XmlErrorKind::Cancel),
            _ => XmlError::new(msg, XmlErrorKind::UnsupportedCmd),
        }
    }
}

// BROM / Preloader errors
#[derive(Debug, Copy, Clone, Eq, PartialEq, Error, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum BrPlErrorKind {
    #[error("SEC: SLA challenge not completed. SLA must be completed before any proceeding")]
    SlaNotPassed = 0x1D0D,
    #[error("SEC: This command can be executed only once")]
    CmdExecMoreThanOnce = 0x1D0C,
    #[error("SEC: DA list max entries reached")]
    DaListMaxEntriesReached = 0x1D10,
    #[error("SEC: DAA signature error")]
    DaaSigError = 0x7015,
    #[error("SEC: An auth file is needed to continue")]
    ToolAuthIsNull = 0x7017,
    #[error("SEC: DAA signature verification failed")]
    DaaSigVfyFailed = 0x7024,

    #[error("Unknown error")]
    Unknown = 0xFFFF,
}

#[derive(Debug, Error)]
#[error("{kind} (code: {code:#06x})")]
pub struct BrPlError {
    pub kind: BrPlErrorKind,
    pub code: u16,
}

impl BrPlError {
    pub fn from_code(code: u16) -> Self {
        let kind = BrPlErrorKind::try_from(code).unwrap_or(BrPlErrorKind::Unknown);
        Self { kind, code }
    }
}

#[derive(Debug, Error)]
pub enum PayloadError {
    #[error("Data too short to contain a valid v6 header")]
    V6ShortData,
    #[error("Invalid v6 payload magic")]
    V6InvalidMagic,
}
