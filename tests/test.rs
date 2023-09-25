use std::time::Duration;

use s7client::{errors::Error, S7Client, S7Pool, S7ReadAccess, S7Types, S7WriteAccess};
use tokio::join;

const TEST_DB: u16 = 1;

#[tokio::test]
async fn create_connections() {
    // create single s7 client object
    let mut client = S7Client::new(std::net::Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)
        .await
        .expect("Could not create S7 Client");

    let data = client
        .db_read(TEST_DB, 48, 4)
        .await
        .expect("Could not read from S7 PLC");
    assert_eq!(data.len(), 4);

    let data2 = client
        .db_read(TEST_DB, 0, 1)
        .await
        .expect("Could not read from S7 PLC");
    assert_eq!(data2.len(), 1);

    // create S7 connection pool
    let pool = S7Pool::new(std::net::Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)
        .expect("Could not create Pool");
    let one = pool.db_read(TEST_DB, 0, 1);
    let two = pool.db_read(TEST_DB, 0, 1);
    let three = pool.db_read(TEST_DB, 0, 1);
    let four = pool.db_read(TEST_DB, 0, 1);

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
    pool.db_write(TEST_DB, 40, &test_data)
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
    pool.db_write(TEST_DB, 40, &test_data)
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
        .db_read_multi(&vec![
            S7ReadAccess::bytes(TEST_DB, 0, 300),
            S7ReadAccess::bit(TEST_DB, 0, 1),
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
        .db_write_multi(&[
            S7WriteAccess::bytes(TEST_DB, 0, &10_u32.to_be_bytes()),
            S7WriteAccess::bit(TEST_DB, 0, 1, true),
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
        .db_read(TEST_DB, 40, 900)
        .await
        .expect("Could not read data from S7");

    assert_eq!(read_data.len(), 900);
}

#[tokio::test]
async fn continuous_test() {
    // create single s7 client object
    let pool = S7Pool::new(std::net::Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)
        .expect("Could not create pool");

    let mut cycles = 0;
    while cycles < 10 {
        // read data
        println!("{:?}", pool.db_read_bit(TEST_DB, 0, 1).await);

        tokio::time::sleep(Duration::from_secs(1)).await;

        cycles += 1;
    }
}

#[tokio::test]
async fn test_triggers() {
    // create single s7 client object
    let pool = S7Pool::new(std::net::Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)
        .expect("Could not create pool");

    let mut trigger_collection = pool
        .new_trigger_collection(&[("AHA", S7ReadAccess::bit(TEST_DB, 0, 1))])
        .unwrap();

    let mut iterator = 0;
    while iterator < 5 {
        pool.db_write_bit(TEST_DB, 0, 1, iterator % 2 == 0)
            .await
            .unwrap();

        // read data
        trigger_collection.update().await.unwrap();

        if iterator % 2 == 0 {
            assert_eq!(trigger_collection.positive_flank("AHA"), Some(true));
        } else {
            assert_eq!(trigger_collection.negative_flank("AHA"), Some(true));
        }

        iterator += 1;
    }
}

#[tokio::test]
async fn bit_error_test() -> Result<(), Error> {
    // create single s7 client object
    let pool = S7Pool::new(std::net::Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;

    assert_eq!(
        pool.db_read_bit(TEST_DB, 0, 8).await,
        Err(Error::RequestedBitOutOfRange),
    );

    assert_eq!(
        pool.db_read_multi(&[S7ReadAccess::bit(TEST_DB, 0, 9)])
            .await,
        Err(Error::RequestedBitOutOfRange),
    );

    Ok(())
}

#[tokio::test]
async fn test_multi_connection_pool() -> Result<(), Error> {
    // create single s7 client object
    let pool = S7Pool::new(std::net::Ipv4Addr::new(192, 168, 10, 72), S7Types::S71200)?;

    let pool_1 = pool.clone();
    tokio::spawn(async move {
        loop {
            assert!(pool_1.db_read(TEST_DB, 0, 10).await.is_ok());
        }
    });

    let pool_2 = pool.clone();
    tokio::spawn(async move {
        loop {
            assert!(pool_2.db_read(TEST_DB, 0, 10).await.is_ok());
        }
    });

    let pool_3 = pool.clone();
    tokio::spawn(async move {
        loop {
            assert!(pool_3.db_read(TEST_DB, 0, 10).await.is_ok());
        }
    });

    let pool_4 = pool.clone();
    tokio::spawn(async move {
        loop {
            assert!(pool_4.db_read(TEST_DB, 0, 10).await.is_ok());
        }
    });

    let pool_5 = pool.clone();
    tokio::spawn(async move {
        loop {
            assert!(pool_5.db_read(TEST_DB, 0, 10).await.is_ok());
        }
    });

    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    Ok(())
}
