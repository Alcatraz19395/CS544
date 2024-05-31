// Define message types for the protocol
pub const MSG_TYPE_DATA: u8 = 1;  // Indicates a data message
pub const MSG_TYPE_END: u8 = 2;   // Indicates the end of the transmission

use serde::{Serialize, Deserialize};

// Define the Protocol Data Unit (PDU) structure
#[derive(Serialize, Deserialize, Debug)]
pub struct PDU {
    pub msg_type: u8,           // Type of the message (data or end)
    pub sequence_number: u32,   // Sequence number of the PDU
    pub payload: Vec<u8>,       // Payload of the PDU
    pub checksum: u32,          // Checksum for error detection
}

// Serialize a PDU to a byte vector
pub fn serialize_pdu(pdu: &PDU) -> Vec<u8> {
    bincode::serialize(pdu).expect("Failed to serialize PDU")
}

// Deserialize a byte slice to a PDU
pub fn deserialize_pdu(data: &[u8]) -> PDU {
    bincode::deserialize(data).expect("Failed to deserialize PDU")
}
