use std::collections::HashMap;

use iced::{Task, widget::{Column, Container, Scrollable}};

use crate::{backend::chat::Chat, error::{ChatError, Res}, frontend::{application::Page, message::{Global, Message}, notification::Notification}, networking::packet::Packet};

#[derive(Debug, Clone)]
pub enum ChatMessage {
    ReceiveForeignPacket(usize, Packet),
    SentLocalPacket(usize, Packet)
}

#[derive(Default)]
pub struct ChatPage {
    active_chat: usize,
    chats: HashMap<usize, Chat>
}

impl ChatPage {
    /// Function to record a packet exchange into the GUI.
    fn add_packet(&mut self, foreign_stable_id: usize, local: bool, packet: Packet) -> Res<()> {
        match self.chats.get_mut(&foreign_stable_id) {
            Some(chat) => Ok(chat.add_packet(local, packet)),
            None => Err(ChatError::NoChatOpen.into())
        }
    }
}

impl Page for ChatPage {
    fn view(&self) -> Container<'_, Message> {
        Container::new(
            Scrollable::new(
                match self.chats.get(&self.active_chat) {
                    Some(chat) => chat.view(String::from("FOREIGN"), String::from("LOCAL")),
                    None => Column::new()
                }
            )
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {

            Message::ChatMessage(message) => match message {

                // Message to record an incoming message. This is the only interface through which the user can see a message.
                ChatMessage::ReceiveForeignPacket(author, packet) => match self.add_packet(author, false, packet) {
                    Ok(value) => value.into(),
                    Err(error) => Task::done(Global::Notify(error.into()).into())
                },

                // Identical to ReceiveForeignPacket but idiomatically this is where packets locally initiated are sent to be displayed.
                ChatMessage::SentLocalPacket(recipient, packet) => match self.add_packet(recipient, true, packet) {
                    Ok(value) => value.into(),
                    Err(error) => Task::done(Global::Notify(error.into()).into())
                }
            },
            _ => Task::none()
        }
    }
}
