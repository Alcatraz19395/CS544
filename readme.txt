# QUIC File Transfer

This project demonstrates a simple file transfer application using QUIC protocol in Rust with the `quinn` library.

## Requirements

- Rust (latest stable version)
- `tokio` for async runtime
- `quinn` for QUIC implementation
- `rustls` for TLS support
- `futures-util` for stream utilities

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
