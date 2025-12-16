use iroh::Endpoint;
use iroh::EndpointAddr;
use iroh::endpoint::Connection;

use crate::error::Res;
use crate::error::Error;
use crate::networking::ALPN;

pub struct Local {
    endpoint: Endpoint,
    connections: Vec<Foreign>
}

impl Local {

    pub async fn establish() -> Res<Local> {
        let endpoint = Endpoint::builder()
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await?;

        Ok(Local { endpoint, connections: Vec::new() })
    }
}

pub struct Foreign {
    address: EndpointAddr,
    connection: Connection
}

impl Foreign {
    
    pub async fn establish(endpoint: Endpoint, target: EndpointAddr) -> Res<Foreign> {
        let address = target.clone();
        let connection = endpoint.connect(target, ALPN).await?;
        Ok(Foreign { address, connection })
    }
}
