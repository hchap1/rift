use std::sync::Arc;

use iced::{Task, widget::{Container, text}};

use crate::{error::Res, frontend::message::{Global, Message}, networking::server::Local};

pub struct Application {
    networking: Option<Local>,
}

pub trait Page {
    fn update(&mut self, message: Message) -> Task<Message>;
    fn view(&self) -> Container<'_, Message>;
}

impl Application {
    pub fn new() -> Application {
        Application {
            networking: None
        }
    }
}

impl Page for Application {
    fn view(&self) -> Container<'_, Message> {
        Container::new(
            text("Cool text")
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
                    eprintln!("Error: {error:?}");
                    Task::none()
                }

                Global::None => Task::none(),
                _ => Task::none()
            }
        }
    }
}
