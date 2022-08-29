use std::convert::TryFrom;
use std::mem;
// use std::net::TcpStream;
use tokio::net::TcpStream;

use super::header::S7ProtocolHeader;
use super::types::{Area, DataItem, ReadWriteParams, RequestItem, S7DataTypes, READ_OPERATION};
use crate::connection::tcp::exchange_buffer;
use crate::errors::{Error, IsoError};

impl ReadWriteParams {
    pub(super) fn build_read(items: Vec<RequestItem>) -> Self {
        Self {
            function_code: READ_OPERATION,
            item_count: items.len() as u8,
            request_item: Some(items),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn read_area(
    conn: &mut TcpStream,
    pdu_length: u16,
    pdu_number: &mut u16,
    area: Area,
    db_number: u16,
    start: u32,
    size: u32,
    data_type: S7DataTypes,
) -> Result<Vec<u8>, Error> {
    // Each packet cannot exceed the PDU length (in bytes) negotiated, and moreover
    // we must ensure to transfer a "finite" number of item per PDU
    // Protocol header size (should be 18)
    let header_size = (mem::size_of::<S7ProtocolHeader>() - 2) - 6; // -6 to account for options
    let requested_size = size * data_type.get_size();
    if ((pdu_length as i32 - header_size as i32) / data_type.get_size() as i32) < 1 {
        return Err(Error::ISORequest(IsoError::InvalidPDU));
    }
    let max_elements = (pdu_length as usize - header_size) as u32 / data_type.get_size();

    let mut buffer: Vec<u8> = Vec::new();
    let mut offset = 0;
    while offset == 0 || offset < requested_size {
        let items_to_request: u32 = match requested_size - offset {
            x if x > max_elements => max_elements,
            _ => size - offset,
        };
        let items = RequestItem::build(
            area,
            db_number,
            start + offset,
            data_type,
            items_to_request as u16,
        );
        let mut request_params: Vec<u8> = ReadWriteParams::build_read(vec![items]).into();
        // TODO!!!! Add last_pdu_ref
        // TODO check if response pdu ref matches requests
        let s7_header = S7ProtocolHeader::build_request(pdu_number, request_params.len() as u16, 0);
        let mut request: Vec<u8> = s7_header.into();
        request.append(&mut request_params);

        let read_data = exchange_buffer(conn, &mut request).await?;
        // TODO Check if s7 header is ack with data and check for errors
        S7ProtocolHeader::try_from(read_data[0..12].to_vec())?
            .is_ack_with_data()?
            .is_current_pdu_response(*pdu_number)?;

        let mut data_item = DataItem::try_from(read_data[14..].to_vec())?;
        offset += data_item.data.len() as u32;
        buffer.append(&mut data_item.data);
    }

    Ok(buffer)
}
