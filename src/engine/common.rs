use serde::{Deserialize, Serialize};

use crate::engine::server::common::BlockArray;

pub struct ChunkRelativePos {
    pub x: u8,
    pub y: u8,
}

impl ChunkRelativePos {
    pub fn new(x: u8, y: u8) -> ChunkRelativePos {
        return ChunkRelativePos {
            x,
            y,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkMesh {
    foreground: BlockArray,
    middleground: BlockArray,
    background: BlockArray,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerPacket {
    Ping,
    ChunkMesh(ChunkMesh)
}