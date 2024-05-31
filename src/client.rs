use structopt::StructOpt;
use std::{net::SocketAddr, sync::Arc};
use tokio::fs;
use tokio::io::AsyncReadExt;
use quinn::{ClientConfig, Endpoint};
use rustls::{Certificate, PrivateKey, RootCertStore, ClientConfig as RustlsClientConfig};
use std::error::Error;
use crc32fast::Hasher; // For checksum calculation
use serde::{Serialize, Deserialize};
mod pdu;
use pdu::{serialize_pdu, PDU, MSG_TYPE_DATA, MSG_TYPE_END};

#[derive(StructOpt, Debug)]
struct Config {
    // The server address to connect to
    #[structopt(long)]
    server_addr: String,

    // The server port to connect to
    #[structopt(long)]
    server_port: u16,

    // Path to the client certificate file (in DER format)
    #[structopt(long)]
    client_cert: String,

    // Path to the client key file (in DER format)
    #[structopt(long)]
    client_key: String,

    // Path to the CA certificate file (in DER format)
    #[structopt(long)]
    ca_cert: String,

    // Path to the file to be sent
    #[structopt(long)]
    file_to_send: String,
}

// Client states for the protocol
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

    // Combine server address and port into a single SocketAddr
    let addr: SocketAddr = format!("{}:{}", config.server_addr, config.server_port).parse()?;

    // Load client certificate, key, and CA certificate
    let cert = fs::read(&config.client_cert).await?;
    let key = fs::read(&config.client_key).await?;
    let ca_cert = fs::read(&config.ca_cert).await?;

    // Create Certificate and PrivateKey objects from the loaded data
    let cert = Certificate(cert);
    let key = PrivateKey(key);

    // Create a RootCertStore and add the CA certificate to it
    let mut ca_cert_store = RootCertStore::empty();
    ca_cert_store.add(&Certificate(ca_cert))?;

    // Configure the Rustls client with the certificates and key
    let rustls_config = RustlsClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(ca_cert_store)
        .with_single_cert(vec![cert], key)?;

    // Create a QUIC client configuration
    let client_config = ClientConfig::new(Arc::new(rustls_config));
    let endpoint = Endpoint::client("[::]:0".parse().unwrap())?;

    let mut state = ClientState::Start;

    // Connect to the server
    match endpoint.connect_with(client_config, addr, "localhost")?.await {
        Ok(quinn::NewConnection { connection, .. }) => {
            println!("connected: addr={}", connection.remote_address());
            state = ClientState::Connected;

            // Open a unidirectional stream to send data
            let mut send = connection.open_uni().await?;
            let mut file = fs::File::open(&config.file_to_send).await?;
            let mut buffer = vec![0; 1024];
            let mut sequence_number = 0;

            state = ClientState::Sending;
            // Read the file and send its contents as PDUs
            loop {
                let n = file.read(&mut buffer).await?;
                if n == 0 {
                    break;
                }

                // Calculate checksum for the data
                let mut hasher = Hasher::new();
                hasher.update(&buffer[..n]);
                let checksum = hasher.finalize();

                // Create a PDU with the read data
                let pdu = PDU {
                    msg_type: MSG_TYPE_DATA,
                    sequence_number,
                    payload: buffer[..n].to_vec(),
                    checksum,
                };

                // Serialize the PDU and send it over the stream
                let serialized = serialize_pdu(&pdu);
                send.write_all(&serialized).await?;
                sequence_number += 1;
            }

            // Send an end of transmission PDU
            let pdu = PDU {
                msg_type: MSG_TYPE_END,
                sequence_number,
                payload: Vec::new(),
                checksum: 0,
            };
            let serialized = serialize_pdu(&pdu);
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
