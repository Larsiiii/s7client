use std::convert::TryFrom;

use bytes::{Buf, BufMut, BytesMut};

use crate::errors::{Error, S7DataItemResponseError};

pub(super) const READ_OPERATION: u8 = 0x04;
pub(super) const WRITE_OPERATION: u8 = 0x05;

pub(super) const SPEC_TYPE_READ_WRITE: u8 = 0x12;
pub(super) const SYNTAX_ID_ANY_TYPE: u8 = 0x10;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum Area {
    ProcessInput = 0x81,
    ProcessOutput = 0x82,
    /// Merkers are address registers within the CPU.
    /// The number of available flag bytes depends on the respective CPU and can be taken from the technical data.
    /// You can use flag bits, flag bytes, flag words or flag double words in a PLC program.
    Merker = 0x83,
    /// German thing, means building blocks
    /// This is your storage  
    DataBlock = 0x84,
    Counter = 0x1C,
    Timer = 0x1D,
    Unknown,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum S7DataTypes {
    S7BIT = 0x01,  // Bit (inside a word)
    S7BYTE = 0x02, // Byte (8 bit)
    S7CHAR = 0x03,
    S7WORD = 0x04, // Word (16 bit)
    S7INT = 0x05,
    S7DWORD = 0x06, // Double Word (32 bit)
    S7DINT = 0x07,
    S7REAL = 0x08,    // Real (32 bit float)
    S7COUNTER = 0x1C, // Counter (16 bit)
    S7TIMER = 0x1D,   // Timer (16 bit)
}

impl S7DataTypes {
    //dataSize to number of byte accordingly
    pub(crate) fn get_size(self) -> u32 {
        match self {
            Self::S7BIT | Self::S7BYTE | Self::S7CHAR => 1,
            Self::S7WORD | Self::S7INT | Self::S7COUNTER | Self::S7TIMER => 2,
            Self::S7DWORD | Self::S7DINT | Self::S7REAL => 4,
        }
    }
}

#[derive(Debug)]
pub(super) struct ReadWriteParams {
    pub(super) function_code: u8, // Constant value of 0x04 for read or 0x05 for write Jobs and replies.
    pub(super) item_count: u8,    // Number of following Request Item structures.
    pub(super) request_item: Option<Vec<RequestItem>>, // This structure is used to address the actual variables,
                                                       // its length and fields depend on the type of addressing being used.
                                                       // These items are only present in the Job request and are emitted from the
                                                       // corresponding Ack Data no matter what the addressing mode is or whether it is
                                                       // a read or write request.
}

impl ReadWriteParams {
    pub(super) fn len() -> usize {
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
            items.iter().for_each(|item| {
                bytes.put(BytesMut::from(*item));
            })
        };

        bytes
    }
}

#[derive(Debug, Copy, Clone)]
pub(super) struct RequestItem {
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
    pub(super) fn len() -> usize {
        // address is only 3 bytes long (not u32 as in struct)
        12
    }

    pub(super) fn build(
        area: Area,
        db_number: u16,
        start: u32,
        data_type: S7DataTypes,
        length: u16,
    ) -> Self {
        Self {
            specification_type: SPEC_TYPE_READ_WRITE,
            item_length: 10, //mem::size_of::<RequestItem>() as u8 - 2,
            syntax_id: SYNTAX_ID_ANY_TYPE,
            var_type: data_type as u8,
            data_length: length,
            area: area as u8,
            db_number,
            address: match data_type {
                // Adjusts the offset
                S7DataTypes::S7BIT | S7DataTypes::S7COUNTER | S7DataTypes::S7TIMER => start,
                _ => start * 8,
            },
        }
    }

    pub(super) fn address_to_bytes(&self) -> BytesMut {
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

#[derive(Debug)]
pub(crate) enum DataItemTransportSize {
    Null = 0x00,        // Null
    Bit = 0x03,         // Bit
    Byte = 0x04,        // Byte/Word/DWord
    Integer = 0x05,     // Integer
    Real = 0x07,        // Real
    OctetString = 0x09, // Octet String
}

impl From<u8> for DataItemTransportSize {
    fn from(val: u8) -> Self {
        match val {
            0x03 => Self::Bit,
            0x04 => Self::Byte,
            0x05 => Self::Integer,
            0x07 => Self::Real,
            0x09 => Self::OctetString,
            _ => Self::Null,
        }
    }
}

impl DataItemTransportSize {
    pub(crate) fn len(&self) -> u16 {
        match self {
            Self::Null => 0,
            Self::Bit => 1,
            Self::Byte | Self::Integer | Self::Real | Self::OctetString => 8,
        }
    }
}

impl From<S7DataTypes> for DataItemTransportSize {
    fn from(data_type: S7DataTypes) -> Self {
        match data_type {
            S7DataTypes::S7BIT => Self::Bit,
            S7DataTypes::S7BYTE
            | S7DataTypes::S7CHAR
            | S7DataTypes::S7WORD
            | S7DataTypes::S7DWORD
            | S7DataTypes::S7DINT
            | S7DataTypes::S7COUNTER
            | S7DataTypes::S7TIMER => Self::Byte,
            S7DataTypes::S7INT => Self::Integer,
            S7DataTypes::S7REAL => Self::Real,
        }
    }
}

#[derive(Debug)]
pub(super) struct DataItem {
    pub(super) error_code: u8, // The return value of the operation, 0xff signals success.
    // In the Write Request message this field is always set to zero.
    pub(super) var_type: u8,  // See RequestItem
    pub(super) count: u16,    // See RequestItem but size is given in bits
    pub(super) data: Vec<u8>, // This field contains the actual value of the addressed variable, its size is len(variable) * count.
}

impl TryFrom<&mut BytesMut> for DataItem {
    type Error = Error;

    fn try_from(bytes: &mut BytesMut) -> Result<Self, Self::Error> {
        // try to convert data item if it is long enough
        match bytes.len() {
            x if x > 4 => {
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
                    data: data.to_vec(),
                })
            }
            _ => Err(Error::TryFrom(
                bytes.to_vec(),
                "Invalid length for data item".to_string(),
            )),
        }
    }
}

impl From<DataItem> for BytesMut {
    fn from(item: DataItem) -> BytesMut {
        let mut bytes = BytesMut::new();

        bytes.put_u8(item.error_code);
        bytes.put_u8(item.var_type);
        bytes.put_u16(item.count);
        bytes.put(item.data.as_slice());

        bytes
    }
}
