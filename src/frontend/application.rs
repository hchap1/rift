use std::sync::Arc;
use iced::{Task, widget::{Container, text}};
use crate::{frontend::{message::{Global, Message}, pages::{Pages, add_chat_page::AddChatPage, browse_chats_page::{BrowseChatsMessage, BrowseChatsPage}, chat_page::ChatPage}}, networking::{connection_manager::ConnectionManagerMessage, server::Local}, util::relay::Relay};
use crate::frontend::notification::Notification;

pub struct Application {
    networking: Option<Arc<Local>>,
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
            chat_page: Some(Box::new(ChatPage::default())),
            add_chat_page: Some(Box::new(AddChatPage::default())),
            browse_chats_page: Some(Box::new(BrowseChatsPage::default())),
            notification_stack: vec![],
        }
    }
}

impl Page for Application {
    fn view(&self) -> Container<'_, Message> {

        let active_page = match &self.active_page {
            Pages::Chat(_) => &self.chat_page,
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
                    Task::done(Global::Notify(error.into()).into())
                }

                Global::SwitchTo(page) => {
                    self.active_page = page;
                    Task::none()
                }

                Global::Connect(node_id) => match self.networking.as_ref() {
                    Some(local) => {
                        Task::future(Local::connect(
                            local.ep(),
                            local.cs(),
                            local.ps(),
                            node_id.into()
                        )).map(|res| match res {
                            Ok(id) => BrowseChatsMessage::ChatConnected(id).into(),
                            Err(error) => Global::Notify(error.into()).into()
                        })
                    }
                    None => Task::done(Global::Notify(Notification::error(String::from("Networking not initialised."))).into())
                }

                // The networking backend was successfully established.
                // Therefore, create a Relay mapping new connections into the frontend.
                Global::LoadSuccess(local) => {
                    let output_receiver = local.yield_output();
                    let packet_receiver = local.yield_packet_output();
                    self.networking = Some(local);
                    
                    let new_connection_stream = Task::stream(Relay::consume_receiver(output_receiver, |message| match message {
                        ConnectionManagerMessage::SuccessfulConnection(stable_id) => Some(BrowseChatsMessage::ChatConnected(stable_id).into()),
                        ConnectionManagerMessage::Error(error) => Some(Global::Error(error).into()),
                        _ => Some(Message::Global(Global::None))
                    }));

                    let new_packet_stream = Task::stream(Relay::consume_receiver(packet_receiver, |(author, packet)| Some(Global::Packet(author, packet).into())));

                    Task::batch(vec![new_connection_stream, new_packet_stream])
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

            Message::BrowseChatsMessage(msg) => {
                match self.browse_chats_page.as_mut() {
                    Some(page) => page.update(Message::BrowseChatsMessage(msg)),
                    None => Task::none()
                }
            }

            Message::ChatMessage(msg) => {
                match self.chat_page.as_mut() {
                    Some(page) => page.update(Message::ChatMessage(msg)),
                    None => Task::none()
                }
            }
        }
    }
}
