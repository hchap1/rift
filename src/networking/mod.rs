/*
    Protocol
    
    Streams (as per QUIC) are a one-time-use for a single distinct message.
    Each instance will own a Server which exclusively receives streams from incoming connections.
    When a node wishes to message another node, that node must spawn a client (long lived) that will transmit the message via short lived streams.
    Servers will have persistent IROH node addresses. Client node addresses are not persistent.

    Security / Identity
    Each Server has a persistent IROH node address.
    When a Server creates a Client to message another Server, the Client will include the node address of the Server, and an authentication code. The foreign Server will attempt to establish its own Client-Server connection with the Server, and upon success, will send the original authentication code as well as a foreign authentication code. The original Server can check the received codes, one of which should be the original, and one is a new one that it must relay back via the Client to the foreign Server. The foreign Server must listen for the code being echoed back by the client of the Server it just connected to.

    This prevents a Client pretending to be associated with a Server and ensures full-circle linked communication exists.

    If the foreign Server cannot connect with the original Server, either a technical failure has occured or the Client was lying about its Server.
    If the original Server receives no new connection, then that Server does not exist or is not a part of this protocol.
    If the original Server receives a connection request with a code it did not send, the connection request is invalid and will be refused.
    If the foreign Server does not receive its code back via the original Client, then the original Client is lying about its Server.
*/

const ALPN: &[u8] = b"hchap1/v1";

pub mod server;
pub mod connection_manager;
pub mod foreign_manager;
