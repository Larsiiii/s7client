use super::create::S7Client;
use crate::s7_protocol::types::Area;
use crate::s7_protocol::write_area::write_area_multi;
use crate::{errors::Error, s7_protocol::write_area::write_area_single};
use crate::{S7Pool, S7WriteAccess};

/// *Methods for writing data into the PLC device*
impl S7Client {
    /// Write a defined number bytes into a specified data block with an offset
    ///
    /// # Example
    /// ```rust, ignore
    /// let (data_block, offset, data) = (100, 0, vec![0, 1, 2, 3]);
    /// let data = client.db_write(data_block, offset, &data)
    ///     .await
    ///     .expect("Could not write to S7 PLC");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn db_write(
        &mut self,
        db_number: u16,
        start: u32,
        data: &Vec<u8>,
    ) -> Result<(), Error> {
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
    /// ```rust, ignore
    /// let (data_block, byte, bit, value) = (100, 0, 0, false);
    /// let bit = client.db_write_bit(data_block, byte, bit, value)
    ///     .await
    ///     .expect("Could not write to S7 PLC");
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
        if bit > 7 {
            Err(Error::RequestedBitOutOfRange)
        } else {
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
    }

    pub async fn db_write_multi(
        &mut self,
        info: Vec<S7WriteAccess<'_>>,
    ) -> Result<Vec<Result<(), Error>>, Error> {
        self.validate_connection_info().await;

        write_area_multi(self, Area::DataBlock, info).await
    }

    /// Write a defined number of bytes to the 'Merker area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust, ignore
    /// let (offset, length, data) = (0, 10, vec![0, 1]);
    /// let bit = client.mb_write(offset, length, &data)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn mb_write(&mut self, start: u32, data: &Vec<u8>) -> Result<(), Error> {
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
    /// ```rust, ignore
    /// let (offset, length, data) = (0, 10, vec![0, 1]);
    /// let bit = client.i_write(offset, length, &data)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn i_write(&mut self, start: u32, data: &Vec<u8>) -> Result<(), Error> {
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
    /// ```rust, ignore
    /// let (offset, length, data) = (0, 10, vec![0, 1]);
    /// let bit = client.o_write(offset, length, &data)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn o_write(&mut self, start: u32, data: &Vec<u8>) -> Result<(), Error> {
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
    /// ```rust, ignore
    /// let (data_block, offset, data) = (100, 0, vec![0, 1, 2, 3]);
    /// let data = client.db_read(data_block, offset, &data)
    ///     .await
    ///     .expect("Could not write to S7 PLC");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn db_write(&self, db_number: u16, start: u32, data: &Vec<u8>) -> Result<(), Error> {
        let mut connection = self.0.get().await?;
        connection.db_write(db_number, start, data).await
    }

    /// Write a specific bit to a specified data block
    ///
    /// The bit number must be within the range 0..7
    /// # Example
    /// ```rust, ignore
    /// let (data_block, byte, bit, value) = (100, 0, 0, false);
    /// let bit = client.db_read_bit(data_block, byte, bit, value)
    ///     .await
    ///     .expect("Could not write to S7 PLC");
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

    pub async fn db_write_multi(
        &self,
        info: Vec<S7WriteAccess<'_>>,
    ) -> Result<Vec<Result<(), Error>>, Error> {
        let mut connection = self.0.get().await?;
        connection.db_write_multi(info).await
    }

    /// Write a defined number of bytes to the 'Merker area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust, ignore
    /// let (offset, length, data) = (0, 10, vec![0, 1]);
    /// let bit = client.mb_write(offset, length, &data)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn mb_write(&self, start: u32, data: &Vec<u8>) -> Result<(), Error> {
        let mut connection = self.0.get().await?;
        connection.mb_write(start, data).await
    }

    /// Write a defined number of bytes into the 'input value area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust, ignore
    /// let (offset, length, data) = (0, 10, vec![0, 1]);
    /// let bit = client.i_write(offset, length, &data)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn i_write(&self, start: u32, data: &Vec<u8>) -> Result<(), Error> {
        let mut connection = self.0.get().await?;
        connection.i_write(start, data).await
    }

    /// Write a defined number of bytes into the 'output value area' of the PLC with a certain offset
    ///
    /// # Example
    /// ```rust, ignore
    /// let (offset, length, data) = (0, 10, vec![0, 1]);
    /// let bit = client.o_write(offset, length, &data)
    ///     .await
    ///     .expect("Could not read from S7 PLC");
    /// ```
    /// # Errors
    ///
    /// Will return `Error` if any errors occurred during writing.
    pub async fn o_write(&self, start: u32, data: &Vec<u8>) -> Result<(), Error> {
        let mut connection = self.0.get().await?;
        connection.o_write(start, data).await
    }
}
