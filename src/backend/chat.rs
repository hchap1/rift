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
    foreign_username: Option<String>,
    packets: Vec<(bool, Packet, PacketState)>
}

impl Chat {
    pub fn new() -> Chat {
        Chat {
            foreign_username: None,
            packets: Vec::default()
        }
    }

    pub fn view(&self, local: String) -> Column<'_, Message> {
        let mut previous: Option<bool> = None;
        Column::from_iter(
            self.packets.iter().map(|(is_local, packet, state)| {
                let headerless = if let Some(previous) = previous {
                    previous == *is_local
                } else { false };
                previous = Some(*is_local);
                let username = if *is_local { &local } else { &self.foreign_username.clone().unwrap_or(String::from("FOREIGN")) };
                PacketWidget::parse(username.clone(), packet, *state, headerless).into()
            })
        ).padding(10).spacing(10)
    }

    pub fn set_foreign_username(&mut self, username: String) {
        self.foreign_username = Some(username);
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
