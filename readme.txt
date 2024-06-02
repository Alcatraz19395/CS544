# QUIC File Transfer

This project demonstrates a simple file transfer application using QUIC protocol in Rust with the `quinn` library.

## Requirements

- Rust (latest stable version)
- `tokio` for async runtime
- `quinn` for QUIC implementation
- `rustls` for TLS support
- `futures-util` for stream utilities

## Installation

### Install Rust

If you don't have Rust installed, you can install it using `rustup`. Follow the instructions at [rustup.rs](https://rustup.rs/) or run the following command in your terminal:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

After installation, make sure to add the Rust binaries to your PATH. You can verify the installation with:
rustc --version
cargo --version

Install OpenSSL

For Unix-like systems (Linux, macOS), you can install OpenSSL using your package manager. For example, on Ubuntu, you can run:
sudo apt-get update
sudo apt-get install openssl libssl-dev

On macOS, you can use Homebrew:
brew install openssl


PDU
Message Types:

MSG_TYPE_DATA: This constant represents a data message. It's used to indicate that the PDU contains data to be processed.
MSG_TYPE_END: This constant indicates the end of the transmission. It's used to signal that no more data will be sent.
PDU Structure:

msg_type: A u8 that represents the type of the message. It helps in distinguishing between data messages and end-of-transmission messages.
sequence_number: A u32 that represents the sequence number of the PDU. This is crucial for ensuring that data is received and processed in the correct order, especially in protocols where packets might arrive out of order.
payload: A Vec<u8> that holds the actual data being transmitted. This allows for flexible and variable-length data transmission.
checksum: A u32 that holds a checksum for error detection. This ensures data integrity by allowing the receiver to verify that the data has not been corrupted during transmission.
Serialization and Deserialization Functions:

serialize_pdu: This function takes a PDU and serializes it into a byte vector using the bincode library. This byte vector can then be sent over the network.
deserialize_pdu: This function takes a byte slice and deserializes it into a PDU using the bincode library. This allows the receiver to reconstruct the original PDU from the received bytes.

## Setup
1. Clone the repository:

    ```sh
    git clone https://github.com/Alcatraz19395/CS5444.git
    cd quic_file_transfer
    ```

2. Generate the necessary certificates and keys:

    ```sh
    # Generate server certificate and key
    openssl req -x509 -newkey rsa:2048 -keyout server_key.pem -out server_cert.pem -days 365 -nodes
    openssl x509 -outform der -in server_cert.pem -out server_cert.der
    openssl pkcs8 -topk8 -inform PEM -outform DER -in server_key.pem -out server_key.der -nocrypt

    # Generate client certificate and key
    openssl req -x509 -newkey rsa:2048 -keyout client_key.pem -out client_cert.pem -days 365 -nodes
    openssl x509 -outform der -in client_cert.pem -out client_cert.der
    openssl pkcs8 -topk8 -inform PEM -outform DER -in client_key.pem -out client_key.der -nocrypt

    # Generate CA certificate
    openssl req -x509 -newkey rsa:2048 -keyout ca_key.pem -out ca_cert.pem -days 365 -nodes
    openssl x509 -outform der -in ca_cert.pem -out ca_cert.der
    ```

3. Create a file to transfer:

    ```sh
    echo "This is a test file." > file_to_send
    ```

## Running the Project

1. Start the server:

    ```sh
    cargo run --bin server -- --server-addr 127.0.0.1 --server-cert server_cert.der --server-key server_key.der
    ```

2. Run the client:

    ```sh
    cargo run --bin client -- --server-addr 127.0.0.1:5000 --server-name localhost --client-cert client_cert.der --client-key client_key.der --ca-cert ca_cert.der
    ```

## Configuration

The client and server can be configured using command line arguments. 

### Server Arguments

- `--server-addr`: Server address (e.g., `127.0.0.1`)
- `--server-cert`: Path to the server certificate (in DER format)
- `--server-key`: Path to the server key (in DER format)

### Client Arguments

- `--server-addr`: Server address (e.g., `127.0.0.1`)
- `--server-port`: Server port (e.g., `5000`). This is hardcoded
- `--client-cert`: Path to the client certificate (in DER format)
- `--client-key`: Path to the client key (in DER format)
- `--ca-cert`: Path to the CA certificate (in DER format)
- `--file-to-send`: Path to the file to be sent to the server.

Extra Credits
Summary: https://github.com/Alcatraz19395/CS544/blob/main/Summary_extra_credit
Demo: https://youtu.be/TY0lmwPWZcM
GitHub: https://github.com/Alcatraz19395/CS544
PPT: https://www.canva.com/design/DAGG7zc1VeA/DqkqaY0w5viNCehFmy8V2Q/view?utm_content=DAGG7zc1VeA&utm_campaign=designshare&utm_medium=link&utm_source=editor

