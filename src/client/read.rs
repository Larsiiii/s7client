use super::create::S7Client;
use super::S7ReadAccess;
use crate::S7Pool;
use crate::{
    errors::Error,
    s7_protocol::{
        read_area::{read_area_multi, read_area_single},
        types::Area,
    },
};

/// *Methods for reading from the PLC device*
impl S7Client {
    /// Read a defined number bytes from a specified data block with an offset
    ///
    /// # Example
    /// ```rust, ignore
    /// let (data_block, offset, length) = (100, 0, 4);
    /// let data = client.db_read(data_block, offset, length)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
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
        self.validate_connection_info().await;
        read_area_single(
            &mut self.connection,
            self.pdu_length,
            &mut self.pdu_number,
            Area::DataBlock,
            S7ReadAccess::Bytes {
                db_number,
                start,
                length,
            },
        )
        .await
    }

    /// Read a specific bit from a specified data block
    ///
    /// The bit number must be within the range 0..7
    /// # Example
    /// ```rust, ignore
    /// let (data_block, byte, bit) = (100, 0, 0);
    /// let bit = client.db_read_bit(data_block, byte, bit)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn db_read_bit(&mut self, db_number: u16, byte: u32, bit: u8) -> Result<bool, Error> {
        self.validate_connection_info().await;
        if bit > 7 {
            Err(Error::RequestedBitOutOfRange)
        } else {
            Ok(read_area_single(
                &mut self.connection,
                self.pdu_length,
                &mut self.pdu_number,
                Area::DataBlock,
                S7ReadAccess::Bit {
                    db_number,
                    byte,
                    bit,
                },
            )
            .await?[0]
                > 0)
        }
    }

    pub async fn db_read_multi(
        &mut self,
        info: Vec<S7ReadAccess>,
    ) -> Result<Vec<Result<Vec<u8>, Error>>, Error> {
        self.validate_connection_info().await;

        read_area_multi(
            &mut self.connection,
            self.pdu_length,
            &mut self.pdu_number,
            Area::DataBlock,
            info,
        )
        .await
    }

    /// Read a defined number of bytes from the 'Merker area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust, ignore
    /// let (offset, length) = (0, 10);
    /// let bit = client.mb_read(offset, length)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn mb_read(&mut self, start: u32, length: u16) -> Result<Vec<u8>, Error> {
        self.validate_connection_info().await;
        read_area_single(
            &mut self.connection,
            self.pdu_length,
            &mut self.pdu_number,
            Area::Merker,
            S7ReadAccess::Bytes {
                db_number: 0,
                start,
                length,
            },
        )
        .await
    }

    /// Read a defined number of bytes from the 'input value area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust, ignore
    /// let (offset, length) = (0, 10);
    /// let bit = client.i_read(offset, length)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn i_read(&mut self, start: u32, length: u16) -> Result<Vec<u8>, Error> {
        self.validate_connection_info().await;
        read_area_single(
            &mut self.connection,
            self.pdu_length,
            &mut self.pdu_number,
            Area::ProcessInput,
            S7ReadAccess::Bytes {
                db_number: 0,
                start,
                length,
            },
        )
        .await
    }

    /// Read a defined number of bytes from the 'output value area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust, ignore
    /// let (offset, length) = (0, 10);
    /// let bit = client.o_read(offset, length)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn o_read(&mut self, start: u32, length: u16) -> Result<Vec<u8>, Error> {
        self.validate_connection_info().await;
        read_area_single(
            &mut self.connection,
            self.pdu_length,
            &mut self.pdu_number,
            Area::ProcessOutput,
            S7ReadAccess::Bytes {
                db_number: 0,
                start,
                length,
            },
        )
        .await
    }
}

/// # Methods for reading from the PLC device
impl S7Pool {
    /// Read a defined number bytes from a specified data block with an offset
    ///
    /// # Example
    /// ```rust, ignore
    /// let (data_block, offset, length) = (100, 0, 4);
    /// let data = client.db_read(data_block, offset, length)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
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
    /// ```rust, ignore
    /// let (data_block, byte, bit) = (100, 0, 0);
    /// let bit = client.db_read_bit(data_block, byte, bit)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn db_read_bit(&self, db_number: u16, byte: u32, bit: u8) -> Result<bool, Error> {
        let mut connection = self.0.get().await?;

        connection.db_read_bit(db_number, byte, bit).await
    }

    pub async fn db_read_multi(
        &self,
        info: Vec<S7ReadAccess>,
    ) -> Result<Vec<Result<Vec<u8>, Error>>, Error> {
        let mut connection = self.0.get().await?;

        connection.db_read_multi(info).await
    }

    /// Read a defined number of bytes from the 'Merker area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust, ignore
    /// let (offset, length) = (0, 10);
    /// let bit = client.mb_read(offset, length)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
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
    /// ```rust, ignore
    /// let (offset, length) = (0, 10);
    /// let bit = client.i_read(offset, length)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
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
    /// ```rust, ignore
    /// let (offset, length) = (0, 10);
    /// let bit = client.o_read(offset, length)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during reading.
    pub async fn o_read(&self, start: u32, length: u16) -> Result<Vec<u8>, Error> {
        let mut connection = self.0.get().await?;

        connection.o_read(start, length).await
    }
}
