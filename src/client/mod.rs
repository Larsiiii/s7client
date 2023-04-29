use std::borrow::Cow;

use crate::{errors::Error, s7_protocol::types::S7DataTypes};

pub(crate) mod create;
pub(crate) mod pooled;
pub(crate) mod read;
pub(crate) mod triggers;
pub(crate) mod write;

pub(crate) fn verify_max_bit(bit: u8) -> Result<(), Error> {
    if bit > 7 {
        return Err(Error::RequestedBitOutOfRange);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
#[must_use]
/// Allows configuration of reading access to S7 PLC
pub enum S7ReadAccess {
    /// Configure reading access for a chunk of bytes
    Bytes {
        /// Number of data block to access
        db_number: u16,
        /// Number of byte to start the reading access from
        start: u32,
        /// Number of bytes to read
        length: u16,
    },
    /// Configure reading access for a single bit
    Bit {
        /// Number of data block to access
        db_number: u16,
        /// Number of byte to access
        byte: u32,
        /// Number of bit to access
        bit: u8,
    },
}

impl S7ReadAccess {
    /// Convenience function to create configuration for reading a single bit from the PLC
    pub fn bit(db_number: u16, byte: u32, bit: u8) -> Self {
        Self::Bit {
            db_number,
            byte,
            bit,
        }
    }

    /// Convenience function to create configuration for reading a chunk of bytes from the PLC
    pub fn bytes(db_number: u16, start: u32, length: u16) -> Self {
        Self::Bytes {
            db_number,
            start,
            length,
        }
    }

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

    pub(crate) fn max_bit(&self) -> u8 {
        match self {
            Self::Bytes { .. } => 0,
            Self::Bit { bit, .. } => *bit,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[must_use]
/// Allows configuration of writing access to S7 PLC
pub enum S7WriteAccess<'a> {
    /// Configure writing access for a chunk of bytes
    Bytes {
        /// Number of data block to access
        db_number: u16,
        /// Number of byte to start writing
        start: u32,
        /// Data bytes to write to the PLC
        data: &'a [u8],
    },
    /// Configure writing access for a single bit
    Bit {
        /// Number of data block to access
        db_number: u16,
        /// Number of byte to write to
        byte: u32,
        /// Number of bit to write to
        bit: u8,
        /// Value to write
        value: bool,
    },
}

impl<'a> S7WriteAccess<'a> {
    /// Convenience function to create configuration for writing a single bit to the PLC
    pub fn bit(db_number: u16, byte: u32, bit: u8, value: bool) -> Self {
        Self::Bit {
            db_number,
            byte,
            bit,
            value,
        }
    }

    /// Convenience function to create configuration for writing a chunk of bytes to the PLC
    pub fn bytes(db_number: u16, start: u32, data: &'a [u8]) -> Self {
        Self::Bytes {
            db_number,
            start,
            data,
        }
    }

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
            Self::Bit { value, .. } => Cow::Owned(vec![u8::from(*value)]),
        }
    }

    pub(crate) fn max_bit(&self) -> u8 {
        match self {
            Self::Bytes { .. } => 0,
            Self::Bit { bit, .. } => *bit,
        }
    }
}
