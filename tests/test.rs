use s7client::{S7Client, S7Pool, S7ReadAccess, S7Types, S7WriteAccess};
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
        .expect("Could not read from S7 PLC");
    assert_eq!(data.len(), 4);

    let data2 = client
        .db_read(0, 0, 1)
        .await
        .expect("Could not read from S7 PLC");
    assert_eq!(data2.len(), 1);

    // create S7 connection pool
    let pool = S7Pool::new(std::net::Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)
        .expect("Could not create Pool");
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
    let test_pool = S7Pool::new(std::net::Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)
        .expect("Could not create pool");
    let pool = test_pool.clone();

    // write data
    let test_value: u32 = 32;
    let test_data = test_value.to_be_bytes();
    pool.db_write(TEST_DB, 40, &test_data.to_vec())
        .await
        .expect("Could not write to S7");

    // read data
    let read_data = pool
        .db_read(TEST_DB, 40, test_data.len() as u16)
        .await
        .expect("Could not read data from S7");

    assert_eq!(read_data, test_data);

    // second test run with different data to ensure data is not preserved from last test run
    // write data
    let test_value: u32 = 18942;
    let test_data = test_value.to_be_bytes();
    pool.db_write(TEST_DB, 40, &test_data.to_vec())
        .await
        .expect("Could not write to S7");

    // read data
    let read_data = pool
        .db_read(TEST_DB, 40, test_data.len() as u16)
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

#[tokio::test]
async fn test_multi() {
    // create single s7 client object
    let client = S7Pool::new(std::net::Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)
        .expect("Could not create pool");

    let res = client
        .db_read_multi(vec![
            S7ReadAccess::Bytes {
                db_number: 0,
                start: 0,
                length: 300,
            },
            S7ReadAccess::Bit {
                db_number: TEST_DB,
                byte: 0,
                bit: 1,
            },
        ])
        .await
        .expect("Could not read multiple items from PLC");

    let result1 = res[0]
        .as_ref()
        .expect("Could not get first result from PLC message");
    assert_eq!(result1.len(), 300);

    let result2 = res[1]
        .as_ref()
        .expect("Could not get first result from PLC message");
    assert_eq!(result2.len(), 1);

    let res = client
        .db_write_multi(vec![
            S7WriteAccess::Bytes {
                db_number: 0,
                start: 0,
                data: &10_u32.to_be_bytes().to_vec(),
            },
            S7WriteAccess::Bit {
                db_number: TEST_DB,
                byte: 0,
                bit: 1,
                value: true,
            },
        ])
        .await
        .expect("Could not write to PLC");

    let result1 = res[0]
        .as_ref()
        .expect("Could not get first result from PLC message");
    assert_eq!(result1, &());

    let result2 = res[1]
        .as_ref()
        .expect("Could not get first result from PLC message");
    assert_eq!(result2, &());
}

#[tokio::test]
async fn test_read_split() {
    // create single s7 client object
    let pool = S7Pool::new(std::net::Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)
        .expect("Could not create pool");

    // read data
    let read_data = pool
        .db_read(0, 40, 900)
        .await
        .expect("Could not read data from S7");

    assert_eq!(read_data.len(), 900);
}
