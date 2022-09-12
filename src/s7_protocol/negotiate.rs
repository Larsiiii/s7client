use std::convert::TryFrom;
use std::mem;

use bytes::{Buf, BufMut, BytesMut};

use crate::errors::Error;

use super::header::S7ProtocolHeader;

pub(crate) const NEGOTIATE_FUNCTION_CODE: u8 = 0xf0;

#[derive(Debug)]
pub(crate) struct S7Negotiation {
    s7_header: S7ProtocolHeader,
    params: NegotiatePDUParameters,
}

impl S7Negotiation {
    pub(crate) fn build() -> S7Negotiation {
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

impl From<S7Negotiation> for BytesMut {
    fn from(data: S7Negotiation) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(20);
        bytes.put(BytesMut::from(data.s7_header));
        bytes.put(BytesMut::from(data.params));

        bytes
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
    pub(crate) fn len() -> usize {
        8
    }

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

impl From<NegotiatePDUParameters> for BytesMut {
    fn from(params: NegotiatePDUParameters) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(8);
        bytes.put_u8(params.function_code);
        bytes.put_u8(params.reserved);
        bytes.put_u16(params.max_amq_caller);
        bytes.put_u16(params.max_amq_calle);
        bytes.put_u16(params.pdu_length);

        bytes
    }
}

impl TryFrom<&mut BytesMut> for NegotiatePDUParameters {
    type Error = Error;

    fn try_from(bytes: &mut BytesMut) -> Result<Self, Self::Error> {
        // check if there are enough bytes for a header
        if bytes.len() >= Self::len() {
            Ok(Self {
                function_code: bytes.get_u8(),
                reserved: bytes.get_u8(),
                max_amq_caller: bytes.get_u16(),
                max_amq_calle: bytes.get_u16_le(),
                pdu_length: bytes.get_u16(),
            })
        } else {
            Err(Error::Connection(
                "Received short packet while negotiating connection".to_string(),
            ))
        }
    }
}
