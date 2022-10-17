use std::borrow::Cow;

use crate::s7_protocol::types::S7DataTypes;

pub(crate) mod create;
pub(crate) mod pooled;
pub(crate) mod read;
pub(crate) mod write;

#[derive(Debug, Clone, Copy)]
/// 
pub enum S7ReadAccess {
    Bytes {
        db_number: u16,
        start: u32,
        length: u16,
    },
    Bit {
        db_number: u16,
        byte: u32,
        bit: u8,
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
            Self::Bit { byte, bit, .. } => byte * 8 + u32::from(*bit),
        }
    }

    pub(crate) fn len(&self) -> u16 {
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

#[derive(Debug, Clone, Copy)]
pub enum S7WriteAccess<'a> {
    Bytes {
        db_number: u16,
        start: u32,
        data: &'a Vec<u8>,
    },
    Bit {
        db_number: u16,
        byte: u32,
        bit: u8,
        value: bool,
    },
}

impl<'a> S7WriteAccess<'a> {
    pub(crate) fn db_number(&'a self) -> u16 {
        match self {
            Self::Bytes { db_number, .. } | Self::Bit { db_number, .. } => *db_number,
        }
    }

    pub(crate) fn start(&'a self) -> u32 {
        match self {
            Self::Bytes { start, .. } => *start,
            Self::Bit { byte, bit, .. } => byte * 8 + u32::from(*bit),
        }
    }

    pub(crate) fn len(&'a self) -> usize {
        match self {
            Self::Bytes { data, .. } => data.len(),
            Self::Bit { .. } => 1,
        }
    }

    pub(crate) fn data_type(&'a self) -> S7DataTypes {
        match self {
            Self::Bytes { .. } => S7DataTypes::S7BYTE,
            Self::Bit { .. } => S7DataTypes::S7BIT,
        }
    }

    pub(crate) fn data(&'a self) -> Cow<'a, [u8]> {
        match self {
            Self::Bytes { data, .. } => Cow::Borrowed(data),
            Self::Bit { value, .. } => Cow::Owned(vec![if *value { 1 } else { 0 }]),
        }
    }
}
