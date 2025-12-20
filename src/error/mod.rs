use std::sync::Arc;
use async_channel::{RecvError, SendError, TryRecvError};
use iroh::endpoint::{BindError, ClosedStream, ConnectError, ConnectingError, ConnectionError, WriteError};

use crate::networking::error::NetworkError;

pub type Res<T> = Result<T, Error>;

macro_rules! error_enum {
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
                $variant(Arc<$variant>),
            )*
        }

        $(
            impl From<$variant> for $name {
                fn from(e: $variant) -> Self {
                    $name::$variant(Arc::new(e))
                }
            }
        )*
    };
}

#[derive(Debug, Clone)]
pub enum ChannelError {
    ChannelDead,
    ChannelEmpty
}

impl<T> From<SendError<T>> for ChannelError {
    fn from(_: SendError<T>) -> ChannelError {
        ChannelError::ChannelDead
    }
}

impl From<RecvError> for ChannelError {
    fn from(_: RecvError) -> ChannelError {
        ChannelError::ChannelDead
    }
}

impl From<TryRecvError> for ChannelError {
    fn from(error: TryRecvError) -> ChannelError {
        match error {
            TryRecvError::Empty => ChannelError::ChannelEmpty,
            _ => ChannelError::ChannelDead
        }
    }
}

error_enum! {
    pub enum Error {
        BindError,
        ConnectError,
        ConnectingError,
        ChannelError,
        ConnectionError,
        WriteError,
        ClosedStream,
        NetworkError
    }
}
