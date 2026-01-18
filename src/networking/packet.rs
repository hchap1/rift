use std::io::Cursor;
use iced::widget::image::Handle;

use async_channel::{Receiver, Sender, unbounded};
use image::DynamicImage;
use rand::{Rng, rng};

use crate::{error::Res, networking::error::NetworkError, util::channel::send};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    Username,
    Message,
    Image
}

impl PacketType {
    pub fn from_byte(byte: u8) -> Res<PacketType> {
        Ok(match byte {
            0 => PacketType::Username,
            1 => PacketType::Message,
            2 => PacketType::Image,
            _ => return Err(NetworkError::InvalidPacket.into())
        })
    }

    pub fn to_byte(self) -> u8 {
        match self {
            PacketType::Username => 0,
            PacketType::Message => 1,
            PacketType::Image => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub kind: PacketType,
    pub code: u32,
    pub data: Vec<u8>,
    pub decoded_image: Option<Handle>
}

impl Packet {
    pub fn from_bytes(bytes: Vec<u8>) -> Res<Packet> {
        let mut iterator = bytes.into_iter();
        let kind_byte = iterator.next().ok_or(NetworkError::InvalidPacket)?;
        let kind = PacketType::from_byte(kind_byte)?;

        let endians = [
            iterator.next().ok_or(NetworkError::InvalidPacket)?,
            iterator.next().ok_or(NetworkError::InvalidPacket)?,
            iterator.next().ok_or(NetworkError::InvalidPacket)?,
            iterator.next().ok_or(NetworkError::InvalidPacket)?
        ];

        let code = u32::from_be_bytes(endians);
        let data: Vec<u8> = iterator.collect();

        let decoded_image = if let PacketType::Image = kind {
            Some(Handle::from_bytes(data.clone()))
        } else { None };

        Ok(Packet {
            kind,
            code,
            data,
            decoded_image
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_bytes(self) -> Vec<u8> {
        let endians = self.code.to_be_bytes().to_vec();
        vec![
            vec![self.kind.to_byte()],
            endians,
            self.data
        ].into_iter().flatten().collect()
    }

    pub fn message(message: String) -> Self {

        let mut rng = rng();
        let code = rng.random_range(u32::MIN..=u32::MAX);

        Packet {
            kind: PacketType::Message,
            code,
            data: message.into_bytes(),
            decoded_image: None
        }
    }

    pub fn username(username: String) -> Self {

        let mut rng = rng();
        let code = rng.random_range(u32::MIN..=u32::MAX);

        Packet {
            kind: PacketType::Username,
            code,
            data: username.into_bytes(),
            decoded_image: None
        }
    }

    pub fn image(image: &DynamicImage) -> Res<Self> {

        let mut rng = rng();
        let code = rng.random_range(u32::MIN..=u32::MAX);
        let mut data = Vec::new();
        image.write_to(&mut Cursor::new(&mut data), image::ImageFormat::Png)?;


        Ok(Packet {
            kind: PacketType::Image,
            code,
            data: data.clone(),
            decoded_image: Some(Handle::from_bytes(data))
        })
    }
}

#[derive(Clone, Debug)]
pub enum TrackedPacketResponse {
    Confirmed,
    Failed,
}

#[derive(Clone, Debug)]
pub struct TrackedPacket {
    pub recipient_stable_id: usize,
    pub packet: Option<Packet>,
    sender: Sender<TrackedPacketResponse>
}

impl TrackedPacket {
    pub fn new(recipient_stable_id: usize, packet: Packet) -> (TrackedPacket, Receiver<TrackedPacketResponse>) {
        let (sender, receiver) = unbounded();
        (
            TrackedPacket { recipient_stable_id, packet: Some(packet), sender },
            receiver
        )
    }

    pub async fn take_packet(&mut self) -> Option<Packet> {
        self.packet.take()
    }

    pub async fn confirm_success(&self) -> Res<()> {
        send(TrackedPacketResponse::Confirmed, &self.sender).await.map_err(|x| x.into())
    }

    pub async fn indicate_failure(&self) -> Res<()> {
        send(TrackedPacketResponse::Failed, &self.sender).await.map_err(|x| x.into())
    }
}
