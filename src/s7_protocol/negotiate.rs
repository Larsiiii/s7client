use std::convert::TryFrom;
use std::mem;

use crate::errors::Error;

use super::header::S7ProtocolHeader;
use super::S7Protocol;

pub(crate) const NEGOTIATE_FUNCTION_CODE: u8 = 0xf0;

#[derive(Debug)]
pub(crate) struct S7Negotiation {
    s7_header: S7ProtocolHeader,
    params: NegotiatePDUParameters,
}

impl S7Protocol for S7Negotiation {
    fn build() -> S7Negotiation {
        Self {
            s7_header: S7ProtocolHeader::build_request(
                &mut 0,
                mem::size_of::<NegotiatePDUParameters>() as u16,
                0,
            ),
            params: NegotiatePDUParameters::build(),
        }
    }
}

impl From<S7Negotiation> for Vec<u8> {
    fn from(data: S7Negotiation) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        vec.append(&mut data.s7_header.into());
        vec.append(&mut data.params.into());
        vec
    }
}

#[derive(Debug)]
pub(crate) struct NegotiatePDUParameters {
    function_code: u8,
    reserved: u8,
    pub(crate) max_amq_caller: u16,
    pub(crate) max_amq_calle: u16,
    pub(crate) pdu_length: u16,
}

impl NegotiatePDUParameters {
    pub(crate) fn build() -> Self {
        Self {
            function_code: NEGOTIATE_FUNCTION_CODE,
            reserved: 0,
            max_amq_caller: 0x0100,
            max_amq_calle: 0x0100,
            pdu_length: 480,
        }
    }
}

impl From<NegotiatePDUParameters> for Vec<u8> {
    fn from(params: NegotiatePDUParameters) -> Vec<u8> {
        let mut vec = vec![params.function_code, params.reserved];
        vec.append(&mut params.max_amq_caller.to_be_bytes().to_vec());
        vec.append(&mut params.max_amq_calle.to_be_bytes().to_vec());
        vec.append(&mut params.pdu_length.to_be_bytes().to_vec());
        vec
    }
}

impl TryFrom<Vec<u8>> for NegotiatePDUParameters {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        match bytes.len() {
            8 => Ok(Self {
                function_code: bytes[0],
                reserved: bytes[1],
                max_amq_caller: u16::from_be_bytes([bytes[2], bytes[3]]),
                max_amq_calle: u16::from_le_bytes([bytes[4], bytes[5]]),
                pdu_length: u16::from_be_bytes([bytes[6], bytes[7]]),
            }),
            _ => Err(Error::TryFrom(
                bytes,
                "Invalid length for negotiated pdu parameters.".to_string(),
            )),
        }
    }
}
