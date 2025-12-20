use std::sync::Arc;
use iced::{Task, widget::{Container, text}};
use crate::{frontend::{message::{Global, Message}, pages::Pages}, networking::server::Local};
use crate::frontend::notification::Notification;

pub struct Application {
    networking: Option<Local>,
    active_page: Pages,
    chat_page: Option<Box<dyn Page>>,
    add_chat_page: Option<Box<dyn Page>>,
    browse_chats_page: Option<Box<dyn Page>>,
    notification_stack: Vec<Notification>,
}

pub trait Page {
    fn update(&mut self, message: Message) -> Task<Message>;
    fn view(&self) -> Container<'_, Message>;
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
                    Some(local) => {
                        let future = local.connect_task(node_id.into());
                        Task::future(future).map(|res| match res {
                            Ok(_) => Global::Notify(Notification::success(String::from("Connection made!"))).into(),
                            Err(error) => Global::Notify(error.into()).into()
                        })
                    }
                    None => Task::done(Global::Notify(Notification::error(String::from("Networking not initialised."))).into())
                }

                Global::None => Task::none(),
                _ => Task::none()
            }

            Message::AddChatMessage(msg) => {
                match self.add_chat_page.as_mut() {
                    Some(page) => page.update(Message::AddChatMessage(msg)),
                    None => Task::none()
                }
            }
        }
    }
}
