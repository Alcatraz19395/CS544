use structopt::StructOpt;
use std::{net::SocketAddr, sync::Arc};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use quinn::{Endpoint, ServerConfig};
use rustls::{Certificate, PrivateKey, ServerConfig as RustlsServerConfig};
use futures_util::stream::StreamExt;
use std::error::Error;
use crc32fast::Hasher;
use serde::{Serialize, Deserialize};
mod pdu;
use pdu::{deserialize_pdu, PDU, MSG_TYPE_DATA, MSG_TYPE_END};

/// Configuration for the server, parsed from command line arguments.
#[derive(StructOpt, Debug)]
struct Config {
    #[structopt(long)]
    server_addr: String,

    #[structopt(long)]
    server_cert: String,

    #[structopt(long)]
    server_key: String,
}

/// Possible states for the server during the connection and file reception process.
#[derive(Debug)]
enum ServerState {
    Start,
    Connected,
    Receiving,
    Finished,
    Error,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    const SERVER_PORT: u16 = 5000;

    // Parse command line arguments
    let config = Config::from_args();

    // Create a socket address for the server
    let addr: SocketAddr = format!("{}:{}", config.server_addr, SERVER_PORT).parse()?;

    // Read and parse the server certificate and key
    let cert = fs::read(&config.server_cert).await?;
    let key = fs::read(&config.server_key).await?;

    let cert = Certificate(cert);
    let key = PrivateKey(key);

    // Configure TLS settings using Rustls
    let rustls_config = RustlsServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;

    let server_config = ServerConfig::with_crypto(Arc::new(rustls_config));

    // Start the QUIC endpoint
    let (endpoint, mut incoming) = Endpoint::server(server_config, addr)?;

    let mut state = ServerState::Start;

    println!("Server running: Listening on {}", addr);

    // Accept incoming connections
    while let Some(connecting) = incoming.next().await {
        match connecting.await {
            Ok(new_conn) => {
                state = ServerState::Connected;
                tokio::spawn(handle_connection(new_conn));
            }
            Err(_) => {
                state = ServerState::Error;
            }
        }
    }

    println!("Server state: {:?}", state);
    Ok(())
}

/// Handles a new incoming connection to the server.
///
/// This function processes the incoming connection, receives data PDUs,
/// validates the checksum, and writes the received payload to a file.
async fn handle_connection(mut new_conn: quinn::NewConnection) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Connection established from: {:?}", new_conn.connection.remote_address());
    let mut state = ServerState::Connected;

    while let Some(Ok(mut recv)) = new_conn.uni_streams.next().await {
        let mut buffer = vec![0; 1024];
        let mut file = fs::File::create("received_file").await?;
        state = ServerState::Receiving;

        loop {
            // Read the length of the incoming PDU
            let mut length_buffer = [0u8; 4];
            if recv.read_exact(&mut length_buffer).await.is_err() {
                state = ServerState::Error;
                eprintln!("Failed to read PDU length");
                break;
            }
            let length = u32::from_be_bytes(length_buffer) as usize;

            // Read the PDU itself
            let mut pdu_buffer = vec![0; length];
            if recv.read_exact(&mut pdu_buffer).await.is_err() {
                state = ServerState::Error;
                eprintln!("Failed to read PDU");
                break;
            }

            // Deserialize the PDU
            let pdu: PDU = deserialize_pdu(&pdu_buffer);
            let mut hasher = Hasher::new();
            hasher.update(&pdu.payload);
            let calculated_checksum = hasher.finalize();

            // Validate the checksum
            println!(
                "Checksum details: received={}, calculated={}",
                pdu.checksum, calculated_checksum
            );
            if calculated_checksum != pdu.checksum {
                state = ServerState::Error;
                eprintln!("Checksum validation failed for sequence number {}", pdu.sequence_number);
                break;
            }

            // Handle the received PDU based on its type
            match pdu.msg_type {
                MSG_TYPE_DATA => {
                    println!("Received data chunk with sequence number {}", pdu.sequence_number);
                    file.write_all(&pdu.payload).await?;
                }
                MSG_TYPE_END => {
                    state = ServerState::Finished;
                    println!("Received end of transmission");
                    break;
                }
                _ => {
                    state = ServerState::Error;
                    eprintln!("Unknown message type: {}", pdu.msg_type);
                }
            }
        }
    }

    println!("Server state: {:?}", state);
    Ok(())
}
