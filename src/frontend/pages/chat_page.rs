use std::{collections::HashMap, mem::take, path::PathBuf};
use iced::{Background, Border, Length, Shadow, Task, widget::{Column, Container, Row, Scrollable, button, scrollable::{AutoScroll, Rail, Scroller}, text, text_input}};

use crate::{backend::chat::Chat, error::{Error, Res}, frontend::{application::Page, message::{Global, Message}, widget::Colour}, networking::packet::{Packet, PacketType, TrackedPacket, TrackedPacketResponse}};

#[derive(Debug, Clone)]
pub enum ChatMessage {
    SetActiveChat(usize),
    ReceiveForeignPacket(usize, Packet),
    SentLocalPacket(usize, Packet),
    UsernameUpdate(String),

    // Update the message box (paste, type)
    UpdateMessageBox(String),

    // Send the current message box contents to the current chat
    Send,

    // Indicators on packet state
    PacketConfirmed(usize, usize),
    PacketFailed(usize, usize),

    // Pick image
    PickImage,
    ImagePicked(usize, PathBuf)
}

#[derive(Default)]
pub struct ChatPage {
    active_chat: usize,
    chats: HashMap<usize, Chat>,
    message_box: String,
    username: String
}

impl ChatPage {
    /// Function to record a packet exchange into the GUI.
    fn add_packet(&mut self, foreign_stable_id: usize, local: bool, packet: Packet) -> Res<()> {
        match self.chats.get_mut(&foreign_stable_id) {
            Some(chat) => Ok(chat.add_packet(local, packet)),
            None => {
                let mut chat = Chat::new();
                chat.add_packet(local, packet);
                self.chats.insert(foreign_stable_id, chat);
                Ok(())
            }
        }
    }
}

impl Page for ChatPage {
    fn view(&self) -> Container<'_, Message> {
        Container::new(
            Column::new().height(Length::Fill).padding(10)
                .push(
                    Scrollable::new(
                        match self.chats.get(&self.active_chat) {
                            Some(chat) => chat.view(match self.username.is_empty() {
                                true => String::from("LOCAL"),
                                false => self.username.clone()
                            }),
                            None => Column::new()
                        }
                    )
                    .spacing(10)
                    .anchor_bottom()
                    .height(Length::FillPortion(10)).width(Length::FillPortion(1))
                    .style(|_, _|
                        iced::widget::scrollable::Style {
                            container: iced::widget::container::Style::default(),
                            vertical_rail: Rail {
                                background: Some(Background::Color(Colour::foreground())),
                                border: Border::default().rounded(10),
                                scroller: Scroller {
                                    background: Background::Color(Colour::accent()),
                                    border: Border::default().rounded(10)
                                }
                            },
                            horizontal_rail: Rail {
                                background: Some(Background::Color(Colour::foreground())),
                                border: Border::default().rounded(10),
                                scroller: Scroller {
                                    background: Background::Color(Colour::accent()),
                                    border: Border::default().rounded(10)
                                }
                            },
                            gap: None,
                            auto_scroll: AutoScroll {
                                background: Background::Color(Colour::accent()),
                                border: Border::default().rounded(10),
                                shadow: Shadow::default(),
                                icon: Colour::text()
                            }
                        }
                    )
                ).push(
                    Row::new().spacing(20)
                        .push(
                            text_input(&format!("Message {}", self.active_chat), &self.message_box)
                                .on_input_maybe(Some(|new_value| ChatMessage::UpdateMessageBox(new_value).into()))
                                .on_submit(ChatMessage::Send.into())
                                .size(20)
                                .style(|_,_| iced::widget::text_input::Style {
                                    background: Background::Color(Colour::foreground()),
                                    border: Border::default().rounded(10),
                                    icon: Colour::accent(),
                                    placeholder: Colour::loading(),
                                    value: Colour::text(),
                                    selection: Colour::accent()
                                })
                        ).push(
                            button(text!("IMAGE").size(15))
                                .on_press_with(|| ChatMessage::PickImage.into())
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
                                )
                        )
                )
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {

            Message::ChatMessage(message) => match message {

                // Set the active chat asynchronously
                ChatMessage::SetActiveChat(stable_id) => {
                    self.active_chat = stable_id;
                    Task::none()
                }

                // Message to record an incoming message. This is the only interface through which the user can see a message.
                ChatMessage::ReceiveForeignPacket(author, packet) => {

                    match packet.kind {
                        PacketType::Username => {
                            let foreign_username = String::from_utf8_lossy(&packet.data).to_string();
                            if let Some(chat) = self.chats.get_mut(&author) {
                                chat.set_foreign_username(foreign_username.clone());
                            }
                            Task::done(Global::BindUsernameToId(author, foreign_username).into())
                        },
                        
                        _ => {

                            let task = match self.add_packet(author, false, packet) {
                                Ok(()) => println!("Received packet into the GUI!").into(),
                                Err(error) => Task::done(Global::Notify(error.into()).into())
                            };

                            let other = if author != self.active_chat {
                                Task::done(Global::AddNotification(author).into())
                            } else {
                                Task::none()
                            };

                            Task::batch(vec![task, other])

                        }
                    }

                },

                // Identical to ReceiveForeignPacket but idiomatically this is where packets locally initiated are sent to be displayed.
                ChatMessage::SentLocalPacket(recipient, packet) => match self.add_packet(recipient, true, packet) {
                    Ok(value) => value.into(),
                    Err(error) => Task::done(Global::Notify(error.into()).into())
                },

                // Update the message box
                ChatMessage::UpdateMessageBox(new_value) => (self.message_box = new_value).into(),

                // Send the current contents of the message box to the current chat
                ChatMessage::Send => {
                    if self.message_box.is_empty() { return Task::none(); }
                    let message = take(&mut self.message_box);
                    let packet = Packet::message(message);
                    let (tracked_packet, receiver) = TrackedPacket::new(self.active_chat, packet.clone());
                    let unique_packet_id = match self.chats.get(&self.active_chat) {
                        Some(chat) => chat.get_unique_id(),
                        None => 0
                    };

                    let active_chat_clone = self.active_chat;

                    Task::batch(vec![
                        Task::done(Global::ClearNotifications(self.active_chat).into()),
                        Task::done(Global::Send(tracked_packet).into()),
                        Task::done(ChatMessage::SentLocalPacket(self.active_chat, packet).into()),
                        Task::future(async move { receiver.recv().await }).map(move |message| match message {
                            Ok(response) => match response {
                                TrackedPacketResponse::Confirmed => ChatMessage::PacketConfirmed(active_chat_clone, unique_packet_id),
                                TrackedPacketResponse::Failed => ChatMessage::PacketFailed(active_chat_clone, unique_packet_id)
                            }

                            Err(_) => ChatMessage::PacketFailed(active_chat_clone, unique_packet_id)
                        }.into())
                    ])
                },

                // Handle a failed message
                ChatMessage::PacketFailed(stable_id, unique_id) => {
                    if let Some(chat) = self.chats.get_mut(&stable_id) {
                        chat.update_state(unique_id, crate::backend::chat::PacketState::Failed);
                    }

                    Task::none()
                }

                // Handle a successful packet that received a confirmation code from the foreign client
                ChatMessage::PacketConfirmed(stable_id, unique_id) => {
                    if let Some(chat) = self.chats.get_mut(&stable_id) {
                        chat.update_state(unique_id, crate::backend::chat::PacketState::Verified);
                    }

                    Task::none()
                },

                ChatMessage::PickImage => {
                    let active_chat = self.active_chat;
                    Task::perform(tokio::task::spawn_blocking(|| rfd::FileDialog::new().pick_file()), move |res| Global::LoadImage(active_chat, res.map_err(Error::from)).into())
                }

                ChatMessage::ImagePicked(stable_id_of_recipient, path) => {
                    
                    let image = match image::open(path) {
                        Ok(image) => image,
                        Err(e) => return Task::done(Global::Error(e.into()).into())
                    };

                    let packet = match Packet::image(&image) {
                        Ok(packet) => packet,
                        Err(e) => return Task::done(Global::Error(e.into()).into())
                    };

                    let (tracked_packet, receiver) = TrackedPacket::new(self.active_chat, packet.clone());
                    let unique_packet_id = match self.chats.get(&self.active_chat) {
                        Some(chat) => chat.get_unique_id(),
                        None => 0
                    };

                    Task::batch(vec![
                        Task::done(Global::Send(tracked_packet).into()),
                        Task::done(ChatMessage::SentLocalPacket(stable_id_of_recipient, packet).into()),
                        Task::future(async move { receiver.recv().await }).map(move |message| match message {
                            Ok(response) => match response {
                                TrackedPacketResponse::Confirmed => ChatMessage::PacketConfirmed(stable_id_of_recipient, unique_packet_id),
                                TrackedPacketResponse::Failed => ChatMessage::PacketFailed(stable_id_of_recipient, unique_packet_id)
                            }

                            Err(_) => ChatMessage::PacketFailed(stable_id_of_recipient, unique_packet_id)
                        }.into())
                    ])
                }

                ChatMessage::UsernameUpdate(username) => {
                    self.username = username.clone();

                    let mut dispatch: Vec<Task<Message>> = vec![];
                    let packet = Packet::username(username);

                    for chat in self.chats.keys() {
                        let (tracked_packet, _) = TrackedPacket::new(*chat, packet.clone());
                        dispatch.push(Task::done(Global::Send(tracked_packet).into()));
                    }

                    Task::batch(dispatch)
                }
            },
            _ => Task::none()
        }
    }
}
