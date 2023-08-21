use std::{net::Ipv4Addr, time::Duration};
use tokio::{net::TcpStream, time::timeout};

use crate::connection::{
    iso::S7Types,
    tcp::{connect, disconnect},
};
use crate::errors::Error;

// Default TCP Port
pub(crate) const TCP_PORT: u32 = 102;
// Default TCP timeout
pub(crate) const CONNECTION_TIMEOUT: Duration = Duration::from_secs(3);

/// Standalone S7 connection
#[derive(Debug)]
pub struct S7Client {
    pub(crate) connection: TcpStream,
    s7_type: S7Types,
    pub(crate) pdu_length: u16,
    pub(crate) pdu_number: u16,
    // The Max AMQ parameters define how many unacknowledged requests a PLC (Callee) is able to accept from a client (Caller).
    pub(crate) max_amq_caller: u16,
    pub(crate) max_amq_calle: u16,
    closed: bool,
}

impl S7Client {
    /// Create new standalone connection to an S7 PLC
    ///```rust
    /// # tokio_test::block_on(async {
    /// use std::net::Ipv4Addr;
    /// use s7client::{S7Client, S7Types};
    ///
    /// // create single s7 client
    /// let mut client = S7Client::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)
    ///          .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// # });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if no connection could be established to the PLC.
    pub async fn new(ip: Ipv4Addr, s7_type: S7Types) -> Result<Self, Error> {
        let tcp_client = match timeout(
            CONNECTION_TIMEOUT,
            TcpStream::connect(format!("{ip}:{TCP_PORT}")),
        )
        .await
        {
            Ok(connection) => connection,
            Err(_err) => {
                return Err(Error::Connection(format!(
                    "Error on connecting to '{}:{}': Timed out after {} seconds",
                    ip,
                    TCP_PORT,
                    CONNECTION_TIMEOUT.as_secs()
                )))
            }
        }?;

        let mut client = Self {
            connection: tcp_client,
            s7_type,
            pdu_length: 0,
            pdu_number: 0,
            max_amq_caller: 0,
            max_amq_calle: 0,
            closed: true,
        };
        client.connect().await?;

        Ok(client)
    }

    /// Manually trigger negotiation of connection parameters
    ///
    /// This is not necessary as the parameters get checked before a request is send to the PLC
    /// # Errors
    ///
    /// Will return `Error` if no connection could be established to the PLC.
    pub async fn connect(&mut self) -> Result<(), Error> {
        let connection_parameters = connect(&mut self.connection, self.s7_type).await?;

        self.pdu_length = connection_parameters.pdu_length;
        self.max_amq_caller = connection_parameters.max_amq_caller;
        self.max_amq_calle = connection_parameters.max_amq_calle;

        self.closed = false;

        Ok(())
    }

    /// Gracefully disconnect from the PLC
    /// # Errors
    ///
    /// Will return `Error` if the connection to the PLC could not be closed gracefully.
    pub async fn disconnect(&mut self) -> Result<(), Error> {
        disconnect(&mut self.connection).await?;
        self.closed = true;
        Ok(())
    }

    pub(crate) async fn validate_connection_info(&mut self) -> Result<(), Error> {
        if self.closed {
            return Err(Error::Connection("Connection is closed".to_string()));
        }
        Ok(())
    }

    pub(crate) fn set_closed(&mut self) {
        self.closed = true;
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.closed
    }
}

// impl Drop for S7Client {
//     fn drop(&mut self) {
//         // TODO implement drop for async
//         let _ = self.disconnect();
//     }
// }
