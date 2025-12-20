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

    pub fn connect_task(&self, target: EndpointAddr) -> impl std::future::Future<Output = Res<()>> {
        let endpoint = self.endpoint.clone();
        let sender = self.connection_manager.yield_sender();
        let packet_sender = self.packet_sender.clone();

        async move {
            let foreign = Foreign::establish(endpoint, target, packet_sender).await?;
            send(ConnectionManagerMessage::Add(foreign), &sender).await?;
            Ok(())
        }
    }
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
