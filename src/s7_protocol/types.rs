pub(super) const READ_OPERATION: u8 = 0x04;
pub(super) const WRITE_OPERATION: u8 = 0x05;

pub(super) const SPEC_TYPE_READ_WRITE: u8 = 0x12;
pub(super) const SYNTAX_ID_ANY_TYPE: u8 = 0x10;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum Area {
    ProcessInput = 0x81,
    ProcessOutput = 0x82,
    /// Merker is an address registers within the CPU.
    /// The number of available flag bytes depends on the respective CPU and can be taken from the technical data.
    /// You can use flag bits, flag bytes, flag words or flag double words in a PLC program.
    Merker = 0x83,
    /// German thing, means building blocks
    /// This is your storage  
    DataBlock = 0x84,
    Counter = 0x1C,
    Timer = 0x1D,
    Unknown,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum S7DataTypes {
    S7BIT = 0x01,  // Bit (inside a word)
    S7BYTE = 0x02, // Byte (8 bit)
    S7CHAR = 0x03,
    S7WORD = 0x04, // Word (16 bit)
    S7INT = 0x05,
    S7DWORD = 0x06, // Double Word (32 bit)
    S7DINT = 0x07,
    S7REAL = 0x08,    // Real (32 bit float)
    S7COUNTER = 0x1C, // Counter (16 bit)
    S7TIMER = 0x1D,   // Timer (16 bit)
}

#[derive(Debug)]
pub(crate) enum DataItemTransportSize {
    Null = 0x00,        // Null
    Bit = 0x03,         // Bit
    Byte = 0x04,        // Byte/Word/DWord
    Integer = 0x05,     // Integer
    Real = 0x07,        // Real
    OctetString = 0x09, // Octet String
}

impl From<u8> for DataItemTransportSize {
    fn from(val: u8) -> Self {
        match val {
            0x03 => Self::Bit,
            0x04 => Self::Byte,
            0x05 => Self::Integer,
            0x07 => Self::Real,
            0x09 => Self::OctetString,
            _ => Self::Null,
        }
    }
}

impl DataItemTransportSize {
    pub(crate) fn len(&self) -> u16 {
        match self {
            Self::Null => 0,
            Self::Bit => 1,
            Self::Byte | Self::Integer | Self::Real | Self::OctetString => 8,
        }
    }
}

impl From<S7DataTypes> for DataItemTransportSize {
    fn from(data_type: S7DataTypes) -> Self {
        match data_type {
            S7DataTypes::S7BIT => Self::Bit,
            S7DataTypes::S7BYTE
            | S7DataTypes::S7CHAR
            | S7DataTypes::S7WORD
            | S7DataTypes::S7DWORD
            | S7DataTypes::S7DINT
            | S7DataTypes::S7COUNTER
            | S7DataTypes::S7TIMER => Self::Byte,
            S7DataTypes::S7INT => Self::Integer,
            S7DataTypes::S7REAL => Self::Real,
        }
    }
}
