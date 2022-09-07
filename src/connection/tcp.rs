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
    // send connection request
    let iso: Vec<u8> = IsoControlPDU::build(1024, s7_type).into();
    tcp_client.write_all(&iso).await?;

    // Get response TTPKT Header
    let packet_header = read_tpkt_header(tcp_client).await?;
    let tpkt_data = read_tpkt_data(tcp_client, packet_header.length).await?;
    let cotp_connection: COTPConnection = tpkt_data.try_into()?;
    cotp_connection.req_ok()?;

    negotiate_connection_params(tcp_client).await
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

    // if not last wait for others till last
    while !is_last {
        let header = read_tpkt_header(conn).await?;
        let iso_cotp_data = read_tpkt_data(conn, header.length).await?;
        let cotp = COTPData::try_from(iso_cotp_data[..3].to_vec())?;

        cotp.req_ok()?;
        data_buffer.append(&mut iso_cotp_data[3..].to_vec());
        is_last = cotp.is_last();
    }

    Ok(data_buffer)
}

pub(crate) async fn exchange_buffer(
    conn: &mut TcpStream,
    data: &mut Vec<u8>,
) -> Result<Vec<u8>, Error> {
    // Send data to PLC with timeout
    match timeout(DATA_SEND_AND_RECEIVE_TIMEOUT, send_buffer(conn, data)).await {
        Ok(_) => {}
        Err(_) => return Err(Error::DataExchangeTimedOut),
    };

    // Receive data from PLC with timeout
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
