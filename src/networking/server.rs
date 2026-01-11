use std::sync::Arc;

use async_channel::Receiver;
use async_channel::Sender;
use async_channel::unbounded;
use iroh::Endpoint;
use iroh::EndpointAddr;
use iroh::endpoint::Connection;

use crate::error::Res;
use crate::networking::ALPN;
use crate::networking::connection_manager::ConnectionManager;
use crate::networking::connection_manager::ConnectionManagerMessage;
use crate::networking::foreign_manager::ForeignManager;
use crate::networking::packet::Packet;
use crate::util::channel::send;

#[derive(Debug)]
pub struct Local {
    endpoint: Endpoint,
    connection_manager: ConnectionManager,
    packet_sender: Sender<Packet>,
    packet_receiver: Receiver<Packet>
}

impl Local {

    pub async fn establish() -> Res<Local> {
        let endpoint = Endpoint::builder()
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await?;

        let (packet_sender, packet_receiver) = unbounded();

        Ok(Local {
            endpoint: endpoint.clone(),
            connection_manager: ConnectionManager::new(endpoint, packet_sender.clone()),
            packet_sender,
            packet_receiver
        })
    }

    pub async fn connect(endpoint: Endpoint, sender: Sender<ConnectionManagerMessage>, packet_sender: Sender<Packet>, target: EndpointAddr) -> Res<usize> {
        let foreign = Foreign::establish(endpoint, target, packet_sender).await?;
        let id = foreign.stable_id;
        send(ConnectionManagerMessage::Add(foreign), &sender).await?;
        Ok(id)
    }

    pub fn ep(&self) -> Endpoint { self.endpoint.clone() }
    pub fn cs(&self) -> Sender<ConnectionManagerMessage> { self.connection_manager.yield_sender() }
    pub fn ps(&self) -> Sender<Packet> { self.packet_sender.clone() }
    pub fn yield_output(&self) -> Receiver<ConnectionManagerMessage> { self.connection_manager.yield_output() }
}

#[derive(Clone, Debug)]
pub struct Foreign {
    stable_id: usize,
    foreign_manager: Arc<ForeignManager>
}

impl Foreign {

    pub fn new(connection: Connection, packet_sender: Sender<Packet>) -> Foreign {
        Foreign {
            stable_id: connection.stable_id(),
            foreign_manager: Arc::new(ForeignManager::new(connection, packet_sender))
        }
    }
    
    pub async fn establish(endpoint: Endpoint, target: EndpointAddr, packet_sender: Sender<Packet>) -> Res<Foreign> {
        let connection = endpoint.connect(target, ALPN).await?;
        Ok(Foreign::new(connection, packet_sender))
    }

    pub fn stable_id(&self) -> usize {
        self.stable_id
    }
}
