use std::sync::Arc;
use iced::{Background, Border, Length, Shadow, Task, widget::{Column, Container, Row, Scrollable, button, text, text_input}};
use crate::{error::ChatError, frontend::{message::{Global, Message}, pages::{Pages, add_chat_page::AddChatPage, chat_page::{ChatMessage, ChatPage}}, widget::Colour}, networking::{connection_manager::ConnectionManagerMessage, packet::{Packet, TrackedPacket}, server::Local}, util::relay::Relay};
use crate::frontend::notification::Notification;

pub struct Application {
    networking: Option<Arc<Local>>,
    active_page: Pages,
    chat_page: Option<ChatPage>,
    add_chat_page: Option<Box<dyn Page>>,
    notification_stack: Vec<Notification>,
    active_chats: Vec<(usize, String, usize)>,
    username_input: String,
    username: Option<String>
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
            chat_page: Some(ChatPage::default()),
            add_chat_page: Some(Box::new(AddChatPage::default())),
            notification_stack: vec![],
            active_chats: vec![],
            username_input: String::new(),
            username: None
        }
    }
}

impl Page for Application {
    fn view(&self) -> Container<'_, Message> {

        let contents = match &self.active_page {
            Pages::Chat(_) => if let Some(page) = self.chat_page.as_ref() { page.view() } else { Container::new(text("Error")) }
            Pages::AddChat => if let Some(page) = self.add_chat_page.as_ref() { page.view() } else { Container::new(text("Error")) }
        };

        Container::new(
            Row::new()
                .push(
                    Column::new().spacing(10).padding(10)
                        .push(
                            Row::new().spacing(5).padding(10)
                                .push(
                                    text_input(
                                        match self.username.as_ref() {
                                            Some(username) => username,
                                            None => "Username..."
                                        },
                                        &self.username_input
                                    ).on_input(|new_username| Global::UsernameInput(new_username).into())
                                    .on_submit(Global::NewUsername.into())
                                    .style(|_,_| iced::widget::text_input::Style {
                                        background: Background::Color(Colour::foreground()),
                                        border: Border::default().rounded(10),
                                        icon: Colour::accent(),
                                        placeholder: Colour::loading(),
                                        value: Colour::text(),
                                        selection: Colour::accent()
                                    })
                                ).push(
                                    button(text("SET"))
                                        .on_press(Global::NewUsername.into())
                                        .style(
                                            |_, status|
                                            iced::widget::button::Style {
                                                background: Some(Background::Color(match status {
                                                    button::Status::Active => Colour::accent(),
                                                    button::Status::Hovered => Colour::foreground(),
                                                    _ => Colour::background()
                                                })),
                                                text_color: Colour::text(),
                                                border: Border::default().rounded(10),
                                                shadow: Shadow::default(),
                                                snap: false
                                            }
                                        ).width(Length::Fill)

                                )
                        ).push(
                            button("ADD CHAT").on_press_with(|| Global::SwitchTo(Pages::AddChat).into())
                                .style(
                                    |_, status|
                                    iced::widget::button::Style {
                                        background: Some(Background::Color(match status {
                                            button::Status::Active => Colour::accent(),
                                            button::Status::Hovered => Colour::foreground(),
                                            _ => Colour::background()
                                        })),
                                        text_color: Colour::text(),
                                        border: Border::default().rounded(10),
                                        shadow: Shadow::default(),
                                        snap: false
                                    }
                                ).width(Length::Fill)
                        ).push(
                            Container::new(
                                Scrollable::new(Column::from_iter(self.active_chats.iter().map(
                                    |(id, chat, notifications)| button(
                                        Row::new().spacing(10).push(text(chat).size(15))
                                            .push(match notifications {
                                                0 => None,
                                                other => Some(
                                                    Container::new(text(other).size(20f32).color(Colour::text()))
                                                        .padding(5)
                                                        .style(|_|
                                                            iced::widget::container::Style {
                                                                background: Some(Background::Color(Colour::error())),
                                                                text_color: None,
                                                                border: Border::default().rounded(5),
                                                                shadow: Shadow::default(),
                                                                snap: false
                                                            }
                                                        )
                                                )
                                            })
                                        )
                                        .on_press_with(|| Global::SwitchTo(Pages::Chat(*id)).into())
                                        .style(
                                            |_, status|
                                            iced::widget::button::Style {
                                                background: Some(Background::Color(match status {
                                                    button::Status::Active => Colour::accent(),
                                                    button::Status::Hovered => Colour::foreground(),
                                                    _ => Colour::background()
                                                })),
                                                text_color: Colour::text(),
                                                border: Border::default().rounded(10),
                                                shadow: Shadow::default(),
                                                snap: false
                                            }
                                        ).into())).spacing(10)
                                        .push(
                                            match self.active_chats.is_empty() {
                                                true => Some(text("You don't seem to have any chats yet...").color(Colour::loading())),
                                                false => None
                                            }
                                        )
                                    )
                            ).style(|_|
                                iced::widget::container::Style {
                                    background: Some(Background::Color(Colour::foreground())),
                                    text_color: None,
                                    border: Border::default().rounded(10),
                                    shadow: Shadow::default(),
                                    snap: false
                                }
                            ).padding(10).width(Length::FillPortion(1)).height(Length::Fill)
                        )
                ).push(contents.width(Length::FillPortion(3)).height(Length::Fill))
        ).style(|_|
            iced::widget::container::Style {
                text_color: None,
                background: Some(iced::Background::Color(Colour::background())),
                border: Border::default().rounded(10),
                shadow: Shadow::default(),
                snap: false
            }
        ).height(Length::Fill).width(Length::Fill)

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
                        ConnectionManagerMessage::SuccessfulConnection(stable_id) => Some(Global::ChatConnected(stable_id).into()),
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

                Global::LoadImage(chat_stable_id, result) => {
                    let option = match result {
                        Ok(option) => option,
                        Err(e) => return Task::done(Global::Error(e).into())
                    };

                    let path = match option {
                        Some(path) => path,
                        None => return Task::done(Global::Error(ChatError::NoFileSelected.into()).into())
                    };

                    Task::done(ChatMessage::ImagePicked(chat_stable_id, path).into())
                }

                Global::ChatConnected(stable_id) => {
                    self.active_chats.push((stable_id, stable_id.to_string(), 1));
                    if let Some(chat_page) = self.chat_page.as_mut() {
                        chat_page.make_empty(stable_id);
                    }
                    match self.username.as_ref() {
                        Some(username) => Task::done(Global::Send(TrackedPacket::new(stable_id, Packet::username(username.to_string())).0).into()),
                        None => Task::none()
                    }
                }

                Global::UsernameInput(value) => {
                    self.username_input = value;
                    Task::none()
                }

                Global::NewUsername => {
                    if self.username_input.is_empty() { return Task::none(); }
                    let new_username = std::mem::take(&mut self.username_input);
                    self.username = Some(new_username.clone());
                    Task::done(ChatMessage::UsernameUpdate(new_username).into())
                }

                Global::BindUsernameToId(stable_id, username) => {
                    for chat in &mut self.active_chats {
                        if chat.0 == stable_id {
                            chat.1 = username.clone();
                        }
                    }

                    Task::none()
                }

                Global::AddNotification(id) => {
                    for chat in &mut self.active_chats {
                        if chat.0 == id {
                            chat.2 += 1;
                        }
                    }
                    Task::none()
                }

                Global::ClearNotifications(id) => {
                    for chat in &mut self.active_chats {
                        if chat.0 == id {
                            chat.2 = 0;
                        }
                    }
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

            Message::ChatMessage(msg) => {
                match self.chat_page.as_mut() {
                    Some(page) => page.update(Message::ChatMessage(msg)),
                    None => Task::none()
                }
            }
        }
    }
}
