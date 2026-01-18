use std::{path::PathBuf, sync::Arc};

use iroh::EndpointId;

use crate::{error::{Error, Res}, frontend::{notification::Notification, pages::{Pages, add_chat_page::AddChatMessage, chat_page::ChatMessage}}, networking::{packet::{Packet, TrackedPacket}, server::Local}};

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
        SwitchTo(Pages),
        Notify(Notification),
        None,

        // Load
        LoadNetworking,
        LoadSuccess(Arc<Local>),
        LoadImage(usize, Res<Option<PathBuf>>),

        // Interface with backend
        Send(TrackedPacket),                       // Send a packet to the given stable_id, requires a Connection to the foreign node to exist already.
        Packet(usize, Packet),                     // When a new packet is received, this is the first message prior to it being relayed to page specific needs.
        Connect(EndpointId),
        ChatConnected(usize),
        NewUsername,
        
        // Frontend
        UsernameInput(String),
        BindUsernameToId(usize, String),
        AddNotification(usize),
        ClearNotifications(usize),
}

message_enum! {
    pub enum Message {
        Global,
        AddChatMessage,
        ChatMessage
    }
}
