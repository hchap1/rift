use iced::widget::Column;
use iced::widget::Container;
use iced::widget::text;

use crate::{frontend::message::Message, networking::packet::{Packet, PacketType}};

pub struct PacketWidget;
impl PacketWidget {
    pub fn parse(author: String, packet: &Packet) -> Container<'_, Message> {
        
        let text_widget = match packet.kind {
            PacketType::Message => text(String::from_utf8_lossy(&packet.data)),
            PacketType::Username => text(format!("Username Update: {}", String::from_utf8_lossy(&packet.data)))
        };

        Container::new(
            Column::new()
                .push(text(author))
                .push(text_widget)
        )
    }
}
