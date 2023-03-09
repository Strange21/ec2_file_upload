use ipfs_api::{IpfsApi, IpfsClient};
use std::{fs::File, io::Error, fs::read_dir};
use aws_config::load_from_env;
use aws_sdk_dynamodb::{Client, Error as aws_error, Region, model::AttributeValue};
use std::collections::HashMap;
// use rusoto_core::{Region, RusotoError};
// use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, PutItemError, PutItemInput};

#[tokio::main]
async fn main() -> Result<(), Error>{
    // Create an instance of the IPFS API client
    let client  = IpfsClient::default();

    // Get the latest file which ever is added to the folder.
    let dir_path = "/home/ubuntu/uploads/";

    let latest_file = read_dir(dir_path)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .max_by_key(|path| path.metadata().unwrap().modified().unwrap())
        .unwrap();

    println!("Latest file: {:?}", latest_file);

    // Open the file to be uploaded
    let file = File::open(latest_file.clone()).expect("Unable to open file");

    // Upload the file to IPFS
    let res = client.add(file).await.expect("Failed to add file to IPFS");

    // Print the IPFS hash of the uploaded file
    println!("IPFS hash: {}", res.hash);
    let cid = res.hash;
    let filename = latest_file.as_os_str().to_str().unwrap();
    // let result = add_item_to_dynamodb(&cid, &filename).await.unwrap();

    // Load AWS credentials from environment variables.
    let config = aws_config::load_from_env().await;
    let table_name = "cid_table";
    
    // Create a DynamoDB client.
    let dynamodb_client = Client::new(&config);

    // Define the table name and item to add.
    let res = dynamodb_client
        .put_item()
        .table_name(table_name)
        .item("id", AttributeValue::S(filename.to_string()))
        .item("payload", AttributeValue::S(cid))
        .send()
        .await;

    println!("Item added to DynamoDB table: {:?}", res);

    Ok(())
}