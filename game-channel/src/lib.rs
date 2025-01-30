use bincode;
use raylib::math::Vector3;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct ChannelVector2 {
    pub x: f32,
    pub z: f32,
}

impl From<Vector3> for ChannelVector2 {
    fn from(value: Vector3) -> Self {
        ChannelVector2 {
            x: value.x,
            z: value.z,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Packet {
    Player {
        pos: ChannelVector2,
        target: ChannelVector2,
    },
    Time(u8),
}

#[derive(Debug)]
pub enum ChannelError {
    SendErr,
    ReceiveErr,
}

pub struct Channel<T: Read + Write> {
    pub stream: T,
}

impl<T: Read + Write> Channel<T> {
    pub fn with_stream(stream: T) -> Self {
        Self { stream }
    }

    pub fn send(&mut self, packet: Packet) -> Result<(), ChannelError> {
        bincode::serialize_into(&mut self.stream, &packet).map_err(|_| ChannelError::SendErr)
    }

    pub fn receive(&mut self) -> Result<Packet, ChannelError> {
        bincode::deserialize_from(&mut self.stream).map_err(|_| ChannelError::ReceiveErr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip_player_pos() {
        todo!()
    }
}
