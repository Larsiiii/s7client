use bytes::{BufMut, BytesMut};
use std::convert::TryFrom;
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
    let mut tpkt_data = read_tpkt_data(tcp_client, packet_header.length).await?;

    let cotp_connection = COTPConnection::try_from(&mut tpkt_data)?;
    cotp_connection.req_ok()?;

    negotiate_connection_params(tcp_client).await
}

pub(crate) async fn disconnect(tcp_client: &mut TcpStream) -> Result<(), Error> {
    let iso: Vec<u8> = IsoDisconnect::build().into();
    tcp_client.write_all(&iso).await?;

    // Get response TTPKT Header
    let packet_header = read_tpkt_header(tcp_client).await?;
    let mut tpkt_data = read_tpkt_data(tcp_client, packet_header.length).await?;

    let cotp_disconnect = COTPDisconnect::try_from(&mut tpkt_data)?;
    cotp_disconnect.req_ok()?;
    Ok(())
}

pub(crate) async fn negotiate_connection_params(
    conn: &mut TcpStream,
) -> Result<NegotiatePDUParameters, Error> {
    let negotiation_params = BytesMut::from(S7Negotiation::build());
    let mut exchanged_data = exchange_buffer(conn, negotiation_params).await?;

    S7ProtocolHeader::try_from(&mut exchanged_data)?.is_ack_with_data()?;
    let params = NegotiatePDUParameters::try_from(&mut exchanged_data)?;
    Ok(params)
}

pub(crate) async fn send_buffer(conn: &mut TcpStream, data: BytesMut) -> Result<(), Error> {
    // Telegram length
    let iso_len = mem::size_of::<TTPKTHeader>()     // TPKT Header
                + mem::size_of::<COTPData>()        // COTP Header Size
                + data.len(); // S7 params
    let tpkt_header = TTPKTHeader::build(iso_len as u16);
    let cotp = COTPData::build();

    // construct data
    let mut bytes = BytesMut::new();
    // add TPKT Header
    bytes.put(BytesMut::from(tpkt_header));
    // add COTP Header
    bytes.put(BytesMut::from(cotp));
    // add data
    bytes.put(data);

    // send data to plc
    conn.write_all(&bytes).await?;

    Ok(())
}

pub(crate) async fn recv_buffer(conn: &mut TcpStream) -> Result<BytesMut, Error> {
    let mut bytes = BytesMut::new();
    let mut is_last: bool = false;

    // if not last wait for others till last
    while !is_last {
        let header = read_tpkt_header(conn).await?;
        let mut iso_cotp_data = read_tpkt_data(conn, header.length).await?;
        let cotp = COTPData::try_from(&mut iso_cotp_data)?;

        cotp.req_ok()?;
        bytes.put(iso_cotp_data);
        is_last = cotp.is_last();
    }

    Ok(bytes)
}

pub(crate) async fn exchange_buffer(
    conn: &mut TcpStream,
    data: BytesMut,
) -> Result<BytesMut, Error> {
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
    let mut data = BytesMut::with_capacity(mem::size_of::<TTPKTHeader>());
    conn.read_buf(&mut data).await?;
    TTPKTHeader::try_from(&mut data)
}

async fn read_tpkt_data(conn: &mut TcpStream, length: u16) -> Result<BytesMut, Error> {
    let mut data = BytesMut::with_capacity(length as usize - mem::size_of::<TTPKTHeader>());

    match conn.read_buf(&mut data).await {
        Ok(_) => Ok(data),
        Err(_) => Err(Error::ISOResponse(IsoError::InvalidDataSize)),
    }
}
