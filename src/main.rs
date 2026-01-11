#![allow(clippy::unit_arg)]

mod frontend;
mod backend;
mod networking;
mod error;
mod util;

use crate::frontend::application::Page;
use crate::frontend::application::Application;

fn main() -> iced::Result {
    iced::run(Application::update, Application::view)
}
