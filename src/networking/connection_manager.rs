use std::collections::HashMap;

use async_channel::Sender;
use async_channel::unbounded;
use iroh::Endpoint;
use tokio::task::JoinHandle;

use crate::error::ChatError;
use crate::error::Error;
use crate::error::Res;
use crate::networking::packet::Packet;
use crate::networking::server::Foreign;
use crate::util::channel::send;

type Send = async_channel::Sender<ConnectionManagerMessage>;
type Recv = async_channel::Receiver<ConnectionManagerMessage>;

#[derive(Debug, Clone)]
pub enum ConnectionManagerMessage {
    Quit,
    Add(Foreign),
    Error(Error),

    // Output
    SuccessfulConnection(usize),

    // Message
    Message(usize, Packet)                  // Signal the management thread to find a client with this stable_id and distribute the packet to it.
}

#[derive(Debug, Clone)]
pub enum Distribution {}

#[derive(Debug)]
pub struct ConnectionManager {
    _listen_handle: JoinHandle<Res<()>>,
    _manage_handle: JoinHandle<Res<()>>,
    sender_to_thread: Send,
    output: Recv
}

impl ConnectionManager {

    pub fn new(endpoint: Endpoint, packet_sender: Sender<(usize, Packet)>) -> ConnectionManager {
        let (thread_sender, thread_receiver) = unbounded();
        let (output_sender, output_receiver) = unbounded();

        ConnectionManager {
            _listen_handle: tokio::task::spawn(Self::listen(endpoint, thread_sender.clone(), output_sender.clone(), packet_sender)),
            _manage_handle: tokio::task::spawn(Self::manage(thread_receiver, output_sender)),
            sender_to_thread: thread_sender,
            output: output_receiver
        }
    }

    async fn listen(endpoint: Endpoint, task_sender: Send, output: Send, packet_sender: Sender<(usize, Packet)>) -> Res<()> {
        loop {
            let res = match endpoint.accept().await {
                Some(accept) => accept.await,

                // None means the endpoint has been closed via Endpoint::close. Destroy the thread in this case.
                None => return Ok(())
            };

            match res {

                // A new connection has been aquired.
                Ok(connection) => send(ConnectionManagerMessage::Add(Foreign::new(connection, packet_sender.clone())), &task_sender).await?,
                Err(e) => send(ConnectionManagerMessage::Error(e.into()), &output).await?
            }
        }
    }

    async fn manage(receiver: Recv, sender: Send) -> Res<()> {

        let mut connections: HashMap<usize, Foreign> = HashMap::new();

        while let Ok(task) = receiver.recv().await {
            match task {
                ConnectionManagerMessage::Quit => return Ok(()),
                ConnectionManagerMessage::Add(connection) => {
                    send(ConnectionManagerMessage::SuccessfulConnection(connection.stable_id()), &sender).await?;
                    let _ = connections.insert(connection.stable_id(), connection);
                },
                ConnectionManagerMessage::Message(stable_id, packet) => {
                    println!("Connection manager received send task to {stable_id}");
                    println!("Connections: {connections:?}");
                    if let Some(foreign) = connections.get(&stable_id) {
                        println!("Found connection");
                        match foreign.distribute(packet).await {
                            Ok(is_valid) => if !is_valid { send(ConnectionManagerMessage::Error(ChatError::InvalidCode.into()), &sender).await? },
                            Err(error) => send(ConnectionManagerMessage::Error(error), &sender).await?
                        }
                    }
                }
                error => send(error, &sender).await?
            }
        }

        Ok(())
    }

    pub fn yield_sender(&self) -> Send {
        self.sender_to_thread.clone()
    }

    pub fn yield_output(&self) -> Recv {
        self.output.clone()
    }
}
