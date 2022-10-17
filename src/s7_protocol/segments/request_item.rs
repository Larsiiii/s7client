use bytes::{BufMut, BytesMut};

use crate::{
    errors::Error,
    s7_protocol::types::{Area, S7DataTypes, SPEC_TYPE_READ_WRITE, SYNTAX_ID_ANY_TYPE},
};

#[derive(Debug, Copy, Clone)]
pub(in crate::s7_protocol) struct RequestItem {
    pub(crate) specification_type: u8, // This field determines the main type of the item struct, for read/write messages
    // it always has the value 0x12 which stands for Variable Specification.
    pub(crate) item_length: u8, // The length of the rest of this item. Length Request Items - 2 bytes.
    pub(crate) syntax_id: u8, // This field determines the addressing mode and the format of the rest of the item structure.
    // It has the constant value of 0x10 for the any-type addressing.
    pub(crate) var_type: u8, // Is is used to determine the type and length of the variable (usual S7 types are used
    // such as REAL, BIT, BYTE, WORD, DWORD, COUNTER, …).
    pub(crate) data_length: u16, // It is possible to select an entire array of similar variables with a single item struct.
    // These variables must have the same type, and must be consecutive in the memory and
    // the count field determines the size of this array. It is set to one for single variable
    // read or write.
    pub(crate) db_number: u16, // The address of the database, it is ignored if the area is not set to DB (see next field).
    pub(crate) area: u8, // Selects the memory area of the addressed variable. See enum Area...
    pub(crate) address: u32, // Contains the offset of the addressed variable in the selected memory area.
                             // Essentially, the addresses are translated to bit offsets and encoded on 3 bytes in
                             // network (big endian) byte order. In practice, the most significant 5 bits are never used
                             // since the address space is smaller than that.
                             // As an example DBX40.3 would be 0x000143 which is 40 * 8 + 3.
}

impl RequestItem {
    pub(in crate::s7_protocol) fn len() -> usize {
        // address is only 3 bytes long (not u32 as in struct)
        12
    }

    pub(in crate::s7_protocol) fn build(
        area: Area,
        db_number: u16,
        start: u32,
        data_type: S7DataTypes,
        length: usize,
    ) -> Result<Self, Error> {
        Ok(Self {
            specification_type: SPEC_TYPE_READ_WRITE,
            item_length: 10, //mem::size_of::<RequestItem>() as u8 - 2,
            syntax_id: SYNTAX_ID_ANY_TYPE,
            var_type: data_type as u8,
            data_length: u16::try_from(length).map_err(|_| Error::TooManyItemsInOneRequest)?,
            area: area as u8,
            db_number,
            address: match data_type {
                // Adjusts the offset
                S7DataTypes::S7BIT | S7DataTypes::S7COUNTER | S7DataTypes::S7TIMER => start,
                _ => start * 8,
            },
        })
    }

    pub(in crate::s7_protocol) fn address_to_bytes(&self) -> BytesMut {
        let mut address = self.address;
        let address_byte3 = (address & 0x0FF) as u8;
        address >>= 8;
        let address_byte2 = (address & 0x0FF) as u8;
        address >>= 8;
        let address_byte1 = (address & 0x0FF) as u8;

        let mut bytes = BytesMut::with_capacity(3);
        bytes.extend_from_slice(&[address_byte1, address_byte2, address_byte3]);
        bytes
    }
}

impl From<RequestItem> for BytesMut {
    fn from(req_item: RequestItem) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(12);
        bytes.put_u8(req_item.specification_type);
        bytes.put_u8(req_item.item_length);
        bytes.put_u8(req_item.syntax_id);
        bytes.put_u8(req_item.var_type);
        bytes.put_u16(req_item.data_length);
        bytes.put_u16(req_item.db_number);
        bytes.put_u8(req_item.area);
        bytes.put(req_item.address_to_bytes());

        bytes
    }
}
