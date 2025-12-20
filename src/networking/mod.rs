/*
    Overview
    Each 'Server' is a permanent IROH node address stored in a database. The Server listens for clients and accepts them as long as they provide the correct ALPN.
    Each of the Server's clients is a foreign Server. All communication is done through single-use bidirectional streams.
    Each complete packet exchance (full message / file) is verified after transmission.

    Protocol
    The first byte of the packet identifies its type.
    The second-fifth bytes of the packet are a unique 32-bit identifier for that packet that must be echoed back to confirm transmission.
    The rest of the bytes are 'data' as defined by the standard for that packet type.
*/

const ALPN: &[u8] = b"hchap1/v1";

pub mod server;
pub mod connection_manager;
pub mod foreign_manager;
pub mod packet;
pub mod error;
