use crate::s7_protocol::types::S7DataTypes;

pub(crate) mod create;
pub(crate) mod pool;
pub(crate) mod read;
pub(crate) mod write;

#[derive(Debug, Clone, Copy)]
pub enum S7ReadAccess {
    Bytes {
        db_number: u16,
        start: u32,
        length: u32,
    },
    Bit {
        db_number: u16,
        byte: u32,
        bit: u32,
    },
}

impl S7ReadAccess {
    pub(crate) fn db_number(&self) -> u16 {
        match self {
            Self::Bytes { db_number, .. } | Self::Bit { db_number, .. } => *db_number,
        }
    }

    pub(crate) fn start(&self) -> u32 {
        match self {
            Self::Bytes { start, .. } => *start,
            Self::Bit { byte, bit, .. } => byte * 8 + bit,
        }
    }

    pub(crate) fn len(&self) -> u32 {
        match self {
            Self::Bytes { length, .. } => *length,
            Self::Bit { .. } => 1,
        }
    }

    pub(crate) fn data_type(&self) -> S7DataTypes {
        match self {
            Self::Bytes { .. } => S7DataTypes::S7BYTE,
            Self::Bit { .. } => S7DataTypes::S7BIT,
        }
    }
}

#[derive(Debug, Clone)]
pub enum S7WriteAccess {
    Bytes {
        db_number: u16,
        start: u32,
        data: Vec<u8>,
    },
    Bit {
        db_number: u16,
        byte: u32,
        bit: u32,
        value: bool,
    },
}
