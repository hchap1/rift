use iroh::Endpoint;

use crate::error::Res;
use crate::error::Error;
use crate::networking::ALPN;

pub struct Server {
    
}

impl Server {

    pub async fn establish() -> Res<Server> {
        let endpoint = Endpoint::builder()
            .alpns(vec![ALPN.to_vec()])
            .bind();

        Ok(Server {})
    }
}
