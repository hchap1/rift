use std::collections::HashMap;

use iced::{Task, widget::{Column, Container, Scrollable}};

use crate::{backend::chat::Chat, frontend::{application::Page, message::Message}, networking::packet::Packet};

#[derive(Debug, Clone)]
pub enum ChatMessage {
    ReceiveForeignPacket(Packet),
    SentLocalPacket(Packet)
}

pub struct ChatPage {
    active_chat: usize,
    chats: HashMap<usize, Chat>
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
            Message::ChatMessage(message) {

            }
        }
    }
}
