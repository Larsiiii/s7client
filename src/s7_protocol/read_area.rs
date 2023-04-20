use bytes::{BufMut, BytesMut};
use std::convert::TryFrom;

use super::segments::{
    data_item::DataItem, header::S7ProtocolHeader, parameters::ReadWriteParams,
    request_item::RequestItem,
};
use super::types::{Area, READ_OPERATION};
use crate::connection::tcp::exchange_buffer;
use crate::errors::{Error, S7ProtocolError};
use crate::{S7Client, S7ReadAccess};

impl ReadWriteParams {
    pub(super) fn build_read(items: Vec<RequestItem>) -> Self {
        Self {
            function_code: READ_OPERATION,
            item_count: items.len() as u8,
            request_item: Some(items),
        }
    }
}

fn assert_pdu_size_for_read(
    data_items: &Vec<S7ReadAccess>,
    max_pdu_size: usize,
) -> Result<(), Error> {
    // send request limit: 19 bytes of header data, 12 bytes of parameter data for each dataItem
    let request_size = 19 + data_items.len() * RequestItem::len();
    if request_size > max_pdu_size {
        return Err(Error::TooManyItemsInOneRequest);
    }

    // response limit: 14 bytes of header data, 4 bytes of result data for each dataItem and the actual data
    let response_size = calculate_response_size(data_items);
    if response_size > max_pdu_size {
        return Err(Error::ResponseDataWouldBeTooLarge {
            req_size: response_size,
            max_pdu: max_pdu_size,
        });
    }

    Ok(())
}

fn calculate_response_size(data_items: &Vec<S7ReadAccess>) -> usize {
    data_items
        .iter()
        .map(|item| usize::from(item.len()))
        .sum::<usize>()
        + data_items.len() * DataItem::header_len()
        + 14
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn read_area_single(
    client: &mut S7Client,
    area: Area,
    data_item: S7ReadAccess,
) -> Result<Vec<u8>, Error> {
    // Each PDU (TPKT Header + COTP Header + S7Header + S7Parameters + S7Data) must not exceed the maximum PDU length (bytes) negotiated with the
    // PLC during connection.
    // Moreover we must ensure that a "finite" number of items is send per PDU. If the command size does not fit in one PDU
    // then it must be split across more subsequent PDU.

    let max_pdu_size_usize = usize::from(client.pdu_length);

    let response_size = calculate_response_size(&vec![data_item]);
    let items = if response_size > max_pdu_size_usize {
        // split request into multiple each smaller than the max PDU size
        // max data size per request (1 item per request)
        // 12 bytes of header data, 2 bytes of param header, 4 bytes of result data for each dataItem and the actual data
        let max_data_size = max_pdu_size_usize
            - S7ProtocolHeader::len_response()
            - ReadWriteParams::len()
            - DataItem::header_len();

        let (item_count_required, rest) = (
            usize::from(data_item.len()) / max_data_size,
            usize::from(data_item.len()) % max_data_size,
        );

        // create multiple items for request
        let mut items: Vec<S7ReadAccess> = (0..item_count_required)
            .map(|i| S7ReadAccess::Bytes {
                db_number: data_item.db_number(),
                start: (i * max_data_size) as u32 + data_item.start(),
                length: max_data_size as u16,
            })
            .collect();

        // add rest of data for request
        if rest > 0 {
            items.push(S7ReadAccess::Bytes {
                db_number: data_item.db_number(),
                start: ((item_count_required) * max_data_size) as u32 + data_item.start(),
                length: rest as u16,
            });
        }

        items
    } else {
        vec![data_item]
    };

    let mut overall_response_data = BytesMut::new();

    for req in items {
        let request_item = RequestItem::build(
            area,
            req.db_number(),
            req.start(),
            req.data_type(),
            req.len().into(),
        )?;
        let request_params = BytesMut::from(ReadWriteParams::build_read(vec![request_item]));

        // create data buffer
        let mut bytes = BytesMut::new();

        let req_header =
            S7ProtocolHeader::build_request(&mut client.pdu_number, request_params.len(), 0)?;
        bytes.put(BytesMut::from(req_header));
        bytes.put(request_params);

        let mut response = exchange_buffer(&mut client.connection, bytes).await?;

        // check if s7 header is ack with data and check for errors
        // check if pdu of response matches request pdu
        let resp_header = S7ProtocolHeader::try_from(&mut response)?;
        resp_header
            .is_ack_with_data()?
            .is_current_pdu_response(client.pdu_number)?;

        // Check for errors
        if resp_header.has_error() {
            let (class, code) = resp_header.get_errors();
            return Err(Error::S7ProtocolError(S7ProtocolError::from_codes(
                class, code,
            )));
        }

        // get data
        let _read_params = ReadWriteParams::from(&mut response);
        let data_item = DataItem::try_from(&mut response)?;
        overall_response_data.put(data_item.data.as_ref());
    }

    Ok(overall_response_data.to_vec())
}

pub(crate) async fn read_area_multi(
    client: &mut S7Client,
    area: Area,
    info: Vec<S7ReadAccess>,
) -> Result<Vec<Result<Vec<u8>, Error>>, Error> {
    // Each PDU (TPKT Header + COTP Header + S7Header + S7Parameters + S7Data) must not exceed the maximum PDU length (bytes) negotiated with the
    // PLC during connection.
    // Moreover we must ensure that a "finite" number of items is send per PDU. If the command size does not fit in one PDU
    // then it must be split across more subsequent PDU.

    assert_pdu_size_for_read(&info, client.pdu_length.into())?;

    let request_params = BytesMut::from(ReadWriteParams::build_read(
        info.iter()
            .map(|info| {
                RequestItem::build(
                    area,
                    info.db_number(),
                    info.start(),
                    info.data_type(),
                    info.len().into(),
                )
            })
            .collect::<Result<Vec<RequestItem>, Error>>()?,
    ));

    // create data buffer
    let mut bytes = BytesMut::new();

    let req_header =
        S7ProtocolHeader::build_request(&mut client.pdu_number, request_params.len(), 0)?;
    bytes.put(BytesMut::from(req_header));
    bytes.put(request_params);

    let mut response = exchange_buffer(&mut client.connection, bytes).await?;

    // check if s7 header is ack with data and check for errors
    // check if pdu of response matches request pdu
    let resp_header = S7ProtocolHeader::try_from(&mut response)?;
    resp_header
        .is_ack_with_data()?
        .is_current_pdu_response(client.pdu_number)?;

    // Check for errors
    if resp_header.has_error() {
        let (class, code) = resp_header.get_errors();
        return Err(Error::S7ProtocolError(S7ProtocolError::from_codes(
            class, code,
        )));
    }

    // get response data
    let read_params = ReadWriteParams::from(&mut response);
    let data = (0..read_params.item_count)
        .map(|_| DataItem::try_from(&mut response))
        .map(|item| match item {
            Ok(item) => Ok(item.data.to_vec()),
            Err(e) => Err(e),
        })
        .collect::<Vec<Result<Vec<u8>, Error>>>();

    Ok(data)
}
