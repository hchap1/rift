use std::{collections::HashMap, mem::take};

use iced::{Length, Task, widget::{Column, Container, Scrollable, text_input}};

use crate::{backend::chat::Chat, error::Res, frontend::{application::Page, message::{Global, Message}}, networking::packet::{Packet, TrackedPacket, TrackedPacketResponse}};

#[derive(Debug, Clone)]
pub enum ChatMessage {
    SetActiveChat(usize),
    ReceiveForeignPacket(usize, Packet),
    SentLocalPacket(usize, Packet),

    // Update the message box (paste, type)
    UpdateMessageBox(String),

    // Send the current message box contents to the current chat
    Send,

    // Indicators on packet state
    PacketConfirmed(usize, usize),
    PacketFailed(usize, usize)
}

#[derive(Default)]
pub struct ChatPage {
    active_chat: usize,
    chats: HashMap<usize, Chat>,
    message_box: String
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
            Column::new().height(Length::Fill)
                .push(
                    Scrollable::new(
                        match self.chats.get(&self.active_chat) {
                            Some(chat) => chat.view(String::from("FOREIGN"), String::from("LOCAL")),
                            None => Column::new()
                        }
                    ).height(Length::FillPortion(10)).width(Length::FillPortion(1))
                ).push(
                    text_input(&format!("Message {}", self.active_chat), &self.message_box)
                        .on_input_maybe(Some(|new_value| ChatMessage::UpdateMessageBox(new_value).into()))
                        .on_submit(ChatMessage::Send.into())
                        .size(32)
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
                ChatMessage::ReceiveForeignPacket(author, packet) => match self.add_packet(author, false, packet) {
                    Ok(()) => println!("Received packet into the GUI!").into(),
                    Err(error) => Task::done(Global::Notify(error.into()).into())
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
                    let message = take(&mut self.message_box);
                    let packet = Packet::message(message);
                    let (tracked_packet, receiver) = TrackedPacket::new(self.active_chat, packet.clone());
                    let unique_packet_id = match self.chats.get(&self.active_chat) {
                        Some(chat) => chat.get_unique_id(),
                        None => 0
                    };

                    let active_chat_clone = self.active_chat;

                    Task::batch(vec![
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
                }
            },
            _ => Task::none()
        }
    }
}
