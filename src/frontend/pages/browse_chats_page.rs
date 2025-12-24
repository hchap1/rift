use iced::Task;
use iced::widget::{Column, Container, Scrollable};
use iced::widget::button;
use iced::widget::text;

use crate::frontend::notification::Notification;
use crate::frontend::{application::Page, message::Message};
use crate::frontend::message::Global;

#[derive(Debug, Clone)]
pub enum BrowseChatsMessage {
    ChatConnected(usize),
    ChatDisconnect(usize)
}

pub struct BrowseChatsPage {
    chats: Vec<usize>,
}

impl Page for BrowseChatsPage {
    fn view(&self) -> Container<'_, Message> {
        Container::new(Scrollable::new(
            Column::from_iter(self.chats.iter()
                .map(|id|
                    button(text(id))
                        .on_press(Global::SwitchTo(super::Pages::Chat).into())
                .into())
            )
        ))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::BrowseChatsMessage(message) => match message {
                BrowseChatsMessage::ChatConnected(id) => {
                    self.chats.push(id);
                    Task::done(Global::Notify(Notification::success(String::from("Connection made!"))).into())
                }
                BrowseChatsMessage::ChatDisconnect(id) => self.chats.retain(|x| *x != id).into()
            }
            _ => Task::none()
        }
    }
}
