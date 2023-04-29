use super::create::S7Client;
use super::verify_max_bit;
use crate::s7_protocol::types::Area;
use crate::s7_protocol::write_area::write_area_multi;
use crate::{errors::Error, s7_protocol::write_area::write_area_single};
use crate::{S7Pool, S7WriteAccess};

/// *Methods for writing data into the PLC device*
impl S7Client {
    /// Write a defined number bytes into a specified data block with an offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Client, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut client = S7Client::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200).await?;
    /// let (data_block, offset, data) = (100, 0, &[0, 1, 2, 3]);
    /// let data = client.db_write(data_block, offset, data)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn db_write(&mut self, db_number: u16, start: u32, data: &[u8]) -> Result<(), Error> {
        self.validate_connection_info().await?;
        write_area_single(
            self,
            Area::DataBlock,
            S7WriteAccess::Bytes {
                db_number,
                start,
                data,
            },
        )
        .await
    }

    /// Write a specific bit to a specified data block
    ///
    /// The bit number must be within the range 0..7
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Client, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut client = S7Client::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200).await?;
    /// let (data_block, byte, bit, value) = (100, 0, 0, false);
    /// let bit = client.db_write_bit(data_block, byte, bit, value)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn db_write_bit(
        &mut self,
        db_number: u16,
        byte: u32,
        bit: u8,
        value: bool,
    ) -> Result<(), Error> {
        self.validate_connection_info().await?;

        verify_max_bit(bit)?;

        write_area_single(
            self,
            Area::DataBlock,
            S7WriteAccess::Bit {
                db_number,
                byte,
                bit,
                value,
            },
        )
        .await
    }

    /// Write multiple bytes or bits to different locations of the PLC
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Client, S7Types, S7WriteAccess};
    /// # tokio_test::block_on(async {
    /// # let mut client = S7Client::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200).await?;
    /// let data = client.db_write_multi(&[
    ///        S7WriteAccess::bytes(100, 0, &[0, 0, 0, 1]),
    ///        S7WriteAccess::bit(101, 0, 1, true),
    ///    ])
    ///    .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn db_write_multi(
        &mut self,
        info: &[S7WriteAccess<'_>],
    ) -> Result<Vec<Result<(), Error>>, Error> {
        self.validate_connection_info().await?;

        for access in info {
            verify_max_bit(access.max_bit())?;
        }

        write_area_multi(self, Area::DataBlock, info).await
    }

    /// Write a defined number of bytes to the 'Merker area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Client, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut client = S7Client::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200).await?;
    /// let (start, data) = (10, &[0, 1]);
    /// let bit = client.mb_write(start, data)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn mb_write(&mut self, start: u32, data: &[u8]) -> Result<(), Error> {
        self.validate_connection_info().await?;
        write_area_single(
            self,
            Area::Merker,
            S7WriteAccess::Bytes {
                db_number: 0,
                start,
                data,
            },
        )
        .await
    }

    /// Write a defined number of bytes into the 'input value area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Client, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut client = S7Client::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200).await?;
    /// let (start, data) = (10, &[0, 1]);
    /// let bit = client.i_write(start, data)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn i_write(&mut self, start: u32, data: &[u8]) -> Result<(), Error> {
        self.validate_connection_info().await?;
        write_area_single(
            self,
            Area::ProcessInput,
            S7WriteAccess::Bytes {
                db_number: 0,
                start,
                data,
            },
        )
        .await
    }

    /// Write a defined number of bytes into the 'output value area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Client, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut client = S7Client::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200).await?;
    /// let (start, data) = (10, &[0, 1]);
    /// let bit = client.o_write(start, data)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn o_write(&mut self, start: u32, data: &[u8]) -> Result<(), Error> {
        self.validate_connection_info().await?;
        write_area_single(
            self,
            Area::ProcessOutput,
            S7WriteAccess::Bytes {
                db_number: 0,
                start,
                data,
            },
        )
        .await
    }
}

/// *Methods for writing data into the PLC device*
impl S7Pool {
    /// Write a defined number bytes into a specified data block with an offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Pool, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut pool = S7Pool::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;
    /// let (data_block, offset, data) = (100, 0, &[0, 1, 2, 3]);
    /// let data = pool.db_write(data_block, offset, data)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn db_write(&self, db_number: u16, start: u32, data: &[u8]) -> Result<(), Error> {
        let mut connection = self.0.get().await?;
        connection.db_write(db_number, start, data).await
    }

    /// Write a specific bit to a specified data block
    ///
    /// The bit number must be within the range 0..7
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Pool, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut pool = S7Pool::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;
    /// let (data_block, byte, bit, value) = (100, 0, 0, false);
    /// let bit = pool.db_write_bit(data_block, byte, bit, value)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn db_write_bit(
        &self,
        db_number: u16,
        byte: u32,
        bit: u8,
        value: bool,
    ) -> Result<(), Error> {
        let mut connection = self.0.get().await?;
        connection.db_write_bit(db_number, byte, bit, value).await
    }

    /// Write multiple bytes or bits to different locations of the PLC
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Pool, S7Types, S7WriteAccess};
    /// # tokio_test::block_on(async {
    /// # let mut pool = S7Pool::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;
    /// let data = pool.db_write_multi(&[
    ///        S7WriteAccess::bytes(100, 0, &[0, 0, 0, 1]),
    ///        S7WriteAccess::bit(101, 0, 1, true),
    ///    ])
    ///    .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn db_write_multi(
        &self,
        info: &[S7WriteAccess<'_>],
    ) -> Result<Vec<Result<(), Error>>, Error> {
        let mut connection = self.0.get().await?;
        connection.db_write_multi(info).await
    }

    /// Write a defined number of bytes to the 'Merker area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Pool, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut pool = S7Pool::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;
    /// let (start, data) = (10, &[0, 1]);
    /// let bit = pool.mb_write(start, data)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn mb_write(&self, start: u32, data: &[u8]) -> Result<(), Error> {
        let mut connection = self.0.get().await?;
        connection.mb_write(start, data).await
    }

    /// Write a defined number of bytes into the 'input value area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Pool, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut pool = S7Pool::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;
    /// let (start, data) = (10, &[0, 1]);
    /// let bit = pool.i_write(start, data)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn i_write(&self, start: u32, data: &[u8]) -> Result<(), Error> {
        let mut connection = self.0.get().await?;
        connection.i_write(start, data).await
    }

    /// Write a defined number of bytes into the 'output value area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Pool, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut pool = S7Pool::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;
    /// let (start, data) = (10, &[0, 1]);
    /// let bit = pool.o_write(start, data)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn o_write(&self, start: u32, data: &[u8]) -> Result<(), Error> {
        let mut connection = self.0.get().await?;
        connection.o_write(start, data).await
    }
}
