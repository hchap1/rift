use iced::Border;
use iced::Length;
use iced::Shadow;
use iced::widget::image::Handle;
use iced::widget::Column;
use iced::widget::Container;
use iced::widget::container;
use iced::widget::text;
use iced::widget::image;

use crate::backend::chat::PacketState;
use crate::frontend::widget::Colour;
use crate::{frontend::message::Message, networking::packet::{Packet, PacketType}};

pub struct PacketWidget;
impl PacketWidget {
    pub fn parse(author: String, packet: &Packet, packet_state: PacketState) -> Container<'_, Message> {
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
                Container::new(image(Handle::from_bytes(packet.data.clone())))
            },
            PacketType::Username => Container::new(text(format!("Username Update: {}", String::from_utf8_lossy(&packet.data))))
        };

        Container::new(
            Column::new()
                .push(text(author).color(Colour::accent()).size(20))
                .push(content_widget)
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
