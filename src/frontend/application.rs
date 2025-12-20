use std::sync::Arc;
use iced::{Task, widget::{Container, text}};
use crate::{error::Error, frontend::{message::{Global, Message}, pages::Pages}, networking::server::Local};

pub struct Application {
    networking: Option<Local>,
    active_page: Pages,
    chat_page: Option<Box<dyn Page>>,
    add_chat_page: Option<Box<dyn Page>>,
    browse_chats_page: Option<Box<dyn Page>>,
    notification_stack: Vec<Notification>,
    all_notifications: Vec<Notification>
}

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

pub trait Page {
    fn update(&mut self, message: Message) -> Task<Message>;
    fn view(&self) -> Container<'_, Message>;
    fn poll_notifications(&mut self) -> Vec<Notification>;
}

impl Application {
    pub fn new() -> Application {
        Application {
            networking: None,
            active_page: Pages::BrowseChats,
            chat_page: None,
            add_chat_page: None,
            browse_chats_page: None,
            notification_stack: vec![],
            all_notifications: vec![]
        }
    }
}

impl Page for Application {
    fn view(&self) -> Container<'_, Message> {

        let active_page = match &self.active_page {
            Pages::Chat => &self.chat_page,
            Pages::AddChat => &self.add_chat_page,
            Pages::BrowseChats => &self.browse_chats_page
        };

        if let Some(page) = active_page {
            page.view()
        } else {
            Container::new(
                text("No active page.")
            )
        }

    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Global(global) => match global {
                Global::LoadNetworking => Task::future(Local::establish())
                    .map(|res| {
                        match res {
                            Ok(local) => Global::LoadSuccess(Arc::new(local)),
                            Err(e) => Global::Error(e)
                         }.into()
                    }),

                Global::Error(error) => {
                    eprintln!("Error: {error:?}");
                    Task::none()
                }

                Global::SwitchTo(page) => {
                    self.active_page = page;
                    Task::none()
                }

                Global::Connect(node_id) => match self.networking.as_ref() {
                    Some(local) => Task::future(local.connect_task(node_id.into())).map(|res| match res {
                        Ok(_) => Global::Notify(Notification::success(String::from("Connection made!"))).into(),
                        Err(error) => Global::Notify(error.into()).into()
                    }),
                    None => Task::done(Global::Notify(Notification::error(String::from("Networking not initialised."))).into())
                }

                Global::None => Task::none(),
                _ => Task::none()
            }
        }
    }

    fn poll_notifications(&mut self) -> Vec<Notification> {
        vec![]
    }
}
