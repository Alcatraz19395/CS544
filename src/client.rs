use structopt::StructOpt;
use std::{net::SocketAddr, sync::Arc};
use tokio::fs;
use tokio::io::AsyncReadExt;
use quinn::{ClientConfig, Endpoint};
use rustls::{Certificate, PrivateKey, RootCertStore, ClientConfig as RustlsClientConfig};
use std::error::Error;
use crc32fast::Hasher;
use serde::{Serialize, Deserialize};
mod pdu;
use pdu::{serialize_pdu, PDU, MSG_TYPE_DATA, MSG_TYPE_END};

/// Configuration for the client, parsed from command line arguments.
#[derive(StructOpt, Debug)]
struct Config {
    #[structopt(long)]
    server_addr: String,

    #[structopt(long)]
    server_port: u16,

    #[structopt(long)]
    client_cert: String,

    #[structopt(long)]
    client_key: String,

    #[structopt(long)]
    ca_cert: String,

    #[structopt(long)]
    file_to_send: String,
}

/// Possible states for the client during the connection and file transfer process.
#[derive(Debug)]
enum ClientState {
    Start,
    Connected,
    Sending,
    Finished,
    Error,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Parse command line arguments
    let config = Config::from_args();

    // Create a socket address for the server
    let addr: SocketAddr = format!("{}:{}", config.server_addr, config.server_port).parse()?;

    // Read and parse the client certificate, key, and CA certificate
    let cert = fs::read(&config.client_cert).await?;
    let key = fs::read(&config.client_key).await?;
    let ca_cert = fs::read(&config.ca_cert).await?;

    let cert = Certificate(cert);
    let key = PrivateKey(key);

    // Initialize the root certificate store and add the CA certificate
    let mut ca_cert_store = RootCertStore::empty();
    ca_cert_store.add(&Certificate(ca_cert))?;

    // Configure TLS settings using Rustls
    let rustls_config = RustlsClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(ca_cert_store)
        .with_single_cert(vec![cert], key)?;

    let client_config = ClientConfig::new(Arc::new(rustls_config));
    let endpoint = Endpoint::client("[::]:0".parse().unwrap())?;

    let mut state = ClientState::Start;

    // Connect to the server
    match endpoint.connect_with(client_config, addr, "localhost")?.await {
        Ok(quinn::NewConnection { connection, .. }) => {
            println!("Client running: Connected to server at {}", connection.remote_address());
            state = ClientState::Connected;

            // Open a unidirectional stream to send data
            let mut send = connection.open_uni().await?;
            let mut file = fs::File::open(&config.file_to_send).await?;
            let mut buffer = vec![0; 1024];
            let mut sequence_number = 0;

            state = ClientState::Sending;
            loop {
                // Read data from the file
                let n = file.read(&mut buffer).await?;
                if n == 0 {
                    break;
                }

                // Compute checksum
                let mut hasher = Hasher::new();
                hasher.update(&buffer[..n]);
                let checksum = hasher.finalize();

                // Create and serialize PDU
                let pdu = PDU {
                    msg_type: MSG_TYPE_DATA,
                    sequence_number,
                    payload: buffer[..n].to_vec(),
                    checksum,
                };

                let serialized = serialize_pdu(&pdu);
                let length = (serialized.len() as u32).to_be_bytes();
                
                // Send length and data
                send.write_all(&length).await?;
                send.write_all(&serialized).await?;
                sequence_number += 1;
            }

            // Send end of transmission message
            let pdu = PDU {
                msg_type: MSG_TYPE_END,
                sequence_number,
                payload: Vec::new(),
                checksum: 0,
            };
            let serialized = serialize_pdu(&pdu);
            let length = (serialized.len() as u32).to_be_bytes();
            send.write_all(&length).await?;
            send.write_all(&serialized).await?;
            send.finish().await?;
            state = ClientState::Finished;
        }
        Err(_) => {
            state = ClientState::Error;
        }
    }

    println!("Client state: {:?}", state);
    Ok(())
}
