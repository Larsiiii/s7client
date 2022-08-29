use super::create::S7Client;
use crate::s7_protocol::types::{Area, S7DataTypes};
use crate::S7Pool;
use crate::{errors::Error, s7_protocol::read_area::read_area};

/// *Methods for reading from the PLC device*
impl S7Client {
    /// Read a defined number bytes from a specified data block with an offset
    ///
    /// # Example
    /// ```rust, ignore
    /// let (data_block, offset, length) = (100, 0, 4);
    /// let data = client.db_read(data_block, offset, length)
    ///     .await
    ///     .expect("Could not read from S7 client");
    /// ```
    pub async fn db_read(
        &mut self,
        db_number: u16,
        start: u32,
        length: u32,
    ) -> Result<Vec<u8>, Error> {
        self.validate_connection_info().await;
        read_area(
            &mut self.connection,
            self.pdu_length,
            &mut self.pdu_number,
            Area::DataBlock,
            db_number,
            start,
            length,
            S7DataTypes::S7BYTE,
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
    ///     .expect("Could not read from S7 client");
    /// ```
    pub async fn db_read_bit(&mut self, db_number: u16, byte: u32, bit: u8) -> Result<bool, Error> {
        self.validate_connection_info().await;
        if bit > 7 {
            Err(Error::RequestedBitOutOfRange)
        } else {
            let start = byte * 8 + bit as u32;
            Ok(read_area(
                &mut self.connection,
                self.pdu_length,
                &mut self.pdu_number,
                Area::DataBlock,
                db_number,
                start,
                1,
                S7DataTypes::S7BIT,
            )
            .await?[0]
                > 0)
        }
    }

    /// Read a defined number of bytes from the 'Merker area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust, ignore
    /// let (offset, length) = (0, 10);
    /// let bit = client.mb_read(offset, length)
    ///     .await
    ///     .expect("Could not read from S7 client");
    /// ```
    pub async fn mb_read(&mut self, start: u32, length: u32) -> Result<Vec<u8>, Error> {
        self.validate_connection_info().await;
        read_area(
            &mut self.connection,
            self.pdu_length,
            &mut self.pdu_number,
            Area::Merker,
            0,
            start,
            length,
            S7DataTypes::S7BYTE,
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
    ///     .expect("Could not read from S7 client");
    /// ```
    pub async fn i_read(&mut self, start: u32, length: u32) -> Result<Vec<u8>, Error> {
        self.validate_connection_info().await;
        read_area(
            &mut self.connection,
            self.pdu_length,
            &mut self.pdu_number,
            Area::ProcessInput,
            0,
            start,
            length,
            S7DataTypes::S7BYTE,
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
    ///     .expect("Could not read from S7 client");
    /// ```
    pub async fn o_read(&mut self, start: u32, length: u32) -> Result<Vec<u8>, Error> {
        self.validate_connection_info().await;
        read_area(
            &mut self.connection,
            self.pdu_length,
            &mut self.pdu_number,
            Area::ProcessOutput,
            0,
            start,
            length,
            S7DataTypes::S7BYTE,
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
    ///     .expect("Could not read from S7 client");
    /// ```
    pub async fn db_read(&self, db_number: u16, start: u32, length: u32) -> Result<Vec<u8>, Error> {
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
    ///     .expect("Could not read from S7 client");
    /// ```
    pub async fn db_read_bit(&self, db_number: u16, byte: u32, bit: u8) -> Result<bool, Error> {
        let mut connection = self.0.get().await?;

        connection.db_read_bit(db_number, byte, bit).await
    }

    /// Read a defined number of bytes from the 'Merker area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust, ignore
    /// let (offset, length) = (0, 10);
    /// let bit = client.mb_read(offset, length)
    ///     .await
    ///     .expect("Could not read from S7 client");
    /// ```
    pub async fn mb_read(&self, start: u32, length: u32) -> Result<Vec<u8>, Error> {
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
    ///     .expect("Could not read from S7 client");
    /// ```
    pub async fn i_read(&self, start: u32, length: u32) -> Result<Vec<u8>, Error> {
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
    ///     .expect("Could not read from S7 client");
    /// ```
    pub async fn o_read(&self, start: u32, length: u32) -> Result<Vec<u8>, Error> {
        let mut connection = self.0.get().await?;

        connection.o_read(start, length).await
    }
}
