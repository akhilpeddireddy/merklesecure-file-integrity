use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use crate::merkle_tree::MerkleTree;

#[derive(Serialize, Deserialize, Debug)]
enum ServerMessage {
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
enum ClientMessage {
    Success { data: Vec<u8> },
    MerkleProof { proof: Vec<(Vec<u8>, bool)> },
    Error { message: String },
}

pub struct Server {
    files: Arc<Mutex<BTreeMap<String, Vec<u8>>>>,
    server_mt: Arc<Mutex<MerkleTree>>,
}

impl Server {
    pub async fn start(&self, addr: &str) {
        let listener = TcpListener::bind(addr).await.expect("Failed to bind");
        loop {
            let (stream, _) = listener.accept().await.expect("Failed to accept");
            let files = Arc::clone(&self.files);
            let server_mt = Arc::clone(&self.server_mt);
            tokio::spawn(async move {
                handle_connection(stream, files, server_mt).await;
            });
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    files: Arc<Mutex<BTreeMap<String, Vec<u8>>>>,
    server_mt: Arc<Mutex<MerkleTree>>,
) {
    let mut length = [0u8; 8];
    if let Err(err) = stream.read_exact(&mut length).await {
        eprintln!("Read error: {}", err);
        return;
    }

    let length = u64::from_be_bytes(length);

    let mut buffer = vec![0u8; length as usize];
    if let Err(err) = stream.read_exact(&mut buffer).await {
        eprintln!("Read error: {}", err);
        return;
    }

    let message: Result<ServerMessage, _> = serde_json::from_slice(&buffer);
    match message {
        Ok(ServerMessage::Upload { client_files }) => {
            // Update files and merkle_tree
            let mut files_guard = files.lock().await;
            let mut new_data = false;
            for (filename, data) in client_files {
                if files_guard.insert(filename.clone(), data.clone()).is_none() {
                    new_data = true;
                }
            }
            // Only update the Merkle tree if new data was added
            if new_data {
                let all_data: Vec<Vec<u8>> = files_guard.values().cloned().collect();
                let new_merkle_tree = MerkleTree::new(all_data);
                // drop the MutexGuard over files before acquiring a new one over server_mt
                drop(files_guard);
                let mut server_mt = server_mt.lock().await;
                *server_mt = new_merkle_tree;
            }

            // Send a success message back to the client
            let root_hash = server_mt.lock().await.get_root_hash();
            let response = ClientMessage::Success { data: root_hash };
            let response = serde_json::to_vec(&response).unwrap();
            if let Err(err) = stream.write_all(&response).await {
                eprintln!("Write error: {}", err);
            }
        }
        Ok(ServerMessage::Download { filename }) => {
            // Try to find the requested file in our server files
            let file_data = files.lock().await.get(&filename).cloned();
            match file_data {
                Some(data) => {
                    let response = ClientMessage::Success { data };
                    let response = serde_json::to_vec(&response).unwrap();
                    if let Err(err) = stream.write_all(&response).await {
                        eprintln!("Write error: {}", err);
                    }
                }
                None => {
                    let response = ClientMessage::Error {
                        message: "File not found".to_string(),
                    };
                    let response = serde_json::to_vec(&response).unwrap();
                    if let Err(err) = stream.write_all(&response).await {
                        eprintln!("Write error: {}", err);
                    }
                }
            }
        }
        Ok(ServerMessage::GetMerkleProof { filename }) => {
            let files_guard = files.lock().await;
            if let Some(index) = files_guard.keys().position(|x| x == &filename) {
                let proof = server_mt.lock().await.get_proof_for(index);
                let response = ClientMessage::MerkleProof { proof };
                let response = serde_json::to_vec(&response).unwrap();
                if let Err(err) = stream.write_all(&response).await {
                    eprintln!("Write error: {}", err);
                }
            } else {
                let response = ClientMessage::Error {
                    message: "File not found".to_string(),
                };
                let response = serde_json::to_vec(&response).unwrap();
                if let Err(err) = stream.write_all(&response).await {
                    eprintln!("Write error: {}", err);
                }
            }
        }
        Err(err) => {
            eprintln!("Invalid client message: {}", err);
        }
    }
}

pub fn new_server() -> Arc<Server> {
    Arc::new(Server {
        files: Arc::new(Mutex::new(BTreeMap::new())),
        server_mt: Arc::new(Mutex::new(MerkleTree::new(vec![vec![]]))),
    })
}
