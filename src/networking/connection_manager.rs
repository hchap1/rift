use std::collections::HashMap;

use async_channel::unbounded;
use iroh::Endpoint;
use tokio::task::JoinHandle;

use crate::error::Error;
use crate::error::Res;
use crate::networking::server::Foreign;
use crate::util::channel::send;

type Sender = async_channel::Sender<ConnectionManagerMessage>;
type Receiver = async_channel::Receiver<ConnectionManagerMessage>;

#[derive(Debug, Clone)]
pub enum ConnectionManagerMessage {
    Quit,
    Add(Foreign),
    Error(Error)
}

pub struct ConnectionManager {
    _listen_handle: JoinHandle<Res<()>>,
    _manage_handle: JoinHandle<Res<()>>,
    sender_to_thread: Sender,
    output: Receiver
}

impl ConnectionManager {

    pub fn new(endpoint: Endpoint) -> ConnectionManager {
        let (thread_sender, thread_receiver) = unbounded();
        let (output_sender, output_receiver) = unbounded();

        ConnectionManager {
            _listen_handle: tokio::task::spawn(Self::listen(endpoint, thread_sender.clone(), output_sender.clone())),
            _manage_handle: tokio::task::spawn(Self::manage(thread_receiver, output_sender)),
            sender_to_thread: thread_sender,
            output: output_receiver
        }
    }

    async fn listen(endpoint: Endpoint, task_sender: Sender, output: Sender) -> Res<()> {
        loop {
            let res = match endpoint.accept().await {
                Some(accept) => accept.await,

                // None means the endpoint has been closed via Endpoint::close. Destroy the thread in this case.
                None => return Ok(())
            };

            match res {

                // A new connection has been aquired.
                Ok(connection) => send(ConnectionManagerMessage::Add(Foreign::new(connection)), &task_sender).await?,
                Err(e) => send(ConnectionManagerMessage::Error(e.into()), &output).await?
            }
        }
    }

    async fn manage(receiver: Receiver, sender: Sender) -> Res<()> {

        let mut connections: HashMap<usize, Foreign> = HashMap::new();

        while let Ok(task) = receiver.recv().await {
            match task {
                ConnectionManagerMessage::Quit => return Ok(()),
                ConnectionManagerMessage::Add(connection) => { let _ = connections.insert(connection.stable_id(), connection); },
                error => send(error, &sender).await?
            }
        }

        Ok(())
    }

    pub fn yield_sender(&self) -> Sender {
        self.sender_to_thread.clone()
    }
}
