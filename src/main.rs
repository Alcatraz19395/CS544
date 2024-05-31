use structopt::StructOpt;
use std::{net::SocketAddr, sync::Arc};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use quinn::{Endpoint, ServerConfig};
use rustls::{Certificate, PrivateKey, ServerConfig as RustlsServerConfig};
use futures_util::stream::StreamExt;
use std::error::Error;
use crc32fast::Hasher; // For checksum calculation
use serde::{Serialize, Deserialize};
mod pdu;
use pdu::{deserialize_pdu, PDU, MSG_TYPE_DATA, MSG_TYPE_END};

#[derive(StructOpt, Debug)]
struct Config {
    // The address the server will bind to
    #[structopt(long)]
    server_addr: String,

    // The port the server will listen on
    #[structopt(long)]
    server_port: u16,

    // Path to the server's certificate file (in DER format)
    #[structopt(long)]
    server_cert: String,

    // Path to the server's key file (in DER format)
    #[structopt(long)]
    server_key: String,
}

// Server states for the protocol
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
    // Parse command line arguments
    let config = Config::from_args();

    // Combine server address and port into a single SocketAddr
    let addr: SocketAddr = format!("{}:{}", config.server_addr, config.server_port).parse()?;

    // Load server certificate and key from files
    let cert = fs::read(&config.server_cert).await?;
    let key = fs::read(&config.server_key).await?;

    // Create Certificate and PrivateKey objects from the loaded data
    let cert = Certificate(cert);
    let key = PrivateKey(key);

    // Configure the Rustls server with the certificate and key
    let rustls_config = RustlsServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;

    // Create a QUIC server configuration
    let server_config = ServerConfig::with_crypto(Arc::new(rustls_config));

    // Bind the QUIC server to the specified address
    let (endpoint, mut incoming) = Endpoint::server(server_config, addr)?;

    let mut state = ServerState::Start;

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

async fn handle_connection(mut new_conn: quinn::NewConnection) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Connection established: {:?}", new_conn.connection.remote_address());
    let mut state = ServerState::Connected;

    // Handle incoming unidirectional streams
    while let Some(Ok(mut recv)) = new_conn.uni_streams.next().await {
        let mut buffer = vec![0; 1024];
        let mut file = fs::File::create("received_file").await?;
        state = ServerState::Receiving;

        // Read data from the stream
        while let Some(n) = recv.read(&mut buffer).await? {
            if n == 0 {
                break;
            }

            // Deserialize the received data into a PDU
            let pdu: PDU = deserialize_pdu(&buffer[..n]);

            // Calculate checksum for the data
            let mut hasher = Hasher::new();
            hasher.update(&pdu.payload);
            let calculated_checksum = hasher.finalize();

            // Validate the checksum
            if calculated_checksum != pdu.checksum {
                state = ServerState::Error;
                eprintln!("Checksum validation failed for sequence number {}", pdu.sequence_number);
                break;
            }

            match pdu.msg_type {
                MSG_TYPE_DATA => {
                    // Write the received data payload to a file
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
