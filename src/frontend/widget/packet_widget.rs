use iced::Border;
use iced::Length;
use iced::Shadow;
use iced::widget::Column;
use iced::widget::Container;
use iced::widget::container;
use iced::widget::text;

use crate::backend::chat::PacketState;
use crate::frontend::widget::Colour;
use crate::{frontend::message::Message, networking::packet::{Packet, PacketType}};

pub struct PacketWidget;
impl PacketWidget {
    pub fn parse(author: String, packet: &Packet, packet_state: PacketState) -> Container<'_, Message> {
        let text_widget = match packet.kind {
            PacketType::Message => {
               text(String::from_utf8_lossy(&packet.data))
                    .color(match packet_state {
                        PacketState::Unknown => Colour::loading(),
                        PacketState::Failed => Colour::error(),
                        PacketState::Verified => Colour::text()
                    })
            }
            PacketType::Username => text(format!("Username Update: {}", String::from_utf8_lossy(&packet.data)))
        };

        Container::new(
            Column::new()
                .push(text(author).color(Colour::accent()).size(20))
                .push(text_widget.size(15))
        ).style(move |_| 
            container::Style {
                text_color: None,
                background: Some(iced::Background::Color(Colour::foreground())),
                border: Border::default().rounded(10),
                shadow: Shadow::default(),
                snap: false
            }
        ).width(Length::Fill).padding(10)
    }
}
