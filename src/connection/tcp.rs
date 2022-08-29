use std::convert::{TryFrom, TryInto};
use std::mem;
use std::time::Duration;
// use std::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::iso::{COTPConnection, COTPData, CoTp, IsoControlPDU, TTPKTHeader};
use crate::connection::iso::{COTPDisconnect, IsoDisconnect};
use crate::errors::{Error, IsoError};
use crate::s7_protocol::header::S7ProtocolHeader;
use crate::s7_protocol::negotiate::{NegotiatePDUParameters, S7Negotiation};
use crate::s7_protocol::S7Protocol;
use crate::S7Types;

const DATA_SEND_AND_RECEIVE_TIMEOUT: Duration = Duration::from_secs(4);

pub(crate) async fn connect(
    tcp_client: &mut TcpStream,
    s7_type: S7Types,
) -> Result<NegotiatePDUParameters, Error> {
    // let mut tcp_client = TcpStream::connect(format!("127.0.0.1:{}", TCP_PORT))?;

    // // match tcp_client {
    // //     Ok(mut stream) => {

    // tcp_client.set_read_timeout(Some(TIMEOUT))?;
    // tcp_client.set_write_timeout(Some(TIMEOUT))?;
    // println!("Successfully connected to server in port 102");

    // send connection request
    let iso: Vec<u8> = IsoControlPDU::build(1024, s7_type).into();
    tcp_client.write_all(&iso).await?;

    // Get response TTPKT Header
    let packet_header = read_tpkt_header(tcp_client).await?;
    let tpkt_data = read_tpkt_data(tcp_client, packet_header.length).await?;
    let cotp_connection: COTPConnection = tpkt_data.try_into()?;
    cotp_connection.req_ok()?;

    negotiate_connection_params(tcp_client).await

    // match read_tpkt_data(&mut tcp_client, packet_header.length) {
    //     Ok(data) => {
    //         let cotp_connection: Result<COTPConnection, ()> = data.try_into();
    //         cotp_connection.unwrap().req_ok()?;
    //         Ok(tcp_client)
    //         // if cotp_connection.unwrap().req_ok() {
    //         //     Ok(tcp_client)
    //         // } else {
    //         //     // TODO
    //         //     Err(Error::Connection("jkdlffa".to_string()))
    //         // }
    //     },
    //     // TODO
    //     Err(e) => {println!("Failed to receive data"); Err(Error::Connection("jkdlffa".to_string()))}
    // }
    // },
    // Err(_) => {
    //     println!("Failed to connect");
    //     Err(())
    // }
    // }
}

pub(crate) async fn disconnect(tcp_client: &mut TcpStream) -> Result<(), Error> {
    let iso: Vec<u8> = IsoDisconnect::build().into();
    tcp_client.write_all(&iso).await?;

    // Get response TTPKT Header
    let packet_header = read_tpkt_header(tcp_client).await?;
    let tpkt_data = read_tpkt_data(tcp_client, packet_header.length).await?;
    let cotp_disconnect: COTPDisconnect = tpkt_data.try_into()?;
    cotp_disconnect.req_ok()?;
    Ok(())
}

pub(crate) async fn negotiate_connection_params(
    conn: &mut TcpStream,
) -> Result<NegotiatePDUParameters, Error> {
    let mut negotiation_params: Vec<u8> = S7Negotiation::build().into();
    let exchanged_data = exchange_buffer(conn, &mut negotiation_params).await?;
    S7ProtocolHeader::try_from(exchanged_data[0..12].to_vec())?.is_ack_with_data()?;
    let params = NegotiatePDUParameters::try_from(exchanged_data[12..].to_vec())?;
    Ok(params)

    // match exchange_buffer(conn, &mut negotiation_params) {
    //     Ok(data) => {
    //         let s7_header = S7ProtocolHeader::try_from(data[0..12].to_vec()).unwrap();
    //         let params = NegotiatePDUParameters::try_from(data[12..].to_vec()).unwrap();
    //         println!("PDU LENGTH: {:?}", params.pdu_length);
    //         println!("MAX AMQ CALLER: {:?}", params.max_amq_caller);
    //         println!("MAX AMQ CALLE: {:?}", params.max_amq_calle);

    //         Ok(params)

    //         let mut reader = crate::s7_protocol::read_area::Reader {};
    //         let mut writer = crate::s7_protocol::write_area::Writer {};
    //         println!(
    //             "{:?}",
    //             reader.read_area(
    //                 conn,
    //                 crate::s7_protocol::types::Area::DataBlock,
    //                 1,
    //                 30,
    //                 5,
    //                 crate::s7_protocol::types::S7DataTypes::S7BYTE
    //             )
    //         );
    //         println!(
    //             "{:?}",
    //             writer.write_area(
    //                 conn,
    //                 crate::s7_protocol::types::Area::DataBlock,
    //                 1,
    //                 30,
    //                 crate::s7_protocol::types::S7DataTypes::S7INT,
    //                 &mut (2_i16).to_be_bytes().to_vec()
    //             )
    //         );
    //     }
    //     Err(_) => {}
    // };
}

pub(crate) async fn send_buffer(conn: &mut TcpStream, data: &mut Vec<u8>) -> Result<(), Error> {
    // Telegram length
    let iso_len = mem::size_of::<TTPKTHeader>()     // TPKT Header
                + mem::size_of::<COTPData>()        // COTP Header Size
                + data.len(); // S7 params
    let tpkt_header = TTPKTHeader::build(iso_len as u16);
    let cotp = COTPData::build();
    let mut data_vec: Vec<u8> = Vec::new();
    // add TPKT Header
    data_vec.append(&mut tpkt_header.into());
    // add COTP Header
    data_vec.append(&mut cotp.into());
    // add data
    data_vec.append(data);
    conn.write_all(&data_vec).await?;
    Ok(())
}

pub(crate) async fn recv_buffer(conn: &mut TcpStream) -> Result<Vec<u8>, Error> {
    let mut data_buffer: Vec<u8> = Vec::new();
    let mut is_last: bool = false;
    // TODO if not last wait for others till last
    // TODO Timeout if last is not coming
    while !is_last {
        let header = read_tpkt_header(conn).await?;
        let iso_cotp_data = read_tpkt_data(conn, header.length).await?;
        let cotp = COTPData::try_from(iso_cotp_data[..3].to_vec())?;

        cotp.req_ok()?;
        data_buffer.append(&mut iso_cotp_data[3..].to_vec());
        is_last = cotp.is_last();
    }

    Ok(data_buffer)
    // match read_tpkt_header(conn) {
    //     Ok(header) => {
    // match read_tpkt_data(conn, header.length) {
    // //     Ok(iso_cotp_data) => {
    //         match COTPData::try_from(iso_cotp_data[..3].to_vec()) {
    //             Ok(cotp) => {
    // if cotp.req_ok() && cotp.is_last() {
    //     Ok(iso_cotp_data[3..].to_vec())
    // } else { Err(()) }
    //     },
    //     Err(_) => Err(())
    // }
    //     },
    //     Err(_) => Err(())
    // }
    //     },
    //     Err(_) => Err(())
    // }
}

pub(crate) async fn exchange_buffer(
    conn: &mut TcpStream,
    data: &mut Vec<u8>,
) -> Result<Vec<u8>, Error> {
    // TODO implement timeout on operations
    // Send data to PLC
    match timeout(DATA_SEND_AND_RECEIVE_TIMEOUT, send_buffer(conn, data)).await {
        Ok(_) => {}
        Err(_) => return Err(Error::DataExchangeTimedOut),
    };

    // Receive data from PLC
    match timeout(DATA_SEND_AND_RECEIVE_TIMEOUT, recv_buffer(conn)).await {
        Ok(data) => Ok(data?),
        Err(_) => Err(Error::DataExchangeTimedOut),
    }
}

async fn read_tpkt_header(conn: &mut TcpStream) -> Result<TTPKTHeader, Error> {
    // Get response TTPKT Header
    let mut data = [0_u8; mem::size_of::<TTPKTHeader>()];
    conn.read_exact(&mut data).await?;
    TTPKTHeader::try_from(data.to_vec())
}

async fn read_tpkt_data(conn: &mut TcpStream, length: u16) -> Result<Vec<u8>, Error> {
    let reader_length = length as usize - mem::size_of::<TTPKTHeader>();
    let mut data = Vec::<u8>::new();

    data.resize(reader_length, 0);
    match conn.read_exact(&mut data).await {
        Ok(_) => Ok(data),
        Err(_) => Err(Error::ISOResponse(IsoError::InvalidDataSize)),
    }
}
