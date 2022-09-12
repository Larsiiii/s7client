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
}

impl S7Client {
    /// Create new standalone connection to an S7 PLC
    ///```rust, ignore
    /// use std::net::Ipv4Addr;
    /// use s7client::{S7Client, S7Types};
    ///
    /// // create single s7 client
    /// let mut client = S7Client::new(Ipv4Addr::new(127, 0, 0, 1), S7Types::S71200)
    ///    .await
    ///    .expect("Could not create S7 Client");
    /// ```
    pub async fn new(ip: Ipv4Addr, s7_type: S7Types) -> Result<Self, Error> {
        let tcp_client = match timeout(
            CONNECTION_TIMEOUT,
            TcpStream::connect(format!("{}:{}", ip, TCP_PORT)),
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

        Ok(Self {
            connection: tcp_client,
            s7_type,
            pdu_length: 0,
            pdu_number: 0,
            max_amq_caller: 0,
            max_amq_calle: 0,
        })
    }

    /// Manually trigger negotiation of connection parameters
    ///
    /// This is not necessary as the parameters get checked before a request is send to the PLC
    pub async fn connect(&mut self) -> Result<(), Error> {
        let connection_parameters = connect(&mut self.connection, self.s7_type).await?;

        self.pdu_length = connection_parameters.pdu_length;
        self.max_amq_caller = connection_parameters.max_amq_caller;
        self.max_amq_calle = connection_parameters.max_amq_calle;

        Ok(())
    }

    /// Gracefully disconnect from the PLC
    pub async fn disconnect(&mut self) -> Result<(), Error> {
        disconnect(&mut self.connection).await?;
        self.reset_connection_info();
        Ok(())
    }

    pub(crate) async fn validate_connection_info(&mut self) -> bool {
        if self.pdu_length == 0 {
            let _connection = self.connect().await;
        }
        self.pdu_length > 0
    }

    pub(crate) fn reset_connection_info(&mut self) {
        self.pdu_length = 0;
        self.max_amq_calle = 0;
        self.max_amq_caller = 0;
    }
}

// impl Drop for S7Client {
//     fn drop(&mut self) {
//         // TODO implement drop for async
//         let _ = self.disconnect();
//     }
// }
