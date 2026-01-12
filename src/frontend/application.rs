use std::sync::Arc;
use iced::{Task, widget::{Column, Container, Row, button, text}};
use crate::{error::ChatError, frontend::{message::{Global, Message}, pages::{Pages, add_chat_page::AddChatPage, browse_chats_page::{BrowseChatsMessage, BrowseChatsPage}, chat_page::{ChatMessage, ChatPage}}}, networking::{connection_manager::ConnectionManagerMessage, server::Local}, util::relay::Relay};
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

impl Default for Application {
    fn default() -> Application {
        Application {
            networking: None,
            active_page: Pages::AddChat,
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

        let contents = if let Some(page) = active_page {
            page.view()
        } else {
            Container::new(
                text("No active page.")
            )
        };

        Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(button("ADD CHAT").on_press_with(|| Global::SwitchTo(Pages::AddChat).into()))
                        .push(button("BROWSE CHATS").on_press_with(|| Global::SwitchTo(Pages::BrowseChats).into()))
                ).push(contents)
        )

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

                    let task = if let Pages::Chat(stable_id) = page {
                        Task::done(ChatMessage::SetActiveChat(stable_id).into())
                    } else { Task::none() };

                    self.active_page = page;

                    task
                }

                // Initiate a connection. Upon success, backend will inform frontend of the connection.
                Global::Connect(node_id) => match self.networking.as_ref() {

                    Some(local) => {
                        // Spawn a future that will attempt to connect with a client.
                        // Must carry clones of all channels due to ownership conflicts.
                        Task::future(Local::connect(
                            local.ep(),
                            local.cs(),
                            local.ps(),
                            node_id.into()
                        )).map(|res| match res {
                            // Upon success, counterintuitively do not track the new ID. Rather, rely on the backend to process the connection and relay it back.
                            Ok(id) => Global::Notify(Notification::success(format!("Connection success! ID: {id}"))),
                            Err(error) => Global::Notify(error.into()),
                        }.into())
                    }
                    None => Task::done(Global::Notify(Notification::error(String::from("Networking not initialised."))).into())
                }

                // The networking backend was successfully established.
                Global::LoadSuccess(local) => {

                    // Establish receiver clones for backend outputs.
                    let output_receiver = local.yield_output();
                    let packet_receiver = local.yield_packet_output();
                    self.networking = Some(local);
                    
                    // Generate a relay converting new connections / errors into frontend messages.
                    // This will occur for foreign and locally initiated connections.
                    let new_connection_stream = Task::stream(Relay::consume_receiver(output_receiver, |message| match message {
                        ConnectionManagerMessage::SuccessfulConnection(stable_id) => Some(BrowseChatsMessage::ChatConnected(stable_id).into()),
                        ConnectionManagerMessage::Error(error) => Some(Global::Error(error).into()),
                        _ => Some(Message::Global(Global::None))
                    }));

                    // Generate a relay converting incoming packets into frontend messages.
                    let new_packet_stream = Task::stream(Relay::consume_receiver(packet_receiver, |(author, packet)| Some(Global::Packet(author, packet).into())));
                    Task::batch(vec![new_connection_stream, new_packet_stream])
                }

                // Originating point of incoming packets from the relay above.
                Global::Packet(author, packet) => {
                    println!("Global::Packet received packet!");
                    Task::done(ChatMessage::ReceiveForeignPacket(author, packet).into())
                }

                Global::Send(tracked_packet) => {
                    let connection_manager_sender = match self.networking.as_mut() {
                        Some(local) => local.cs(),
                        None => return Task::done(Global::Error(ChatError::NetworkingBackendFailedToInitialise.into()).into())
                    };

                    Task::future(Local::send_packet_to(connection_manager_sender, tracked_packet)).map(|res| match res {
                        Ok(_) => Global::None,
                        Err(error) => Global::Error(error)
                    }.into())
                }

                Global::Notify(notification) => {
                    println!("NOTIFICATION: {notification:?}");
                    self.notification_stack.push(notification);
                    Task::none()
                }

                Global::None => Task::none()
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
