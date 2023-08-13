use std::hash::Hash;
use std::net::Ipv4Addr;

use crate::S7ReadAccess;
use crate::{errors::Error, S7Client, S7Types, TriggerCollection};
use deadpool::{
    async_trait,
    managed::{self, BuildError},
};
pub(crate) struct S7PoolManager {
    s7_ip: Ipv4Addr,
    s7_type: S7Types,
}

#[async_trait]
impl managed::Manager for S7PoolManager {
    type Type = S7Client;
    type Error = Error;

    async fn create(&self) -> Result<S7Client, Error> {
        let mut client = S7Client::new(self.s7_ip, self.s7_type).await?;
        client.connect().await?;

        Ok(client)
    }

    async fn recycle(&self, client: &mut S7Client) -> managed::RecycleResult<Error> {
        if client.is_closed() {
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
    ///```rust
    /// use std::net::Ipv4Addr;
    /// use s7client::{S7Pool, S7Types};
    ///
    /// // create S7 pool
    /// let mut pool = S7Pool::new(Ipv4Addr::new(127, 0, 0, 1), S7Types::S71200)?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if the `Pool` could not be created.
    pub fn new(ip: Ipv4Addr, s7_type: S7Types) -> Result<Self, BuildError<Error>> {
        let mgr = S7PoolManager { s7_ip: ip, s7_type };
        let pool = S7PooledConnection::builder(mgr).max_size(3).build()?;

        Ok(S7Pool(pool))
    }

    /// Create new collection of observed `Bool` variables of S7 PLC
    ///```rust
    /// use std::net::Ipv4Addr;
    /// use s7client::{S7Pool, S7Types, S7ReadAccess};
    ///
    /// // create S7 pool
    /// let mut pool = S7Pool::new(Ipv4Addr::new(127, 0, 0, 1), S7Types::S71200)?;
    /// // create trigger collection
    /// let  trigger_collection = pool.new_trigger_collection(&[
    ///         ("TRIGGER_ONE", S7ReadAccess::bit(100, 0, 1)),
    ///         ("TRIGGER_TWO", S7ReadAccess::bit(100, 0, 2)),
    ///     ])?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if the `TriggerCollection` could not be created.
    pub fn new_trigger_collection<T>(
        &self,
        triggers: &[(T, S7ReadAccess)],
    ) -> Result<TriggerCollection<T>, Error>
    where
        T: Hash + Eq + Clone,
    {
        TriggerCollection::new(self, triggers)
    }
}
