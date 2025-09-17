use bincode::{Encode, Decode};
use glam::I64Vec2;
use serde::{Deserialize, Serialize};

use crate::engine::server::common::{BlockArray, BlockType, LayerType};

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

#[derive(Serialize, Deserialize, Encode, Decode, Debug)]
pub struct ChunkMesh {
    pub foreground: BlockArray,
    pub middleground: BlockArray,
    pub background: BlockArray,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug)]
pub struct BlockChange {
    pub layer: LayerType,
    pub block_type: BlockType,
    pub block_id: u16,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug)]
pub enum ServerPacket {
    Ping,
    Message(String),
    BlockChange(((i64, i64), BlockChange)),
    ChunkMesh(((i32, i32), Box<ChunkMesh>)),
}