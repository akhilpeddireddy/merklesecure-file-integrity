# MerkleSecure: Robust File Integrity Verification System

The MerkleSecure project provides a robust solution for uploading, storing, and verifying the integrity of files on a server using Merkle trees. This system ensures that clients can verify the integrity of their files at any time, leveraging the efficiency of Merkle proofs for integrity checks without needing to access the entire file set. This README guides you through setting up and running a demonstration of the system, that simulates the client-server interaction.

## Prerequisites

Before running the code, ensure you have the following installed on your system:

- Rust programming language: The project is developed in Rust, so you'll need the Rust compiler and Cargo (the Rust package manager) installed. You can install both by following the instructions on the [official Rust website](https://www.rust-lang.org/tools/install).

- Git: To clone the repository, you'll need Git installed. You can download it from [git-scm.com](https://git-scm.com/).

## Installation

1. **Clone the Repository:**

   Open a terminal and run the following command to clone the project repository:

   ```sh
   git clone https://github.com/akhilpeddireddy/merkle-file-server-client.git
   ```

2. **Navigate to the Project Directory:**

   Change into the project directory:

   ```sh
   cd merkle-file-server-client
   ```

## Running the Demo

The project includes an integration test that simulates the uploading of files to the server, deleting local copies, downloading files, and verifying their integrity using Merkle proofs. Follow these steps to run the demo:

1. **Build the Project:**

   Compile the project with Cargo to ensure all dependencies are downloaded and the project is built:

   ```sh
   cargo build
   ```

2. **Run the Integration Test (Demo):**

   Execute the integration test using Cargo:

   ```sh
   make demo
   ```
    or
   ```sh
   cargo test -- --nocapture test_client_server_interaction
   ```

   The `--nocapture` flag allows you to see printed output from the test, which includes log messages detailing the steps of the test, such as file uploads, download, and the results of integrity checks.

## Understanding the Demo

The output of the integration test will show a series of steps being performed, including:

- Setting up and starting the server.
- Uploading files from the client to the server.
- Storing Root Hash and Deleting the local copies on the client side.
- Downloading a file from the server with its Merkle proof.
- Verifying the downloaded file's integrity using the Merkle proof.

## Conclusion

This project demonstrates a practical application of Merkle trees for ensuring the integrity of files stored on a server. The modular design and asynchronous implementation make this project a solid foundation for further development and exploration of file integrity verification solutions.