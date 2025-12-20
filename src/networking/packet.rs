use crate::{error::{Error, Res}, networking::error::NetworkError};

#[derive(Debug, Clone, Copy)]
pub enum PacketType {
    Username,
    Message
}

impl PacketType {
    pub fn from_byte(byte: u8) -> Res<PacketType> {
        Ok(match byte {
            0 => PacketType::Username,
            1 => PacketType::Message,
            _ => return Err(NetworkError::InvalidPacket.into())
        })
    }

    pub fn to_byte(self) -> u8 {
        match self {
            PacketType::Username => 0,
            PacketType::Message => 1
        }
    }
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub kind: PacketType,
    pub code: u32,
    pub data: Vec<u8>
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
        let data = iterator.collect();

        Ok(Packet {
            kind,
            code,
            data
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
}
