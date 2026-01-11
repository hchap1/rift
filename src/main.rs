#![allow(clippy::unit_arg)]

mod frontend;
mod backend;
mod networking;
mod error;
mod util;

use iced::Task;

use crate::frontend::application::Page;
use crate::frontend::application::Application;
use crate::frontend::message::Global;
use crate::frontend::message::Message;

fn main() -> iced::Result {
    iced::application(|| (Application::default(), Task::done(Message::Global(Global::LoadNetworking))), Application::update, Application::view)
        .title("rift")
        .run()
}
