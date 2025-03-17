//create a module named blob_writer in this file and move the following functions to the module:
// - store_blob
// - store_blob_success
// - write_blob_to_azurite
//
// Then, update the mod declarations to bring the module into scope:
// mod blob_writer;




#[cfg(test)]
mod tests {
    use super::*;
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
        let connection_string = ConnectionString::new(&"DefaultEndpointsProtocol=http;AccountName=devstoreaccount1;AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;BlobEndpoint=http://127.0.0.1:10000/devstoreaccount1;").expect("Error connection string");

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

}







