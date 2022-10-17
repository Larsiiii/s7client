use std::borrow::Cow;

use bytes::{Buf, BufMut, BytesMut};

use crate::{
    errors::{Error, S7DataItemResponseError},
    s7_protocol::types::DataItemTransportSize,
};

#[derive(Debug)]
pub(in crate::s7_protocol) struct DataItem<'a> {
    pub(in crate::s7_protocol) error_code: u8, // The return value of the operation, 0xff signals success.
    // In the Write Request message this field is always set to zero.
    pub(in crate::s7_protocol) var_type: u8, // See RequestItem
    pub(in crate::s7_protocol) count: u16,   // See RequestItem but size is given in bits
    pub(in crate::s7_protocol) data: Cow<'a, [u8]>, // This field contains the actual value of the addressed variable, its size is len(variable) * count.
}

impl DataItem<'_> {
    pub(in crate::s7_protocol) fn header_len() -> usize {
        4
    }
}

impl TryFrom<&mut BytesMut> for DataItem<'_> {
    type Error = Error;

    fn try_from(bytes: &mut BytesMut) -> Result<Self, Self::Error> {
        // try to convert data item if it is long enough
        match bytes.len() {
            x if x >= 4 => {
                let error_code = bytes.get_u8();
                let var_type = bytes.get_u8();
                let count = bytes
                    .get_u16()
                    .checked_div(DataItemTransportSize::from(var_type).len())
                    .unwrap_or(0);
                let data = bytes.split_to(count as usize);

                // check for errors
                // 255 signals everything went alright
                if error_code != 255 {
                    return Err(Error::DataItemError(S7DataItemResponseError::from(
                        error_code,
                    )));
                }

                Ok(Self {
                    error_code,
                    var_type,
                    count,
                    data: Cow::from(data.to_vec()),
                })
            }
            _ => Err(Error::TryFrom(
                bytes.to_vec(),
                "Invalid length for data item".to_string(),
            )),
        }
    }
}

impl From<DataItem<'_>> for BytesMut {
    fn from(item: DataItem<'_>) -> BytesMut {
        let mut bytes = BytesMut::new();

        bytes.put_u8(item.error_code);
        bytes.put_u8(item.var_type);
        bytes.put_u16(item.count);
        bytes.put(item.data.as_ref());

        bytes
    }
}
