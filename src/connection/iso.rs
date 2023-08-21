use std::convert::TryFrom;
use std::mem;

use bytes::{Buf, BufMut, BytesMut};

use crate::errors::{Error, IsoError};

// PDU Type constants (Code + Credit)
const PDU_TYPE_CR: u8 = 224; // Connection request (0xE0)
pub(crate) const PDU_TYPE_CC: u8 = 208; // Connection confirm (0xD0)
const PDU_TYPE_DR: u8 = 128; // Disconnect request (0x80)
pub(crate) const PDU_TYPE_DC: u8 = 192; // Disconnect confirm (0xC0)
pub(crate) const PDU_TYPE_DT: u8 = 240; // Data transfer (0xF0)

const PDU_EOT: u8 = 128; // End of Transmission Packet (0x80) (This packet is complete)

const SRC_REF: u16 = 0x0100; // RFC0983 states that SrcRef and DetRef should be 0
                             // and, in any case, they are ignored.
                             // S7 instead requires a number != 0
                             // Libnodave uses 0x0100
                             // S7Manager uses 0x0D00
                             // TIA Portal V12 uses 0x1D00
                             // WinCC     uses 0x0300
                             // Seems that every non zero value is good enough...
const DST_REF: u16 = 0x0000;
const SRC_TSAP: u16 = 0x0100;

pub(crate) const ISO_TCP_VERSION: u8 = 3; // RFC 1006

// Client Connection Type
#[allow(dead_code)]
pub(crate) enum ConnectionType {
    /// Connect to the PLC programming console (Programmiergeräte)
    PG = 1,
    /// Connect to the PLC Siemens HMI panel
    OP = 2,
    /// Basic connection for generic data transfer connection
    /// 14 Basic connections
    Basic = 3,
}

struct TSAPInfo {
    rack: u8,
    slot: u8,
}

/// Supported PLC devices from the S7 family
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum S7Types {
    /// Choose this if you want to connect to a S7 200
    S7200,
    /// Choose this if you want to connect to a S7 300
    S7300,
    /// Choose this if you want to connect to a S7 400
    S7400,
    /// Choose this if you want to connect to a S7 1200.
    ///
    /// You need to activate the [PUT/GET communication](https://cache.industry.siemens.com/dl/files/115/82212115/att_108330/v2/82212115_s7_communication_s7-1500_en.pdf) method in order for this to work
    S71200,
    /// Choose this if you want to connect to a S7 1500.
    ///
    /// You need to activate the [PUT/GET communication](https://cache.industry.siemens.com/dl/files/115/82212115/att_108330/v2/82212115_s7_communication_s7-1500_en.pdf) method in order for this to work
    S71500,
}

impl S7Types {
    fn to_tsap_info(self) -> TSAPInfo {
        match self {
            Self::S7200 | Self::S7300 | Self::S7400 => TSAPInfo { rack: 0, slot: 2 },
            Self::S71200 | Self::S71500 => TSAPInfo { rack: 0, slot: 0 },
        }
    }
}

struct Tsap {}
impl Tsap {
    #[allow(clippy::cast_possible_truncation)]
    fn build(s7_type: S7Types) -> Vec<u8> {
        let tsap_info = s7_type.to_tsap_info();
        let dst_tsap = ((ConnectionType::Basic as u16) << 8)
            + (u16::from(tsap_info.rack) * 0x20)
            + u16::from(tsap_info.slot);
        vec![
            0xC1,                  // code that identifies source TSAP
            2,                     // source TSAP Len
            (SRC_TSAP >> 8) as u8, // HI part
            SRC_TSAP as u8,        // LO part
            0xC2,                  // code that identifies dest TSAP
            2,                     // dest TSAP Len
            (dst_tsap >> 8) as u8, // HI part
            dst_tsap as u8,        // LO part
        ]
    }
}

/// TPKT Header - ISO on TCP - RFC 1006 (4 bytes)
#[derive(Debug, Copy, Clone)]
pub struct TTPKTHeader {
    version: u8,     // Always 3 for RFC 1006
    reserved: u8,    // 0
    pub length: u16, // Packet length : min 7 max 65535
}

impl TTPKTHeader {
    pub(crate) fn build(length: u16) -> Self {
        Self {
            version: ISO_TCP_VERSION,
            reserved: 0,
            length,
        }
    }

    pub(crate) fn len() -> u8 {
        4
    }
}

impl TryFrom<&mut BytesMut> for TTPKTHeader {
    type Error = Error;

    fn try_from(bytes: &mut BytesMut) -> Result<Self, Self::Error> {
        // check if there are enough bytes for a header
        if bytes.len() >= usize::from(Self::len()) {
            Ok(Self {
                version: bytes.get_u8(),
                reserved: bytes.get_u8(),
                length: bytes.get_u16(),
            })
        } else {
            Err(Error::ISOResponse(IsoError::ShortPacket))
        }
    }
}

impl From<TTPKTHeader> for BytesMut {
    fn from(header: TTPKTHeader) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(4);
        bytes.put_u8(header.version);
        bytes.put_u8(header.reserved);
        bytes.put_u16(header.length);

        bytes
    }
}

impl From<TTPKTHeader> for Vec<u8> {
    fn from(header: TTPKTHeader) -> Vec<u8> {
        let mut vec = vec![header.version, header.reserved];
        vec.append(&mut header.length.to_be_bytes().to_vec());
        vec
    }
}

#[derive(Debug)]
struct COTPParams {
    pdu_size_code: u8,
    pdu_size_len: u8,
    pdu_size_val: u8,
    tsap: Vec<u8>, // We don't know in advance these fields....
}

impl TryFrom<&mut BytesMut> for COTPParams {
    type Error = Error;

    fn try_from(bytes: &mut BytesMut) -> Result<Self, Self::Error> {
        // check if there are enough bytes for a header
        if bytes.len() >= 3 {
            Ok(Self {
                pdu_size_code: bytes.get_u8(),
                pdu_size_len: bytes.get_u8(),
                pdu_size_val: bytes.get_u8(),
                tsap: bytes.to_vec(),
            })
        } else {
            Err(Error::ISOResponse(IsoError::ShortPacket))
        }
    }
}

impl From<COTPParams> for Vec<u8> {
    fn from(params: COTPParams) -> Vec<u8> {
        let mut vec = vec![
            params.pdu_size_code,
            params.pdu_size_len,
            params.pdu_size_val,
        ];
        let mut tsap = params.tsap;
        vec.append(&mut tsap);
        vec
    }
}

// COTP Header for CONNECTION REQUEST/CONFIRM - DISCONNECT REQUEST/CONFIRM
#[derive(Debug)]
pub(crate) struct COTPConnection {
    header_length: u8, // Header length : initialized to 6 (length without params - 1)
    // descending classes that add values in params field must update it.
    pdu_type: u8, // 0xE0 Connection request
    // 0xD0 Connection confirm
    // 0x80 Disconnect request
    // 0xDC Disconnect confirm
    dst_ref: u16, // Destination reference : Always 0x0000
    src_ref: u16, // Source reference : Always 0x0000
    co_r: u8,     // If the telegram is used for Connection request/Confirm,
    // the meaning of this field is CLASS+OPTION :
    //   Class (High 4 bits) + Option (Low 4 bits)
    //   Class : Always 4 (0100) but is ignored in input (RFC States this)
    //   Option : Always 0, also this in ignored.
    // Parameter data : depending on the protocol implementation.
    // ISO 8073 define several type of parameters, but RFC 1006 recognizes only
    // TSAP related parameters and PDU size.  See RFC 0983 for more details.
    cotp_params: COTPParams,
    /* Other params not used here, list only for completeness
        ACK_TIME     	   = 0x85,  1000 0101 Acknowledge Time
        RES_ERROR    	   = 0x86,  1000 0110 Residual Error Rate
        PRIORITY           = 0x87,  1000 0111 Priority
        TRANSIT_DEL  	   = 0x88,  1000 1000 Transit Delay
        THROUGHPUT   	   = 0x89,  1000 1001 Throughput
        SEQ_NR       	   = 0x8A,  1000 1010 Subsequence Number (in AK)
        REASSIGNMENT 	   = 0x8B,  1000 1011 Reassignment Time
        FLOW_CNTL    	   = 0x8C,  1000 1100 Flow Control Confirmation (in AK)
        TPDU_SIZE    	   = 0xC0,  1100 0000 TPDU Size
        SRC_TSAP     	   = 0xC1,  1100 0001 TSAP-ID / calling TSAP ( in CR/CC )
        DST_TSAP     	   = 0xC2,  1100 0010 TSAP-ID / called TSAP
        CHECKSUM     	   = 0xC3,  1100 0011 Checksum
        VERSION_NR   	   = 0xC4,  1100 0100 Version Number
        PROTECTION   	   = 0xC5,  1100 0101 Protection Parameters (user defined)
        OPT_SEL            = 0xC6,  1100 0110 Additional Option Selection
        PROTO_CLASS  	   = 0xC7,  1100 0111 Alternative Protocol Classes
        PREF_MAX_TPDU_SIZE = 0xF0,  1111 0000
        INACTIVITY_TIMER   = 0xF2,  1111 0010
        ADDICC             = 0xe0   1110 0000 Additional Information on Connection Clearing
    */
}

#[derive(Debug)]
pub(crate) struct COTPDisconnect {
    header_length: u8,
    pdu_type: u8,
    dst_ref: u16, // Destination reference : Always 0x0000
    src_ref: u16, // Source reference : Always 0x0000
    reason: u8,   // If the telegram is used for Disconnect request,
                  // the meaning of this field is REASON :
                  //    1     Congestion at TSAP
                  //    2     Session entity not attached to TSAP
                  //    3     Address unknown (at TCP connect time)
                  //  128+0   Normal disconnect initiated by the session
                  //          entity.
                  //  128+1   Remote transport entity congestion at connect
                  //          request time
                  //  128+3   Connection negotiation failed
                  //  128+5   Protocol Error
                  //  128+8   Connection request refused on this network
                  //          connection
}

impl COTPDisconnect {
    pub(crate) fn len() -> usize {
        7
    }
}

impl CoTp for COTPConnection {
    fn get_pdu_type(&self) -> u8 {
        self.pdu_type
    }

    fn req_ok(&self) -> Result<(), Error> {
        if self.validate_expected_pdu_type(PDU_TYPE_CC) {
            Ok(())
        } else {
            Err(Error::ISOResponse(IsoError::InvalidPDU))
        }
    }
}

impl TryFrom<&mut BytesMut> for COTPConnection {
    type Error = Error;

    fn try_from(bytes: &mut BytesMut) -> Result<Self, Self::Error> {
        // check if there are enough bytes for a header
        if bytes.len() >= 7 {
            Ok(Self {
                header_length: bytes.get_u8(),
                pdu_type: bytes.get_u8(),
                dst_ref: bytes.get_u16(),
                src_ref: bytes.get_u16(),
                co_r: bytes.get_u8(),
                cotp_params: COTPParams::try_from(bytes)?,
            })
        } else {
            Err(Error::ISOResponse(IsoError::ShortPacket))
        }
    }
}

impl From<COTPConnection> for Vec<u8> {
    fn from(cotp: COTPConnection) -> Vec<u8> {
        let mut vec = vec![cotp.header_length, cotp.pdu_type];
        vec.append(&mut cotp.dst_ref.to_be_bytes().to_vec());
        vec.append(&mut cotp.src_ref.to_be_bytes().to_vec());
        vec.push(cotp.co_r);
        vec.append(&mut cotp.cotp_params.into());
        vec
    }
}

impl TryFrom<&mut BytesMut> for COTPDisconnect {
    type Error = Error;

    fn try_from(bytes: &mut BytesMut) -> Result<Self, Self::Error> {
        // check if there are enough bytes for a header
        if bytes.len() >= Self::len() {
            Ok(Self {
                header_length: bytes.get_u8(),
                pdu_type: bytes.get_u8(),
                dst_ref: bytes.get_u16(),
                src_ref: bytes.get_u16(),
                reason: bytes.get_u8(),
            })
        } else {
            Err(Error::ISOResponse(IsoError::ShortPacket))
        }
    }
}

impl From<COTPDisconnect> for Vec<u8> {
    fn from(cotp: COTPDisconnect) -> Vec<u8> {
        let mut vec = vec![cotp.header_length, cotp.pdu_type];
        vec.append(&mut cotp.dst_ref.to_be_bytes().to_vec());
        vec.append(&mut cotp.src_ref.to_be_bytes().to_vec());
        vec.push(cotp.reason);
        vec
    }
}

impl CoTp for COTPDisconnect {
    fn get_pdu_type(&self) -> u8 {
        self.pdu_type
    }

    fn req_ok(&self) -> Result<(), Error> {
        if self.validate_expected_pdu_type(PDU_TYPE_DC) {
            Ok(())
        } else {
            Err(Error::ISOResponse(IsoError::InvalidPDU))
        }
    }
}

// COTP Header for DATA EXCHANGE
#[derive(Debug)]
pub(crate) struct COTPData {
    header_length: u8, // Header length : 3 for this header - 1
    pdu_type: u8,      // 0xF0 for this header
    eot_num: u8,       // EOT (bit 7) + PDU Number (bits 0..6)
                       // EOT = 1 -> End of Transmission Packet (This packet is complete)
                       // PDU Number : Always 0
}

impl COTPData {
    pub(crate) fn len() -> u8 {
        3
    }

    pub(crate) fn build() -> Self {
        COTPData {
            header_length: COTPData::len() - 1,
            pdu_type: PDU_TYPE_DT,
            eot_num: PDU_EOT,
        }
    }

    pub(crate) fn is_last(&self) -> bool {
        self.eot_num == PDU_EOT
    }
}

impl CoTp for COTPData {
    fn get_pdu_type(&self) -> u8 {
        self.pdu_type
    }

    fn req_ok(&self) -> Result<(), Error> {
        if self.validate_expected_pdu_type(PDU_TYPE_DT) {
            Ok(())
        } else {
            Err(Error::ISOResponse(IsoError::InvalidPDU))
        }
    }
}

impl TryFrom<&[u8]> for COTPData {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        match bytes.len() {
            3 => Ok(Self {
                header_length: bytes[0],
                pdu_type: bytes[1],
                eot_num: bytes[2],
            }),
            _ => Err(Error::TryFrom(
                bytes.to_vec(),
                "Could not convert bytes into TPKT Header".to_string(),
            )),
        }
    }
}

impl TryFrom<&mut BytesMut> for COTPData {
    type Error = Error;

    fn try_from(bytes: &mut BytesMut) -> Result<Self, Self::Error> {
        // check if there are enough bytes for a header
        if bytes.len() >= usize::from(Self::len()) {
            Ok(Self {
                header_length: bytes.get_u8(),
                pdu_type: bytes.get_u8(),
                eot_num: bytes.get_u8(),
            })
        } else {
            Err(Error::ISOResponse(IsoError::ShortPacket))
        }
    }
}

impl From<COTPData> for BytesMut {
    fn from(cotp: COTPData) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(3);
        bytes.put_u8(cotp.header_length);
        bytes.put_u8(cotp.pdu_type);
        bytes.put_u8(cotp.eot_num);

        bytes
    }
}

pub(super) trait CoTp {
    fn validate_expected_pdu_type(&self, expected_type: u8) -> bool {
        expected_type == self.get_pdu_type()
    }

    fn req_ok(&self) -> Result<(), Error>;
    fn get_pdu_type(&self) -> u8;
}

#[derive(Debug)]
pub(super) struct IsoControlPDU {
    tpkt_header: TTPKTHeader,       // TPKT Header
    cotp_co_header: COTPConnection, // COPT Header for CONNECTION stuffs
}

impl IsoControlPDU {
    pub(crate) fn build(pdu_size: u32, s7_type: S7Types) -> Self {
        // Params length
        let par_len = 11_u8; // 2 Src TSAP (Code+field Len)      +
                             // 2 Src TSAP len                   +
                             // 2 Dst TSAP (Code+field Len)      +
                             // 2 Src TSAP len                   +
                             // 3 PDU size (Code+field Len+Val)  = 11
                             // Telegram length
        let iso_len = TTPKTHeader::len()     // TPKT Header
                    + 7                          // COTP Header Size without params
                    + par_len; // COTP params

        let cotp = COTPConnection {
            cotp_params: COTPParams {
                pdu_size_code: 0xC0, // code that identifies TPDU size
                pdu_size_len: 0x01,  // 1 byte this field
                pdu_size_val: match pdu_size {
                    128 => 0x07,
                    256 => 0x08,
                    512 => 0x09,
                    1024 => 0x0A,
                    4096 => 0x0C,
                    8192 => 0x0D,
                    // 2048 => 0x0B,
                    _ => 0x0B, // Our Default
                },
                tsap: Tsap::build(s7_type),
            },
            header_length: par_len + 6, // <-- 6 = 7 - 1 (COTP Header size - 1)
            pdu_type: PDU_TYPE_CR,      // Connection Request
            dst_ref: DST_REF,           // Destination reference
            src_ref: SRC_REF,           // Source reference
            co_r: 0x00, // Class + Option : RFC0983 states that it must be always 0x40
                        // but for some equipment (S7) must be 0 in contrast to specifications !!!
        };

        let header = TTPKTHeader {
            version: ISO_TCP_VERSION,
            reserved: 0,
            length: u16::from(iso_len),
        };

        IsoControlPDU {
            tpkt_header: header,
            cotp_co_header: cotp,
        }
    }
}

impl From<IsoControlPDU> for Vec<u8> {
    fn from(control_pdu: IsoControlPDU) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.append(&mut control_pdu.tpkt_header.into());
        vec.append(&mut control_pdu.cotp_co_header.into());
        vec
    }
}

pub(super) struct IsoDisconnect {
    tpkt_header: TTPKTHeader,       // TPKT Header
    cotp_co_header: COTPDisconnect, // COPT Header for DISCONNECT stuffs
}

impl IsoDisconnect {
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn build() -> Self {
        let iso_len = mem::size_of::<TTPKTHeader>() - 1     // TPKT Header
                    + mem::size_of::<COTPDisconnect>(); // COTP Header Size without params

        let cotp = COTPDisconnect {
            header_length: 6,
            pdu_type: PDU_TYPE_DR,
            dst_ref: DST_REF,
            src_ref: SRC_REF,
            reason: 128, // normal disconnect
        };
        let header = TTPKTHeader {
            version: ISO_TCP_VERSION,
            reserved: 0,
            length: iso_len as u16,
        };

        Self {
            tpkt_header: header,
            cotp_co_header: cotp,
        }
    }
}

impl From<IsoDisconnect> for Vec<u8> {
    fn from(control_pdu: IsoDisconnect) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.append(&mut control_pdu.tpkt_header.into());
        vec.append(&mut control_pdu.cotp_co_header.into());
        vec
    }
}
