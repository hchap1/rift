use iced::widget::Column;

use crate::{frontend::{message::Message, widget::packet_widget::PacketWidget}, networking::packet::Packet};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Chat {
    packets: Vec<(bool, Packet)>
}

impl Chat {
    pub fn view(&self, foreign: String, local: String) -> Column<'_, Message> {
        Column::from_iter(
            self.packets.iter().map(|(is_local, packet)| {
                let username = if *is_local { &local } else { &foreign };
                PacketWidget::parse(username.clone(), packet).into()
            })
        )
    }

    pub fn add_packet(&mut self, local: bool, packet: Packet) {
        self.packets.push((local, packet));
    }
}
