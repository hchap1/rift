use iced::widget::Column;
use iced::widget::Container;
use iced::widget::text;
use iced::Length;

use crate::backend::chat::PacketState;
use crate::frontend::widget::Colour;
use crate::{frontend::message::Message, networking::packet::{Packet, PacketType}};

pub struct PacketWidget;
impl PacketWidget {
    pub fn parse(author: String, packet: &Packet, packet_state: PacketState, headerless: bool) -> Container<'_, Message> {
        let content_widget = match packet.kind {
            PacketType::Message => {
               Container::new(text(String::from_utf8_lossy(&packet.data))
                   .size(15)
                    .color(match packet_state {
                        PacketState::Unknown => Colour::loading(),
                        PacketState::Failed => Colour::error(),
                        PacketState::Verified => Colour::text()
                    })
               )
            },
            PacketType::Image => {
                Container::new(packet.decoded_image.as_ref().map(iced::widget::image))
                    .height(Length::Fixed(512f32))
            },
            PacketType::Username => Container::new(text(format!("Username Update: {}", String::from_utf8_lossy(&packet.data))))
        };

        Container::new(
            Column::new()
                .push(if headerless { None } else { Some(text(author).color(Colour::accent()).size(20)) })
                .push(content_widget)
        )
    }
}
