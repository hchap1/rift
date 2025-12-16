use iroh::Endpoint;
use iroh::EndpointAddr;
use iroh::endpoint::Connection;

use crate::error::Res;
use crate::networking::ALPN;
use crate::networking::connection_manager::ConnectionManager;
use crate::networking::connection_manager::ConnectionManagerMessage;
use crate::util::channel::send;

pub struct Local {
    endpoint: Endpoint,
    connection_manager: ConnectionManager
}

impl Local {

    pub async fn establish() -> Res<Local> {
        let endpoint = Endpoint::builder()
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await?;

        Ok(Local {
            endpoint: endpoint.clone(),
            connection_manager: ConnectionManager::new(endpoint)
        })
    }

    pub fn connect_task(&self, target: EndpointAddr) -> impl std::future::Future<Output = Res<()>> {
        let endpoint = self.endpoint.clone();
        let sender = self.connection_manager.yield_sender();
        async move {
            let foreign = Foreign::establish(endpoint, target).await?;
            send(ConnectionManagerMessage::Add(foreign), &sender).await?;
            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
pub struct Foreign {
    stable_id: usize,
    connection: Connection
}

impl Foreign {

    pub fn new(connection: Connection) -> Foreign {
        Foreign {
            stable_id: connection.stable_id(),
            connection
        }
    }
    
    pub async fn establish(endpoint: Endpoint, target: EndpointAddr) -> Res<Foreign> {
        let connection = endpoint.connect(target, ALPN).await?;
        Ok(Foreign { stable_id: connection.stable_id(), connection })
    }

    pub fn stable_id(&self) -> usize {
        self.stable_id
    }
}
