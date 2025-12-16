use std::collections::HashMap;
use std::sync::Arc;

use async_channel::Receiver;
use async_channel::Sender;
use async_channel::unbounded;
use iroh::Endpoint;
use iroh::EndpointAddr;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::error::Error;
use crate::error::Res;
use crate::networking::server::Foreign;

pub type AM<T> = Arc<Mutex<T>>;

#[derive(Debug, Clone)]
pub enum ConnectionManagerMessage {
    Error(Error)
}

pub struct ConnectionManager {
    connections: AM<HashMap<usize, Foreign>>,
    sender_to_thread: Sender<ConnectionManagerMessage>
}

impl ConnectionManager {

    pub fn new(endpoint: Endpoint) -> ConnectionManager {
        let connections = Arc::new(Mutex::new(HashMap::new()));
        let (sender, receiver) = unbounded();

        ConnectionManager {
            connections,
            sender_to_thread: sender
        }
    }

    async fn listen(endpoint: Endpoint, connections: AM<HashMap<usize, Foreign>>, sender: Sender<ConnectionManagerMessage>) -> Res<()> {
        loop {
            let res = match endpoint.accept().await {
                Some(accept) => accept.await,

                // None means the endpoint has been closed via Endpoint::close. Destroy the thread in this case.
                None => return Ok(())
            };

            match res {

                // A new connection has been aquired.
                Ok(connection) => {
                    let mut lock = connections.lock().await;
                    lock.insert(connection.stable_id(), Foreign::new(connection));
                },
                
                Err(e) => sender.send(ConnectionManagerMessage::Error(Error::from(e))).await?
            }
        }
    }
}
