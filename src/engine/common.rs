use bincode::{Encode, Decode};
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
    pub x: i32,
    pub y: i32,
    pub foreground: BlockArray,
    pub middleground: BlockArray,
    pub background: BlockArray,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug)]
pub struct BlockChange {
    pub x: i64,
    pub y: i64,
    pub layer: LayerType,
    pub block_type: BlockType,
    pub block_id: u16,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug)]
pub enum ServerPacket {
    Ping,
    Message(String),
    BlockChange(BlockChange),
    ChunkMesh(Box<ChunkMesh>),
}