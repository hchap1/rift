use std::str::FromStr;

use iced::{Background, Border, Task, widget::{Column, Container, button, text_input}};
use iroh::EndpointId;

use crate::frontend::{application::Page, message::{Global, Message}, notification::Notification, widget::Colour};

#[derive(Default)]
pub struct AddChatPage {
    input: String
}

#[derive(Clone, Debug)]
pub enum AddChatMessage {
    Input(String),
    Submit
}

impl Page for AddChatPage {
    fn view(&self) -> Container<'_, Message> {
        Container::new(
            Column::new()
                .push(
                    text_input("Enter NODE ID.", &self.input)
                        .on_submit(AddChatMessage::Submit.into())
                        .on_input_maybe(Some(|new_content| AddChatMessage::Input(new_content).into()))
                        .style(|_,_| iced::widget::text_input::Style {
                            background: Background::Color(Colour::foreground()),
                            border: Border::default().rounded(10),
                            icon: Colour::accent(),
                            placeholder: Colour::loading(),
                            value: Colour::text(),
                            selection: Colour::accent()
                        })
                )
                .push(
                    button("Connect")
                        .on_press(AddChatMessage::Submit.into())
                )
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddChatMessage(message) => match message {
                AddChatMessage::Input(new_content) => {
                    self.input = new_content;
                    Task::none()
                }
                AddChatMessage::Submit => {
                    match EndpointId::from_str(&std::mem::take(&mut self.input)) {
                        Ok(valid_id) => Task::done(Global::Connect(valid_id).into())
                            .chain(Task::done(Global::SwitchTo(super::Pages::BrowseChats).into())),
                        Err(_) => Task::done(Global::Notify(Notification::error(String::from("Invalid ID"))).into())
                    }
                }
            },
            _ => Task::none()
        }
    }
}
