use bytes::{Buf, BufMut, BytesMut};

use super::request_item::RequestItem;

#[derive(Debug)]
pub(in crate::s7_protocol) struct ReadWriteParams {
    pub(in crate::s7_protocol) function_code: u8, // Constant value of 0x04 for read or 0x05 for write Jobs and replies.
    pub(in crate::s7_protocol) item_count: u8,    // Number of following Request Item structures.
    pub(in crate::s7_protocol) request_item: Option<Vec<RequestItem>>, // This structure is used to address the actual variables,
                                                                       // its length and fields depend on the type of addressing being used.
                                                                       // These items are only present in the Job request and are emitted from the
                                                                       // corresponding Ack Data no matter what the addressing mode is or whether it is
                                                                       // a read or write request.
}

impl ReadWriteParams {
    pub(in crate::s7_protocol) fn len() -> usize {
        2
    }
}

impl From<&mut BytesMut> for ReadWriteParams {
    fn from(bytes: &mut BytesMut) -> Self {
        Self {
            function_code: bytes.get_u8(),
            item_count: bytes.get_u8(),
            request_item: None,
        }
    }
}

impl From<ReadWriteParams> for BytesMut {
    fn from(req_item: ReadWriteParams) -> BytesMut {
        let mut bytes = BytesMut::new();
        bytes.put_u8(req_item.function_code);
        bytes.put_u8(req_item.item_count);
        if let Some(items) = req_item.request_item {
            for item in &items {
                bytes.put(BytesMut::from(*item));
            }
        };

        bytes
    }
}
