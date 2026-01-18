use iced::widget::Column;
use crate::{frontend::{message::Message, widget::packet_widget::PacketWidget}, networking::packet::Packet};

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum PacketState {
    Unknown,
    Failed,
    Verified
}

#[derive(Clone, Debug)]
pub struct Chat {
    packets: Vec<(bool, Packet, PacketState)>
}

impl Chat {
    pub fn new() -> Chat {
        Chat {
            packets: Vec::default()
        }
    }

    pub fn view(&self, foreign: String, local: String) -> Column<'_, Message> {
        Column::from_iter(
            self.packets.iter().map(|(is_local, packet, state)| {
                let username = if *is_local { &local } else { &foreign };
                PacketWidget::parse(username.clone(), packet, *state).into()
            })
        ).padding(10).spacing(10)
    }

    pub fn add_packet(&mut self, local: bool, packet: Packet) {
        self.packets.push((local, packet, if local { PacketState::Unknown } else { PacketState::Verified }));
    }

    pub fn get_unique_id(&self) -> usize {
        self.packets.len()
    }

    pub fn update_state(&mut self, id: usize, state: PacketState) {
        if let Some(packet) = self.packets.get_mut(id) {
            packet.2 = state;
        }
    }
}
