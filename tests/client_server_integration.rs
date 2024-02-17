use merklefile::{client, server};
use std::collections::BTreeMap;

#[tokio::test]
async fn test_client_server_interaction() {
    // Set up and start server
    let server_addr = "127.0.0.1:8080";
    let server_instance = server::new_server(); // Created a new instance of server
    tokio::spawn(async move {
        server_instance.start(server_addr).await; // used the instance to call start()
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Simulate client actions
    let mut files = BTreeMap::<String, Vec<u8>>::new();
    files.insert("test_file_1.txt".to_string(), b"Hello World".to_vec());
    files.insert(
        "test_file_2.txt".to_string(),
        b"This is file 2 contents".to_vec(),
    );
    files.insert("test_file_3.txt".to_string(), b"See you again".to_vec());

    let client_root_hash = client::compute_merkle_root_hash(files.values().cloned().collect());
    println!("Client root hash: {:?}", client_root_hash);

    // Upload files
    let upload_result = client::upload_files(files.clone(), server_addr).await;
    assert!(upload_result.is_ok(), "Files upload failed");

    // Delete local copies
    files.clear();

    // Download file and request Merkle proof
    let download_result = client::download_file("test_file_2.txt", server_addr).await;
    assert!(download_result.is_ok(), "File 2 download failed");

    let proof_result = client::get_merkle_proof("test_file_2.txt", server_addr).await;
    assert!(proof_result.is_ok(), "Merkle proof request failed");
    let server_proof = proof_result.unwrap();

    // Verify Merkle proof
    let downloaded_data = download_result.unwrap();
    let is_valid_proof =
        client::verify_merkle_proof(&server_proof, &client_root_hash, &downloaded_data);
    assert!(is_valid_proof, "Merkle proof verification failed");

    // Check if file contents are actually similar (sanity check: not a part of the actual client)
    assert_eq!(
        downloaded_data,
        b"This is file 2 contents".to_vec(),
        "Downloaded data does not match original"
    );
}
