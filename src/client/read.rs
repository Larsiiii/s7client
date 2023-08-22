use super::create::S7Client;
use super::{verify_max_bit, S7ReadAccess};
use crate::S7Pool;
use crate::{
    errors::Error,
    s7_protocol::{
        read_area::{read_area_multi, read_area_single},
        types::Area,
    },
};

/// * Methods for reading from the PLC device*
impl S7Client {
    /// Read a defined number bytes from a specified data block with an offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Client, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut client = S7Client::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200).await?;
    /// let (data_block, offset, length) = (100, 0, 4);
    /// let data = client.db_read(data_block, offset, length)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn db_read(
        &mut self,
        db_number: u16,
        start: u32,
        length: u16,
    ) -> Result<Vec<u8>, Error> {
        self.validate_connection_info()?;
        match read_area_single(
            self,
            Area::DataBlock,
            S7ReadAccess::Bytes {
                db_number,
                start,
                length,
            },
        )
        .await
        {
            Ok(result) => Ok(result),
            Err(error) => {
                if error.is_connection_error() {
                    self.set_closed();
                }
                Err(error)
            }
        }
    }

    /// Read a specific bit from a specified data block
    ///
    /// The bit number must be within the range 0..7
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Client, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut client = S7Client::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200).await?;
    /// let (data_block, byte, bit) = (100, 0, 0);
    /// let bit = client.db_read_bit(data_block, byte, bit)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn db_read_bit(&mut self, db_number: u16, byte: u32, bit: u8) -> Result<bool, Error> {
        self.validate_connection_info()?;

        verify_max_bit(bit)?;

        match read_area_single(
            self,
            Area::DataBlock,
            S7ReadAccess::Bit {
                db_number,
                byte,
                bit,
            },
        )
        .await
        {
            Ok(result) => Ok(result[0] > 0),
            Err(error) => {
                if error.is_connection_error() {
                    self.set_closed();
                }
                Err(error)
            }
        }
    }

    /// Read multiple bytes or bits from different locations of the PLC
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Client, S7Types, S7ReadAccess};
    /// # tokio_test::block_on(async {
    /// # let mut client = S7Client::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200).await?;
    /// let data = client.db_read_multi(&[
    ///        S7ReadAccess::bytes(100, 0, 300),
    ///        S7ReadAccess::bit(101, 0, 1),
    ///    ])
    ///    .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn db_read_multi(
        &mut self,
        info: &[S7ReadAccess],
    ) -> Result<Vec<Result<Vec<u8>, Error>>, Error> {
        self.validate_connection_info()?;

        for access in info {
            verify_max_bit(access.max_bit())?;
        }

        match read_area_multi(self, Area::DataBlock, info).await {
            Ok(result) => Ok(result),
            Err(error) => {
                if error.is_connection_error() {
                    self.set_closed();
                }
                Err(error)
            }
        }
    }

    /// Read a defined number of bytes from the 'Merker area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Client, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut client = S7Client::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200).await?;
    /// let (offset, length) = (0, 10);
    /// let bit = client.mb_read(offset, length)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn mb_read(&mut self, start: u32, length: u16) -> Result<Vec<u8>, Error> {
        self.validate_connection_info()?;
        match read_area_single(
            self,
            Area::Merker,
            S7ReadAccess::Bytes {
                db_number: 0,
                start,
                length,
            },
        )
        .await
        {
            Ok(result) => Ok(result),
            Err(error) => {
                if error.is_connection_error() {
                    self.set_closed();
                }
                Err(error)
            }
        }
    }

    /// Read a defined number of bytes from the 'input value area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Client, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut client = S7Client::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200).await?;
    /// let (offset, length) = (0, 10);
    /// let bit = client.i_read(offset, length)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn i_read(&mut self, start: u32, length: u16) -> Result<Vec<u8>, Error> {
        self.validate_connection_info()?;
        match read_area_single(
            self,
            Area::ProcessInput,
            S7ReadAccess::Bytes {
                db_number: 0,
                start,
                length,
            },
        )
        .await
        {
            Ok(result) => Ok(result),
            Err(error) => {
                if error.is_connection_error() {
                    self.set_closed();
                }
                Err(error)
            }
        }
    }

    /// Read a defined number of bytes from the 'output value area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Client, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut client = S7Client::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200).await?;
    /// let (offset, length) = (0, 10);
    /// let bit = client.o_read(offset, length)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn o_read(&mut self, start: u32, length: u16) -> Result<Vec<u8>, Error> {
        self.validate_connection_info()?;
        match read_area_single(
            self,
            Area::ProcessOutput,
            S7ReadAccess::Bytes {
                db_number: 0,
                start,
                length,
            },
        )
        .await
        {
            Ok(result) => Ok(result),
            Err(error) => {
                if error.is_connection_error() {
                    self.set_closed();
                }
                Err(error)
            }
        }
    }
}

/// # Methods for reading from the PLC device
impl S7Pool {
    /// Read a defined number bytes from a specified data block with an offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Pool, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut pool = S7Pool::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;
    /// let (data_block, offset, length) = (100, 0, 4);
    /// let data = pool.db_read(data_block, offset, length)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn db_read(&self, db_number: u16, start: u32, length: u16) -> Result<Vec<u8>, Error> {
        let mut connection = self.0.get().await?;

        connection.db_read(db_number, start, length).await
    }

    /// Read a specific bit from a specified data block
    ///
    /// The bit number must be within the range 0..7
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Pool, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut pool = S7Pool::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;
    /// let (data_block, byte, bit) = (100, 0, 0);
    /// let bit = pool.db_read_bit(data_block, byte, bit)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn db_read_bit(&self, db_number: u16, byte: u32, bit: u8) -> Result<bool, Error> {
        let mut connection = self.0.get().await?;

        connection.db_read_bit(db_number, byte, bit).await
    }

    /// Read multiple bytes or bits from different locations of the PLC
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Pool, S7Types, S7ReadAccess};
    /// # tokio_test::block_on(async {
    /// # let mut pool = S7Pool::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;
    /// let data = pool.db_read_multi(&[
    ///        S7ReadAccess::bytes(100, 0, 300),
    ///        S7ReadAccess::bit(101, 0, 1),
    ///    ])
    ///    .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn db_read_multi(
        &self,
        info: &[S7ReadAccess],
    ) -> Result<Vec<Result<Vec<u8>, Error>>, Error> {
        let mut connection = self.0.get().await?;

        connection.db_read_multi(info).await
    }

    /// Read a defined number of bytes from the 'Merker area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Pool, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut pool = S7Pool::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;
    /// let (offset, length) = (0, 10);
    /// let bit = pool.mb_read(offset, length)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn mb_read(&self, start: u32, length: u16) -> Result<Vec<u8>, Error> {
        let mut connection = self.0.get().await?;

        connection.mb_read(start, length).await
    }

    /// Read a defined number of bytes from the 'input value area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Pool, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut pool = S7Pool::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;
    /// let (offset, length) = (0, 10);
    /// let bit = pool.i_read(offset, length)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn i_read(&self, start: u32, length: u16) -> Result<Vec<u8>, Error> {
        let mut connection = self.0.get().await?;

        connection.i_read(start, length).await
    }

    /// Read a defined number of bytes from the 'output value area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust
    /// # use std::net::Ipv4Addr;
    /// # use s7client::{S7Pool, S7Types};
    /// # tokio_test::block_on(async {
    /// # let mut pool = S7Pool::new(Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;
    /// let (offset, length) = (0, 10);
    /// let bit = pool.o_read(offset, length)
    ///     .await?;
    /// # Ok::<(), s7client::errors::Error>(())
    /// });
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn o_read(&self, start: u32, length: u16) -> Result<Vec<u8>, Error> {
        let mut connection = self.0.get().await?;

        connection.o_read(start, length).await
    }
}
