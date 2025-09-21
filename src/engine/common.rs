use std::array::from_fn;

use bincode::{Encode, Decode};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::engine::server::{chunk::Chunk, common::{BlockType, LayerType}, constants::{CHUNK_BLOCK_COUNT, CHUNK_SIZE}};

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

// The order of this is reversed compared to the normal chunk to account
// for drawing order. This way, the background gets drawn first, then
// the middleground, and lastly the foreground
#[repr(C)]
#[derive(Clone, Copy, Serialize, Deserialize, Encode, Decode, Debug, Zeroable, Pod)]
pub struct ChunkMesh {
    #[serde(with = "serde_arrays")]
    pub background: [Block; CHUNK_BLOCK_COUNT as usize],
    #[serde(with = "serde_arrays")]
    pub middleground: [Block; CHUNK_BLOCK_COUNT as usize],
    #[serde(with = "serde_arrays")]
    pub foreground: [Block; CHUNK_BLOCK_COUNT as usize],
}

impl From<&PacketChunk> for ChunkMesh {
    fn from(packet: &PacketChunk) -> Self {
        ChunkMesh {
            background: convert_layer_to_aos_mesh(
                packet.background_blockid,
                packet.background_blocktype,
                packet.background_textureindex,
            ),
            middleground: convert_layer_to_aos_mesh(
                packet.middleground_blockid,
                packet.middleground_blocktype,
                packet.middleground_textureindex,
            ),
            foreground: convert_layer_to_aos_mesh(
                packet.foreground_blockid,
                packet.foreground_blocktype,
                packet.foreground_textureindex,
            ),
        }
    }
}

fn convert_layer_to_aos_mesh(
    block_ids: [u16; CHUNK_BLOCK_COUNT as usize],
    block_types: [u8; CHUNK_BLOCK_COUNT as usize],
    texture_indices: [u8; CHUNK_BLOCK_COUNT as usize],
) -> [Block; CHUNK_BLOCK_COUNT as usize] {
    core::array::from_fn(|i| {
        Block {
            x: (i % CHUNK_SIZE as usize) as u8,
            y: (i / CHUNK_SIZE as usize) as u8,
            
            block_id: block_ids[i] as u32,
            block_type: block_types[i],
            texture_index: texture_indices[i],
        }
    })
}

#[repr(C)]
#[derive(Clone, Copy, Serialize, Deserialize, Encode, Decode, Debug, Zeroable, Pod)]
pub struct PacketChunk {
    #[serde(with = "serde_arrays")]
    pub foreground_blockid: [u16; CHUNK_BLOCK_COUNT as usize],
    #[serde(with = "serde_arrays")]
    pub foreground_blocktype: [u8; CHUNK_BLOCK_COUNT as usize],
    #[serde(with = "serde_arrays")]
    pub foreground_textureindex: [u8; CHUNK_BLOCK_COUNT as usize],
    #[serde(with = "serde_arrays")]
    pub middleground_blockid: [u16; CHUNK_BLOCK_COUNT as usize],
    #[serde(with = "serde_arrays")]
    pub middleground_blocktype: [u8; CHUNK_BLOCK_COUNT as usize],
    #[serde(with = "serde_arrays")]
    pub middleground_textureindex: [u8; CHUNK_BLOCK_COUNT as usize],
    #[serde(with = "serde_arrays")]
    pub background_blockid: [u16; CHUNK_BLOCK_COUNT as usize],
    #[serde(with = "serde_arrays")]
    pub background_blocktype: [u8; CHUNK_BLOCK_COUNT as usize],
    #[serde(with = "serde_arrays")]
    pub background_textureindex: [u8; CHUNK_BLOCK_COUNT as usize],
}

impl From<&Chunk> for PacketChunk {
    fn from(chunk: &Chunk) -> Self {
        PacketChunk {
            foreground_blockid: from_fn(|i| chunk.foreground.block_id[i]),
            foreground_blocktype: from_fn(|i| chunk.foreground.block_type[i] as u8),
            foreground_textureindex: from_fn(|i| chunk.foreground.texture_index[i]),

            middleground_blockid: from_fn(|i| chunk.middleground.block_id[i]),
            middleground_blocktype: from_fn(|i| chunk.middleground.block_type[i] as u8),
            middleground_textureindex: from_fn(|i| chunk.middleground.texture_index[i]),

            background_blockid: from_fn(|i| chunk.background.block_id[i]),
            background_blocktype: from_fn(|i| chunk.background.block_type[i] as u8),
            background_textureindex: from_fn(|i| chunk.background.texture_index[i]),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Serialize, Deserialize, Encode, Decode, Debug, Zeroable, Pod)]
pub struct Block {
    pub block_id: u32,
    pub x: u8,
    pub y: u8,
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
    Chunk(((i32, i32), Box<PacketChunk>)),
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug)]
pub struct PacketHeader {
    pub is_compressed: bool,
    pub original_size: usize,
    pub data: Vec<u8>,
}