use std::collections::HashMap;

use iced::{Task, widget::{Column, Container, Scrollable}};

use crate::{backend::chat::Chat, error::{ChatError, Res}, frontend::{application::Page, message::{Global, Message}, notification::Notification}, networking::packet::Packet};

#[derive(Debug, Clone)]
pub enum ChatMessage {
    ReceiveForeignPacket(Packet),
    SentLocalPacket(Packet)
}

#[derive(Default)]
pub struct ChatPage {
    active_chat: usize,
    chats: HashMap<usize, Chat>
}

impl ChatPage {
    fn add_packet(&mut self, local: bool, packet: Packet) -> Res<()> {
        match self.chats.get_mut(&self.active_chat) {
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
                ChatMessage::ReceiveForeignPacket(packet) => match self.add_packet(false, packet) {
                    Ok(value) => value.into(),
                    Err(error) => Task::done(Global::Notify(error.into()).into())
                },
                ChatMessage::SentLocalPacket(packet) => match self.add_packet(true, packet) {
                    Ok(value) => value.into(),
                    Err(error) => Task::done(Global::Notify(error.into()).into())
                }
            },
            _ => Task::none()
        }
    }
}
