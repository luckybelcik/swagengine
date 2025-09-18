use bincode::{Encode, Decode};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::engine::server::{common::{BlockArray, BlockType, LayerType}, constants::CHUNK_BLOCK_COUNT};

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

#[repr(C)]
#[derive(Clone, Copy, Serialize, Deserialize, Encode, Decode, Debug, Zeroable, Pod)]
pub struct ChunkMesh {
    #[serde(with = "serde_arrays")]
    pub foreground: [Block; CHUNK_BLOCK_COUNT as usize],
    #[serde(with = "serde_arrays")]
    pub middleground: [Block; CHUNK_BLOCK_COUNT as usize],
    #[serde(with = "serde_arrays")]
    pub background: [Block; CHUNK_BLOCK_COUNT as usize],
}

#[repr(C)]
#[derive(Clone, Copy, Serialize, Deserialize, Encode, Decode, Debug, Zeroable, Pod)]
pub struct Block {
    pub x: u8,
    pub y: u8,
    pub block_id: u16,
    pub block_type: u8,
    pub texture_index: u8,
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