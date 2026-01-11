use std::time::Duration;

use async_channel::Sender;
use iroh::endpoint::Connection;
use tokio::task::JoinHandle;

use crate::{error::Res, networking::{error::NetworkError, packet::Packet}, util::channel::send};

#[derive(Debug)]
pub struct ForeignManager {
   connection: Connection,
   _receive_handle: JoinHandle<Res<()>>
}

impl ForeignManager {

    pub fn new(connection: Connection, packet_sender: Sender<(usize, Packet)>) -> ForeignManager {
        ForeignManager {
            connection: connection.clone(),
            _receive_handle: tokio::spawn(ForeignManager::receive(connection, packet_sender))
        }
    }

    /// Establish a bi-directional channel through which the message can be streamed.
    /// Ok(bool) represents the message being sent correctly, and the boolean indicates whether a confirmation was received.
    /// This function yields a future that must be executed.
    pub async fn send_task(connection: Connection, packet: Packet) -> Res<bool> {

        // Open a bi-directional channel to the targetted connection (usually a clone)
        let (mut send, mut recv) = connection.open_bi().await?;
        // Cache the security code
        let expected_reply = packet.code;
        // Write all the bytes into the stream before closing it, idiomatically signalling the end of this discrete packet.
        send.write_all(&packet.to_bytes()).await?;
        send.finish()?;

        // Create a buffer to accept the verification code.
        let mut buffer: Vec<u8> = Vec::new();
        Ok(match tokio::time::timeout(Duration::from_secs(5), recv.read(&mut buffer)).await {
            Ok(read_result) => match read_result {
                Ok(read_option) => match read_option {
                    Some(_bytes_read) => {
                        if buffer.len() == 4 {
                            let mut iterator = buffer.into_iter();

                            let endians = [
                                iterator.next().ok_or(NetworkError::MalformedCode)?,
                                iterator.next().ok_or(NetworkError::MalformedCode)?,
                                iterator.next().ok_or(NetworkError::MalformedCode)?,
                                iterator.next().ok_or(NetworkError::MalformedCode)?
                            ];

                            let code = u32::from_be_bytes(endians);

                            code == expected_reply
                        } else {
                            false
                        }
                    },
                    None => false // Special case where stream was finished early.
                },
                // Failed to read buffer (did not receive confirmation).
                Err(_) => false
            },
            // Timed out (did not receive confirmation).
            Err(_) => false
        })
    }

    pub async fn receive(connection: Connection, packet_sender: Sender<(usize, Packet)>) -> Res<()> {

        let author: usize = connection.stable_id();
        
        loop {

            // Accept a single bidirectional channel instance for this packet exchange.
            let (mut sender, mut receiver) = match connection.accept_bi().await {
                Ok(v) => v,
                _ => return Ok(())
            };

            println!("Accepted new BD channel!");

            let mut buffer: Vec<u8> = Vec::new();

            // All errors indicate failure of this transmission.
            if receiver.read(&mut buffer).await.is_err() {
                let _ = sender.finish();
                continue;
            }

            println!("Successfully read packet!");
            let packet = Packet::from_bytes(buffer)?;
            println!("Packet! {packet:?}");
            sender.write_all(&packet.code.to_be_bytes()).await?;
            println!("Response echoed!");
            let _ = sender.finish();
            println!("channel finished");


            // Send the packet off to be processed, alongside this connection id.
            send((author, packet), &packet_sender).await?;
        }

    }

    pub fn clone_connection(&self) -> Connection {
        self.connection.clone()
    }
}
