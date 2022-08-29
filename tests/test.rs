use s7client::{S7Client, S7Pool, S7Types};
use tokio::join;

const TEST_DB: u16 = 1;

#[tokio::test]
async fn create_connections() {
    // create single s7 client object
    let mut client = S7Client::new(std::net::Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)
        .await
        .expect("Could not create S7 Client");

    let data = client
        .db_read(0, 48, 4)
        .await
        .expect("Could not read from S7 client");
    assert_eq!(data.len(), 4);

    let data2 = client
        .db_read(0, 0, 1)
        .await
        .expect("Could not read from S7 client");
    assert_eq!(data2.len(), 1);

    // create S7 connection pool
    let pool = S7Pool::new(std::net::Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200);
    let one = pool.db_read(0, 0, 1);
    let two = pool.db_read(0, 0, 1);
    let three = pool.db_read(0, 0, 1);
    let four = pool.db_read(0, 0, 1);

    let (r1, r2, r3, r4) = join!(one, two, three, four);
    assert!(r1.is_ok() && r2.is_ok() && r3.is_ok() && r4.is_ok());
    assert_eq!(r1.unwrap().len(), 1);
    assert_eq!(r2.unwrap().len(), 1);
    assert_eq!(r3.unwrap().len(), 1);
    assert_eq!(r4.unwrap().len(), 1);
}

#[tokio::test]
async fn test_data_exchange() {
    // create S7 connection pool
    let pool = S7Pool::new(std::net::Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200);

    // write data
    let test_value: u32 = 32;
    let test_data = test_value.to_be_bytes();
    pool.db_write(TEST_DB, 508, &test_data.to_vec())
        .await
        .expect("Could not write to S7");

    // read data
    let read_data = pool
        .db_read(TEST_DB, 508, test_data.len() as u32)
        .await
        .expect("Could not read data from S7");

    assert_eq!(read_data, test_data);

    // second test run with different data to ensure data is not preserved from last test run
    // write data
    let test_value: u32 = 18942;
    let test_data = test_value.to_be_bytes();
    pool.db_write(TEST_DB, 508, &test_data.to_vec())
        .await
        .expect("Could not write to S7");

    // read data
    let read_data = pool
        .db_read(TEST_DB, 508, test_data.len() as u32)
        .await
        .expect("Could not read data from S7");

    assert_eq!(read_data, test_data);
}

#[tokio::test]
async fn test_bit_exchange() {
    let test_byte = 0;
    let test_bit = 1;

    // create single s7 client object
    let mut client = S7Client::new(std::net::Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)
        .await
        .expect("Could not create S7 Client");

    // write bit to true
    client
        .db_write_bit(TEST_DB, test_byte, test_bit, true)
        .await
        .expect("Could not write bit");

    assert_eq!(
        client
            .db_read_bit(TEST_DB, test_byte, test_bit)
            .await
            .expect("Could not read bit"),
        true
    );

    // write bit to false
    client
        .db_write_bit(TEST_DB, test_byte, test_bit, false)
        .await
        .expect("Could not write bit");

    assert_eq!(
        client
            .db_read_bit(TEST_DB, test_byte, test_bit)
            .await
            .expect("Could not read bit"),
        false
    );
}
