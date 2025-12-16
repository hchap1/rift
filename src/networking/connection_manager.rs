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
use crate::util::channel::send;

pub type AM<T> = Arc<Mutex<T>>;

#[derive(Debug, Clone)]
pub enum ConnectionManagerMessage {
    Error(Error)
}

pub struct ConnectionManager {
    _listen_handle: JoinHandle<Res<()>>,
    connections: AM<HashMap<usize, Foreign>>,
    sender_to_thread: Sender<ConnectionManagerMessage>,
    output: Receiver<ConnectionManagerMessage>
}

impl ConnectionManager {

    pub fn new(endpoint: Endpoint) -> ConnectionManager {
        let connections = Arc::new(Mutex::new(HashMap::new()));
        let (thread_sender, thread_receiver) = unbounded();
        let (output_sender, output_receiver) = unbounded();
        let connections_clone = connections.clone();

        ConnectionManager {
            _listen_handle: tokio::task::spawn(Self::listen(endpoint, connections_clone, output_sender)),
            connections,
            sender_to_thread: thread_sender,
            output: output_receiver
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
                
                Err(e) => send(ConnectionManagerMessage::Error(e.into()), &sender).await?
            }
        }
    }
}
