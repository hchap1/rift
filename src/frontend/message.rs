use std::sync::Arc;

use crate::{error::Error, networking::{packet::Packet, server::Local}};

macro_rules! message_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        $vis enum $name {
            $(
                $variant($variant),
            )*
        }

        $(
            impl From<$variant> for $name {
                fn from(e: $variant) -> Self {
                    $name::$variant(e)
                }
            }
        )*
    };
}

#[derive(Clone, Debug)]
pub enum Global {

        // Basic
        Error(Error),
        None,

        // Load
        LoadNetworking,
        LoadSuccess(Arc<Local>),

        // Interface with backend
        Send(usize, Packet),
        Packet(usize, Packet),
        Connect(usize),
}

message_enum! {
    pub enum Message {
        Global
    }
}
