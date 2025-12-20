use crate::error::Error;

#[derive(Debug, Clone, Copy)]
pub enum NotificationType {
    Error,
    Warning,
    Info,
    Success
}

#[derive(Debug, Clone)]
pub struct Notification {
    kind: NotificationType,
    heading: String,
    body: Option<String>
}

impl Notification {
    pub fn success(heading: String) -> Notification {
        Notification {
            kind: NotificationType::Success,
            heading,
            body: None
        }
    }

    pub fn error(heading: String) -> Notification {
        Notification {
            kind: NotificationType::Error,
            heading,
            body: None
        }
    }
}

impl From<Error> for Notification {
    fn from(error: Error) -> Notification {
        Notification {
            kind: NotificationType::Error,
            heading: format!("{error:?}"),
            body: None
        }
    }
}

