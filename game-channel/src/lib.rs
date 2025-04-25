use crate::error::ChannelError;
use bincode;
use raylib::math::Vector3;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

pub mod error;

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
pub enum Winner {
    Player,
    Enemy,
    None,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Packet {
    Player {
        pos: ChannelVector2,
        target: ChannelVector2,
    },
    Time(u8),
    Health(u8),
    GameOver(Winner),
}

pub struct Channel<T: Read + Write> {
    pub stream: T,
}

impl<T: Read + Write> Channel<T> {
    pub fn with_stream(stream: T) -> Self {
        Self { stream }
    }

    pub fn send(&mut self, packet: Packet) -> Result<(), ChannelError> {
        bincode::serialize_into(&mut self.stream, &packet).map_err(|err| err.into())
    }

    pub fn receive(&mut self) -> Result<Packet, ChannelError> {
        bincode::deserialize_from(&mut self.stream).map_err(|err| err.into())
    }
}
