//! Types for working with errors produced by s7client.

use std::fmt;
use std::io::{Error as IOError, ErrorKind};

use deadpool::managed::PoolError;

#[derive(Debug)]
pub enum Error {
    IO(ErrorKind),
    Pool(Box<PoolError<Self>>),
    Connection(String),
    DataExchangeTimedOut,
    TryFrom(Vec<u8>, String),
    ISOResponse(IsoError),
    ISORequest(IsoError),
    RequestedBitOutOfRange,
    RequestNotAcknowledged,
    S7ProtocolError(S7ProtocolError),
    DataItemError(S7DataItemResponseError),
    ResponseDoesNotBelongToCurrentPDU,
    TooManyItemsInOneRequest,
    DataItemTooLarge,
    TooMuchDataToWrite,
    ResponseDataWouldBeTooLarge { req_size: usize, max_pdu: usize },
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::IO(e.kind())
    }
}

impl From<PoolError<Error>> for Error {
    fn from(e: PoolError<Error>) -> Self {
        Error::Pool(Box::new(e))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::IO(e) => format!("IO Error: {e}"),
                Error::Pool(e) => format!("Pool Error: {e}"),
                Error::Connection(e) => format!("Connection Error: {e}"),
                Error::DataExchangeTimedOut => "Timeout during data exchange".to_string(),
                Error::TryFrom(_, e) => e.to_string(),
                Error::ISOResponse(e) => format!("ISO Response Error: {e}"),
                Error::ISORequest(e) => format!("ISO Request Error: {e}"),
                Error::RequestedBitOutOfRange =>
                    "The request bit is out of range [0..7]".to_string(),
                Error::RequestNotAcknowledged => "The PLC did not respond successfully".to_string(),
                Error::S7ProtocolError(e) => e.to_string(),
                Error::DataItemError(e) => e.to_string(),
                Error::ResponseDoesNotBelongToCurrentPDU =>
                    "Mismatch in response and request ID".to_string(),
                Error::TooManyItemsInOneRequest => "Too many items in one request".to_string(),
                Error::DataItemTooLarge => "The data item in the request is too large".to_string(),
                Error::TooMuchDataToWrite =>
                    "Too much data supplied for one write request".to_string(),
                    Error::ResponseDataWouldBeTooLarge { req_size, max_pdu } => format!("Too much data requested for one read request. Response size ({req_size}) is larger than the protocol limit ({max_pdu})")
            }
        )
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum IsoError {
    Connect = 0x0001_0000,         // Connection error
    Disconnect = 0x0002_0000,      // Disconnect error
    InvalidPDU = 0x0003_0000,      // Bad format
    InvalidDataSize = 0x0004_0000, // Bad Data size passed to send/recv : buffer is invalid
    // NullPointer = 0x00050000,      // Null passed as pointer
    ShortPacket = 0x0006_0000,      // A short packet received
    TooManyFragments = 0x0007_0000, // Too many packets without EoT flag
    PduOverflow = 0x0008_0000,      // The sum of fragments data exceeded maximum packet size
    SendPacket = 0x0009_0000,       // An error occurred during send
    RecvPacket = 0x000A_0000,       // An error occurred during recv
    InvalidParams = 0x000B_0000,    // Invalid TSAP params
    Unknown,
}

impl fmt::Display for IsoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::Connect => " ISO : Connection error",
                Self::Disconnect => " ISO : Disconnect error",
                Self::InvalidPDU => " ISO : Bad PDU format",
                Self::InvalidDataSize => " ISO : Data size passed to send/recv buffer is invalid",
                // Self::NullPointer => " ISO : Null passed as pointer",
                Self::ShortPacket => " ISO : A short packet received",
                Self::TooManyFragments => " ISO : Too many packets without EoT flag",
                Self::PduOverflow =>
                    " ISO : The sum of fragments data exceeded maximum packet size",
                Self::SendPacket => " ISO : An error occurred during send",
                Self::RecvPacket => " ISO : An error occurred during recv",
                Self::InvalidParams => "ISO : Invalid connection params (wrong TSAPs)",
                Self::Unknown => " ISO : Unknown error",
            }
        )
    }
}

/// S7 protocol error
#[derive(Debug)]
pub struct S7ProtocolError {
    /// Error class
    class: &'static str,
    /// Error code
    error: Option<u8>,
}

impl fmt::Display for S7ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut error = vec![format!("S7 Protocol error: {}", &self.class)];
        if let Some(error_code) = self.error {
            error.push(format!("error code: {}", error_code));
        }
        write!(f, "{}", error.join(" - "))
    }
}

impl S7ProtocolError {
    pub(crate) fn from_codes(class: Option<u8>, code: Option<u8>) -> Self {
        Self {
            class: match class {
                Some(class_code) => match class_code {
                    0x00 => "No error",
                    0x81 => "Application relationship error",
                    0x82 => "Object definition error",
                    0x83 => "No resources available error",
                    0x84 => "Error on service processing",
                    0x85 => "Error on supplies",
                    0x87 => "Access error",
                    _ => "Unknown error class",
                },
                None => "No error class given",
            },
            error: code,
        }
    }
}

/// Errors from a data item included inside a S7 PLC response
#[derive(Debug)]
pub enum S7DataItemResponseError {
    /// Reserved
    Reserved,
    /// Hardware fault
    HardwareFault,
    /// Accessing the object is not allowed
    AccessNotAllowed,
    /// Address out of range
    AddressOutOfRange,
    /// Data type is not supported
    DataTypeNotSupported,
    /// Inconsistencies in the data type occurred
    DataTypeInconsistent,
    /// Requested object does not exist
    ObjectDoesNotExist,
    /// Unknown error
    Unknown,
}

impl fmt::Display for S7DataItemResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::Reserved => "Reserved",
            Self::HardwareFault => "Hardware fault",
            Self::AccessNotAllowed => "Accessing the object not allowed",
            Self::AddressOutOfRange => "Address out of range",
            Self::DataTypeNotSupported => "Data type not supported",
            Self::DataTypeInconsistent => "Data type inconsistent",
            Self::ObjectDoesNotExist => "Object does not exist",
            Self::Unknown => "Unknown error",
        };
        write!(f, "S7 Data Item response error: {msg}")
    }
}

impl From<u8> for S7DataItemResponseError {
    fn from(code: u8) -> Self {
        match code {
            0x00 => Self::Reserved,
            0x01 => Self::HardwareFault,
            0x03 => Self::AccessNotAllowed,
            0x05 => Self::AddressOutOfRange,
            0x06 => Self::DataTypeNotSupported,
            0x07 => Self::DataTypeInconsistent,
            0x0a => Self::ObjectDoesNotExist,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use error_stack::{IntoReport, Report, ResultExt};

    use super::*;

    #[test]
    fn error_stack() {
        println!("{:?}", create_error_stack());
    }

    fn create_error_stack() -> Result<(), Report<Error>> {
        create_error()
            .into_report()
            .change_context(Error::RequestedBitOutOfRange)
    }

    fn create_error() -> Result<(), Error> {
        Err(Error::RequestNotAcknowledged)
    }
}
