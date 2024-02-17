use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::merkle_tree;

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Upload {
        client_files: BTreeMap<String, Vec<u8>>,
    },
    Download {
        filename: String,
    },
    GetMerkleProof {
        filename: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Success { data: Vec<u8> },
    MerkleProof { proof: Vec<(Vec<u8>, bool)> },
    Error { message: String },
}

async fn send_server_message(
    server_addr: &str,
    message: ServerMessage,
) -> io::Result<ClientMessage> {
    let mut stream = TcpStream::connect(server_addr).await?;
    let message = serde_json::to_vec(&message)?;
    stream.write_u64(message.len() as u64).await?;
    stream.write_all(&message).await?;
    stream.flush().await?;

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;

    let response: ClientMessage = serde_json::from_slice(&buffer)?;
    Ok(response)
}

pub fn compute_merkle_root_hash(data: Vec<Vec<u8>>) -> Vec<u8> {
    let merkle_tree = merkle_tree::MerkleTree::new(data);
    merkle_tree.get_root_hash()
}

pub fn verify_merkle_proof(proof: &[(Vec<u8>, bool)], root: &Vec<u8>, leaf: &Vec<u8>) -> bool {
    let result = merkle_tree::MerkleTree::verify_proof(proof, root, leaf);
    if result {
        println!("Merkle Proof verified succesfully");
    }
    result
}

pub async fn upload_files(
    client_files: BTreeMap<String, Vec<u8>>,
    server_addr: &str,
) -> io::Result<()> {
    let message = ServerMessage::Upload { client_files };
    let response = send_server_message(server_addr, message).await?;

    match response {
        ClientMessage::Success { data } => {
            println!(
                "Files uploaded successfully. Merkle Root Hash from Server: {:?}",
                data
            );
            Ok(())
        }
        ClientMessage::Error { message } => {
            println!("Failed to upload files: {}", message);
            Err(io::Error::new(io::ErrorKind::Other, message))
        }
        _ => {
            println!("Unexpected response from server");
            Err(io::Error::new(io::ErrorKind::Other, "Unexpected response"))
        }
    }
}

pub async fn download_file(filename: &str, server_addr: &str) -> io::Result<Vec<u8>> {
    let message = ServerMessage::Download {
        filename: filename.to_string(),
    };
    let response = send_server_message(server_addr, message).await?;

    match response {
        ClientMessage::Success { data } => {
            println!("File downloaded successfully");
            Ok(data)
        }
        ClientMessage::Error { message } => {
            println!("Failed to download file: {}", message);
            Err(io::Error::new(io::ErrorKind::Other, message))
        }
        _ => {
            println!("Unexpected response from server");
            Err(io::Error::new(io::ErrorKind::Other, "Unexpected response"))
        }
    }
}

pub async fn get_merkle_proof(
    filename: &str,
    server_addr: &str,
) -> io::Result<Vec<(Vec<u8>, bool)>> {
    let message = ServerMessage::GetMerkleProof {
        filename: filename.to_string(),
    };
    let response = send_server_message(server_addr, message).await?;

    match response {
        ClientMessage::MerkleProof { proof } => {
            println!("Merkle Proof fetched successfully");
            Ok(proof)
        }
        ClientMessage::Error { message } => {
            println!("Failed to fetch Merkle proof: {}", message);
            Err(io::Error::new(io::ErrorKind::Other, message))
        }
        _ => {
            println!("Unexpected response from server");
            Err(io::Error::new(io::ErrorKind::Other, "Unexpected response"))
        }
    }
}
