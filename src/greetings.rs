use std::fmt::{Debug, Formatter};
use std::str::FromStr;

use crate::db::RepoError;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::migrate::MigrateError;
use sqlx::Pool;
use std::time::Duration;
use uuid::Uuid;

// use azure_core::error::{ErrorKind, ResultExt};
use futures_util::StreamExt;

pub struct GreetingRepositoryImpl {
    pool: Box<Pool<sqlx::Postgres>>,
}

#[async_trait]
pub trait GreetingRepository {
    async fn store(&mut self, greeting: Greeting) -> Result<(), RepoError>;
    async fn store_blob(&mut self, greeting: Greeting) -> Result<(), RepoError>;
    //  async fn store_blob2(
    //     container_name: &str,
    //     blob_name: &str,
    //     data: Vec<u8>,
    // ) -> Result<(), Error>;
}

impl Debug for GreetingRepositoryImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "GreetingRepository")
    }
}
impl GreetingRepositoryImpl {
    pub async fn new(pool: Box<Pool<sqlx::Postgres>>) -> Result<Self, RepoError> {
        Ok(Self { pool })
    }
}
#[async_trait]
impl GreetingRepository for GreetingRepositoryImpl {
    async fn store(&mut self, greeting: Greeting) -> Result<(), RepoError> {
        let mut transaction = self.pool.begin().await?;

        let id: (i64,) = sqlx::query_as("INSERT INTO greeting(message_id, \"from\", \"to\", heading, message, created) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id")
            .bind(Uuid::from_str(&*greeting.id).unwrap())
            .bind(greeting.from)
            .bind(greeting.to)
            .bind(greeting.heading)
            .bind(greeting.message)
            .bind(greeting.created)
            .fetch_one(&mut *transaction).await?;

        sqlx::query("INSERT INTO ikke_paa_logg(greeting_id) VALUES ($1)")
            .bind(id.0)
            .execute(&mut *transaction)
            .await?;

        transaction.commit().await?;
        Ok(())
    }
    async fn store_blob(&mut self, greeting: Greeting) -> Result<(), RepoError> {
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Greeting {
    id: String,
    to: String,
    from: String,
    heading: String,
    message: String,
    created: NaiveDateTime,
}

impl From<azure_core::Error> for RepoError {
    fn from(value: azure_core::Error) -> Self {
        RepoError {
            error_message: format!("{:?}", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use azure_identity::{create_default_credential, DefaultAzureCredential};
    use azure_sdk_storage_blob::prelude::*;
    use azure_sdk_storage_core::client::Client;
    use azure_storage_blobs::prelude::{BlobServiceClient, ClientBuilder};
    use std::fs::File;
    use std::io::Read;

    use azure_storage::{ConnectionString, StorageCredentials};
    use tokio;

    #[tokio::test]
    async fn test_store_blob_success() -> azure_core::Result<()> {
        // Replace with your Azure storage account details
        let account_name = "devstoreaccount1";  // e.g. "myaccount"
        let container_name = "greeting-blob-container";      // Your container name
        let blob_name = "Cargo.toml";      // The name of the blob you want to upload
        let file_path = "Cargo.toml";  // Path to your local file to upload
        let connection_string = ConnectionString::new(&"DefaultEndpointsProtocol=http;AccountName=devstoreaccount1;AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;BlobEndpoint=http://127.0.0.1:10000/devstoreaccount1;").expect("Error connectiongsrting");


        let blob_service = BlobServiceClient::new(
            connection_string.account_name.unwrap(),
            connection_string.storage_credentials()?,
        );

        let container_client = blob_service.container_client(container_name);
        // container_client
        //     .create()
        //     .public_access(PublicAccess::None)
        //     .await?;

        // Read file content into a vector of bytes
        let mut file = File::open(file_path).expect("Failed to open file");
        let mut file_contents = Vec::new();
        file.read_to_end(&mut file_contents).expect("Failed to read file");

        // Create a blob client
        let blob_client = container_client.blob_client(blob_name);

        // Upload the file content as a block blob
        match blob_client.put_block_blob(file_contents).await {
            Ok(response) => {
                println!("Blob uploaded successfully! Response: {:?}", response);
            }
            Err(e) => {
                eprintln!("Error uploading blob: {:?}", e);
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn write_blob_to_azurite() -> azure_core::Result<()> {
        let account_name = "devstoreaccount1";  // e.g. "myaccount"
        let container_name = "greeting-blob-container";      // Your container name
        let blob_name = "Cargo.toml";      // The name of the blob you want to upload
        let file_path = "Cargo.toml";  // Path to your local file to upload
    let connection_string = ConnectionString::new(&"DefaultEndpointsProtocol=http;AccountName=devstoreaccount1;AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;BlobEndpoint=http://127.0.0.1:10000/devstoreaccount1;").expect("Error connection string");

    let blob_service = BlobServiceClient::new(
        connection_string.account_name.unwrap(),
        connection_string.storage_credentials()?,
    );

    let container_client = blob_service.container_client(container_name);

    // Read file content into a vector of bytes
    let mut file = File::open(file_path).expect("Failed to open file");
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents).expect("Failed to read file");

    // Create a blob client
    let blob_client = container_client.blob_client(blob_name);

    // Upload the file content as a block blob
    match blob_client.put_block_blob(file_contents).await {
        Ok(response) => {
            println!("Blob uploaded successfully! Response: {:?}", response);
        }
        Err(e) => {
            eprintln!("Error uploading blob: {:?}", e);
        }
    }

    Ok(())
}
}
