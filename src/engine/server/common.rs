use bincode::{Encode, Decode};
use serde::{Deserialize, Serialize};

use crate::engine::{common::ChunkRelativePos, server::constants::{CHUNK_BLOCK_COUNT, CHUNK_SIZE}};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, Encode, Decode)]
pub enum BlockType {
    Air = 0,
    Tile = 1,
    Wall = 2,
    Sprite = 3,
    TileEntity = 4,
}

/*  
/   The texture_index is used for connections in the tile and wall type,
/   different textures for each tile in the sprite type, and like
/   idk what in the tile entity type lol i gotta find some use
*/ 

#[derive(Debug, Clone, Copy)]
pub struct BlockArray {
    pub block_type: [BlockType; CHUNK_BLOCK_COUNT as usize],
    pub block_id: [u32; CHUNK_BLOCK_COUNT as usize],
    pub texture_index: [u8; CHUNK_BLOCK_COUNT as usize],
}

impl BlockArray {
    pub fn set_block_type_byindex(&mut self, index: usize, block_type: BlockType) {
        self.block_type[index] = block_type;
    }

    pub fn set_block_type(&mut self, chunk_relative_pos: ChunkRelativePos, block_type: BlockType) {
        self.block_type[(chunk_relative_pos.y * CHUNK_SIZE + chunk_relative_pos.x) as usize] = block_type;
    }

    pub fn set_block_id_byindex(&mut self, index: usize, id: u32) {
        self.block_id[index] = id;
    }

    pub fn set_block_id(&mut self, chunk_relative_pos: ChunkRelativePos, id: u32) {
        self.block_id[(chunk_relative_pos.y * CHUNK_SIZE + chunk_relative_pos.x) as usize] = id;
    }

    pub fn set_block_texture_index_byindex(&mut self, index: usize, texture_index: u8) {
        self.texture_index[index] = texture_index;
    }

    pub fn set_block_texture_index(&mut self, chunk_relative_pos: ChunkRelativePos, texture_index: u8) {
        self.texture_index[(chunk_relative_pos.y * CHUNK_SIZE + chunk_relative_pos.x) as usize] = texture_index;
    }

    pub fn clear_block_byindex(&mut self, index: usize) {
        self.block_type[index] = BlockType::Air;
        self.block_id[index] = 0;
        self.texture_index[index] = 0;
    }

    pub fn clear_block(&mut self, chunk_relative_pos: ChunkRelativePos) {
        let pos = chunk_relative_pos.y as usize * CHUNK_SIZE as usize + chunk_relative_pos.x as usize;
        self.block_type[pos as usize] = BlockType::Air;
        self.block_id[pos as usize] = 0;
        self.texture_index[pos as usize] = 0;
    }

    pub fn filled_basic_tile() -> BlockArray {
        return BlockArray { 
            block_type: [BlockType::Tile; CHUNK_BLOCK_COUNT as usize],
            block_id: ([1; CHUNK_BLOCK_COUNT as usize]),
            texture_index: ([0; CHUNK_BLOCK_COUNT as usize]),
        }
    }

    pub fn filled_basic_wall() -> BlockArray {
        return BlockArray { 
            block_type: [BlockType::Wall; CHUNK_BLOCK_COUNT as usize],
            block_id: ([1; CHUNK_BLOCK_COUNT as usize]),
            texture_index: ([0; CHUNK_BLOCK_COUNT as usize]),
        }
    }

    pub fn filled_basic_air() -> BlockArray {
        return BlockArray { 
            block_type: [BlockType::Air; CHUNK_BLOCK_COUNT as usize],
            block_id: ([0; CHUNK_BLOCK_COUNT as usize]),
            texture_index: ([0; CHUNK_BLOCK_COUNT as usize]),
        }
    }
}

// The fore and middle ground never have walls, while the background has only walls
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq)]
pub enum LayerType {
    Foreground,
    Middleground,
    Background
}