#![warn(missing_docs)]
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
A connection with a PLC can either be opened via a standalone connection ([S7Client](crate::client::create::S7Client)) or with a connection pool ([S7Pool](crate::client::pool::S7Pool)).
## Connection via standalone connection
```rust
use std::net::Ipv4Addr;
use s7client::{S7Client, S7Types};

// create single s7 client
let mut client = S7Client::new(Ipv4Addr::new(127, 0, 0, 1), S7Types::S71200)
    .await
    .expect("Could not create S7 Client");

// read some data
let data = client.db_read(100, 0, 4).await.expect("Could not read from S7 client");
```

## Connection via a pooled connection
```rust
use std::net::Ipv4Addr;
use s7client::{S7Pool, S7Types};

// create connection pool
let mut client = S7Pool::new(Ipv4Addr::new(127, 0, 0, 1), S7Types::S71200);

// read some data
let data = client.db_read(100, 0, 4).await.expect("Could not read from S7 client");
```
*/

mod client;
mod connection;
pub mod errors;
mod s7_protocol;

pub use client::create::S7Client;
pub use connection::iso::S7Types;

pub use client::pool::S7Pool;
