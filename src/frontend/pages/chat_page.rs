use std::{collections::HashMap, mem::take};

use iced::{Task, widget::{Column, Container, Scrollable, text_input}};

use crate::{backend::chat::Chat, error::{ChatError, Res}, frontend::{application::Page, message::{Global, Message}, notification::Notification}, networking::packet::Packet};

#[derive(Debug, Clone)]
pub enum ChatMessage {
    SetActiveChat(usize),
    ReceiveForeignPacket(usize, Packet),
    SentLocalPacket(usize, Packet),

    // Update the message box (paste, type)
    UpdateMessageBox(String),

    // Send the current message box contents to the current chat
    Send
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
            None => Err(ChatError::NoChatOpen.into())
        }
    }
}

impl Page for ChatPage {
    fn view(&self) -> Container<'_, Message> {
        Container::new(
            Column::new()
                .push(
                    Scrollable::new(
                        match self.chats.get(&self.active_chat) {
                            Some(chat) => chat.view(String::from("FOREIGN"), String::from("LOCAL")),
                            None => Column::new()
                        }
                    )
                ).push(
                    text_input(&format!("Message {}", self.active_chat), &self.message_box)
                        .on_input_maybe(Some(|new_value| ChatMessage::UpdateMessageBox(new_value).into()))
                        .on_submit(ChatMessage::Send.into())
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
                    Ok(value) => value.into(),
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
                    Task::done(Global::Send(self.active_chat, packet).into())
                }
            },
            _ => Task::none()
        }
    }
}
