#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![deny(
    missing_debug_implementations,
    rust_2018_idioms,
    single_use_lifetimes,
    unreachable_pub
)]

/*!
This crate provides a library for reading and writing data to and from PLC devices of the Siemens S7 family.


Until now the crate is not tested on actual hardware but only on an implementation of the [Snap7](http://snap7.sourceforge.net) server that mocks an S7 PLC.

# Usage

This crate is on [github.com](https://github.com/Larsiiii/s7client) and can be
used by adding `s7client` to your dependencies in your project's `Cargo.toml`.

```toml
[dependencies]
s7client = { git = "https://github.com/Larsiiii/s7client" }
```

# Examples
A connection with a PLC can either be opened via a standalone connection ([`S7Client`](crate::client::create::S7Client)) or with a connection pool ([`S7Pool`](crate::client::pooled::S7Pool)).
## Connection via standalone connection
```rust
# tokio_test::block_on(async {
use std::net::Ipv4Addr;
use s7client::{S7Client, S7Types};

// create single s7 client
let mut client = S7Client::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)
    .await?;

// read some data
let data = client.db_read(100, 0, 4).await?;

# Ok::<(), s7client::errors::Error>(())
# });
```

## Connection via a pooled connection
```rust
# tokio_test::block_on(async {
use std::net::Ipv4Addr;
use s7client::{S7Pool, S7Types};

// create connection pool
let mut pool = S7Pool::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;

// read some data
let data = pool.db_read(100, 0, 4).await?;
# Ok::<(), s7client::errors::Error>(())
# });
```
*/

mod client;
mod connection;
pub mod errors;
mod s7_protocol;

pub use client::create::S7Client;
pub use client::{triggers::TriggerCollection, S7ReadAccess, S7WriteAccess};
pub use connection::iso::S7Types;

pub use client::pooled::S7Pool;
