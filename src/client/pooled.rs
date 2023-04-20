use std::net::Ipv4Addr;

use crate::{errors::Error, S7Client, S7Types};
use deadpool::{
    async_trait,
    managed::{self, BuildError},
};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy)]
struct S7Info {
    pdu_length: u16,
    max_amq_caller: u16,
    max_amq_calle: u16,
}

impl From<&S7Client> for S7Info {
    fn from(value: &S7Client) -> Self {
        Self {
            pdu_length: value.pdu_length,
            max_amq_calle: value.max_amq_calle,
            max_amq_caller: value.max_amq_caller,
        }
    }
}

pub(crate) struct S7PoolManager {
    s7_ip: Ipv4Addr,
    s7_type: S7Types,
    s7_info: RwLock<Option<S7Info>>,
}

#[async_trait]
impl managed::Manager for S7PoolManager {
    type Type = S7Client;
    type Error = Error;

    async fn create(&self) -> Result<S7Client, Error> {
        let mut client = S7Client::new(self.s7_ip, self.s7_type).await?;
        let s7_info_reader = self.s7_info.read().await;
        if let Some(info) = *s7_info_reader {
            client.pdu_length = info.pdu_length;
            client.max_amq_calle = info.max_amq_calle;
            client.max_amq_caller = info.max_amq_caller;
        } else {
            drop(s7_info_reader);
            client.connect().await?;
            let mut s7_info_writer = self.s7_info.write().await;
            *s7_info_writer = Some(S7Info::from(&client));
        }

        Ok(client)
    }

    async fn recycle(&self, client: &mut S7Client) -> managed::RecycleResult<Error> {
        if client.pdu_length == 0 {
            let mut s7_info_writer = self.s7_info.write().await;
            *s7_info_writer = None;
            Err(managed::RecycleError::StaticMessage("Connection closed"))
        } else {
            Ok(())
        }
    }
}

type S7PooledConnection = managed::Pool<S7PoolManager>;

/// Pooled connection to a PLC device from the S7 family
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct S7Pool(pub(crate) S7PooledConnection);

impl S7Pool {
    /// Create new pooled connection to an S7 PLC
    ///```rust, ignore
    /// use std::net::Ipv4Addr;
    /// use s7client::{S7Pool, S7Types};
    ///
    /// // create S7 pool
    /// let mut pool = S7Pool::new(Ipv4Addr::new(127, 0, 0, 1), S7Types::S71200)
    ///     .expect("Could not create pool");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if the `Pool` could not be created.
    pub fn new(ip: Ipv4Addr, s7_type: S7Types) -> Result<Self, BuildError<Error>> {
        let mgr = S7PoolManager {
            s7_ip: ip,
            s7_type,
            s7_info: RwLock::new(None),
        };
        let pool = S7PooledConnection::builder(mgr).max_size(3).build()?;

        Ok(S7Pool(pool))
    }
}
