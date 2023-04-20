use bytes::{Buf, BufMut, BytesMut};
use std::borrow::Cow;
use std::convert::TryFrom;

use super::segments::{
    data_item::DataItem, header::S7ProtocolHeader, parameters::ReadWriteParams,
    request_item::RequestItem,
};
use super::types::{Area, DataItemTransportSize, WRITE_OPERATION};
use crate::connection::{iso::TTPKTHeader, tcp::exchange_buffer};
use crate::errors::{Error, S7DataItemResponseError, S7ProtocolError};
use crate::{S7Client, S7WriteAccess};

impl ReadWriteParams {
    fn build_write(items: Vec<RequestItem>) -> Self {
        Self {
            function_code: WRITE_OPERATION,
            item_count: items.len() as u8,
            request_item: Some(items),
        }
    }
}

impl<'a> DataItem<'a> {
    // fn build_write(data_type: DataItemTransportSize, data: Option<&[u8]>) -> Result<Self, Error> {
    //     let transport_size = data_type.len();
    //     match data {
    //         Some(vec) => Ok(Self {
    //             error_code: 0,
    //             var_type: data_type as u8,
    //             count: vec.len() as u16 * transport_size,
    //             data: vec,
    //         }),
    //         None => Err(Error::ISORequest(IsoError::InvalidDataSize)),
    //     }
    // }

    fn build_write2(data_type: DataItemTransportSize, data: Cow<'a, [u8]>) -> Result<Self, Error> {
        let transport_size = data_type.len();
        Ok(Self {
            error_code: 0,
            var_type: data_type as u8,
            count: u16::try_from(data.len()).map_err(|_| Error::DataItemTooLarge)? * transport_size,
            data,
        })
    }
}

fn assert_pdu_size_for_write<'a>(
    data_items: &'a Vec<S7WriteAccess<'a>>,
    max_pdu_size: usize,
) -> Result<(), Error> {
    // 12 bytes of header data, 18 bytes of parameter data for each dataItem
    if data_items.len() * 18 + usize::from(TTPKTHeader::len()) > max_pdu_size {
        return Err(Error::TooManyItemsInOneRequest);
    }

    // 12 bytes of header data, 16 bytes of data for each dataItem and the actual data
    if data_items.iter().map(S7WriteAccess::len).sum::<usize>()
        + data_items.len() * 16
        + usize::from(TTPKTHeader::len())
        > max_pdu_size
    {
        return Err(Error::TooMuchDataToWrite);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn write_area_single(
    client: &mut S7Client,
    area: Area,
    data_item: S7WriteAccess<'_>,
) -> Result<(), Error> {
    // Each PDU (TPKT Header + COTP Header + S7Header + S7Parameters + S7Data) must not exceed the maximum PDU length (bytes) negotiated with the
    // PLC during connection.
    // Moreover we must ensure that a "finite" number of items is send per PDU. If the command size does not fit in one PDU
    // then it must be split across more subsequent PDU.

    assert_pdu_size_for_write(&vec![data_item], client.pdu_length.into())?;

    let request_params = BytesMut::from(ReadWriteParams::build_write(vec![RequestItem::build(
        area,
        data_item.db_number(),
        data_item.start(),
        data_item.data_type(),
        data_item.len(),
    )?]));
    let data_items: BytesMut =
        DataItem::build_write2(data_item.data_type().into(), data_item.data())?.into();

    // create data buffer
    let mut bytes = BytesMut::new();

    let req_header = S7ProtocolHeader::build_request(
        &mut client.pdu_number,
        request_params.len(),
        data_items.len(),
    )?;
    bytes.put(BytesMut::from(req_header));
    bytes.put(request_params);
    bytes.put(data_items);

    let mut response = exchange_buffer(&mut client.connection, bytes).await?;

    // check if s7 header is ack with data and check for errors
    // check if pdu of response matches request pdu
    let resp_header = S7ProtocolHeader::try_from(&mut response)?;
    resp_header
        .is_ack()?
        .is_current_pdu_response(client.pdu_number)?;

    // Check for errors
    if resp_header.has_error() {
        let (class, code) = resp_header.get_errors();
        return Err(Error::S7ProtocolError(S7ProtocolError::from_codes(
            class, code,
        )));
    }

    let _read_params = ReadWriteParams::from(&mut response);

    let error_code = response.get_u8();
    // 255 signals everything went alright
    if error_code == 255 {
        Ok(())
    } else {
        Err(Error::DataItemError(S7DataItemResponseError::from(
            error_code,
        )))
    }
}

pub(crate) async fn write_area_multi(
    client: &mut S7Client,
    area: Area,
    info: Vec<S7WriteAccess<'_>>,
) -> Result<Vec<Result<(), Error>>, Error> {
    // Each PDU (TPKT Header + COTP Header + S7Header + S7Parameters + S7Data) must not exceed the maximum PDU length (bytes) negotiated with the
    // PLC during connection.
    // Moreover we must ensure that a "finite" number of items is send per PDU. If the command size does not fit in one PDU
    // then it must be split across more subsequent PDU.

    assert_pdu_size_for_write(&info, client.pdu_length.into())?;

    // build request
    let request_params = BytesMut::from(ReadWriteParams::build_write(
        info.iter()
            .map(|info| {
                RequestItem::build(
                    area,
                    info.db_number(),
                    info.start(),
                    info.data_type(),
                    info.len(),
                )
            })
            .collect::<Result<Vec<RequestItem>, Error>>()?,
    ));
    // build data items
    let data_items = info
        .iter()
        .map(|info| DataItem::build_write2(info.data_type().into(), info.data()))
        .collect::<Result<Vec<DataItem<'_>>, Error>>()?
        .into_iter()
        .flat_map(BytesMut::from)
        .collect::<BytesMut>();

    // create data buffer
    let mut bytes = BytesMut::new();

    let req_header = S7ProtocolHeader::build_request(
        &mut client.pdu_number,
        request_params.len(),
        data_items.len(),
    )?;
    bytes.put(BytesMut::from(req_header));
    bytes.put(request_params);
    bytes.put(data_items);

    let mut response = exchange_buffer(&mut client.connection, bytes).await?;

    // check if s7 header is ack with data and check for errors
    // check if pdu of response matches request pdu
    let resp_header = S7ProtocolHeader::try_from(&mut response)?;
    resp_header
        .is_ack()?
        .is_current_pdu_response(client.pdu_number)?;

    // Check for errors
    if resp_header.has_error() {
        let (class, code) = resp_header.get_errors();
        return Err(Error::S7ProtocolError(S7ProtocolError::from_codes(
            class, code,
        )));
    }

    let read_params = ReadWriteParams::from(&mut response);

    Ok((0..read_params.item_count)
        .map(|_| {
            let error_code = response.get_u8();
            // 255 signals everything went alright
            if error_code == 255 {
                Ok(())
            } else {
                Err(Error::DataItemError(S7DataItemResponseError::from(
                    error_code,
                )))
            }
        })
        .collect::<Vec<Result<(), Error>>>())
}
