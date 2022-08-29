use std::convert::TryFrom;
use std::mem;
// use std::net::TcpStream;
use tokio::net::TcpStream;

use super::header::S7ProtocolHeader;
use super::types::{Area, DataItem, ReadWriteParams, RequestItem, S7DataTypes, WRITE_OPERATION};
use crate::connection::tcp::exchange_buffer;
use crate::errors::{Error, IsoError, S7DataItemResponseError};

impl ReadWriteParams {
    fn build_write(items: Vec<RequestItem>) -> Self {
        Self {
            function_code: WRITE_OPERATION,
            item_count: items.len() as u8,
            request_item: Some(items),
        }
    }
}

impl DataItem {
    fn build_write(data_type: S7DataTypes, data: Option<&[u8]>) -> Result<Self, Error> {
        match data {
            Some(vec) => Ok(Self {
                error_code: 0,
                var_type: data_type as u8,
                count: vec.len() as u16 * 8,
                data: vec.to_vec(),
            }),
            None => Err(Error::ISORequest(IsoError::InvalidDataSize)),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn write_area(
    conn: &mut TcpStream,
    pdu_length: u16,
    pdu_number: &mut u16,
    area: Area,
    db_number: u16,
    start: u32,
    data_type: S7DataTypes,
    buffer: &Vec<u8>,
) -> Result<(), Error> {
    // Each packet cannot exceed the PDU length (in bytes) negotiated, and moreover
    // we must ensure to transfer a "finite" number of item per PDU
    // Reply telegram header (should be 35)
    let header_size = (mem::size_of::<S7ProtocolHeader>() - 2) - 6 // -2 without first two fields;  -6 to account for options
                        + (mem::size_of::<ReadWriteParams>())
        - 3;
    let requested_size = buffer.len() as u32 / data_type.get_size();
    if ((pdu_length as i32 - header_size as i32) / data_type.get_size() as i32) < 1 {
        return Err(Error::ISORequest(IsoError::InvalidPDU));
    }
    let max_elements = (pdu_length as usize - header_size) as u32 / data_type.get_size();

    let mut offset: u32 = 0;
    while offset == 0 || offset < buffer.len() as u32 {
        let items_to_write: u32 = match buffer.len() as u32 - offset {
            x if x > max_elements => max_elements,
            _ => requested_size - offset,
        };

        let items = RequestItem::build(
            area,
            db_number,
            start + offset,
            data_type,
            items_to_write as u16,
        );
        let mut data: Vec<u8> = DataItem::build_write(
            data_type,
            buffer.get(offset as usize..(items_to_write * data_type.get_size()) as usize),
        )?
        .into();
        let data_length = data.len();
        let mut write_params: Vec<u8> = ReadWriteParams::build_write(vec![items]).into();
        write_params.append(&mut data);

        dbg!("Datenlänge: {:?}", data_length);
        // TODO!!!! Add last_pdu_ref
        // TODO check if response pdu ref matches requests
        let s7_header = S7ProtocolHeader::build_request(
            pdu_number,
            (write_params.len() - data_length) as u16,
            data_length as u16,
        );
        dbg!("{:?}", &write_params);
        let mut request: Vec<u8> = s7_header.into();
        request.append(&mut write_params);

        offset += requested_size;

        let exchanged_data = exchange_buffer(conn, &mut request).await?;
        S7ProtocolHeader::try_from(exchanged_data[0..12].to_vec())?
            .is_ack_with_data()?
            .is_current_pdu_response(*pdu_number)?;

        // Check for errors
        let error_code = exchanged_data[14];
        // 255 signals everything went alright
        if error_code != 255 {
            return Err(Error::DataItemError(S7DataItemResponseError::from(
                error_code,
            )));
        }
        offset += requested_size;
    }

    Ok(())
}
