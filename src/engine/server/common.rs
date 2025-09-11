use crate::engine::common::ChunkRelativePos;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BlockType {
    Air,
    Tile,
    Wall,
    Sprite,
    TileEntity,
}

/*  
/   The texture_index is used for connections in the tile and wall type,
/   different textures for each tile in the sprite type, and like
/   idk what in the tile entity type lol i gotta find some use
*/ 

#[derive(Debug, Clone, Copy)]
pub struct BlockArray {
    pub block_type: [BlockType; 4096],
    pub block_id: [u16; 4096],
    pub texture_index: [u8; 4096],
    pub damage: [u8; 4096],
}

impl BlockArray {
    pub fn set_block_type_byindex(&mut self, index: usize, block_type: BlockType) {
        self.block_type[index] = block_type;
    }

    pub fn set_block_type(&mut self, chunk_relative_pos: ChunkRelativePos, block_type: BlockType) {
        self.block_type[chunk_relative_pos.y * 64 + chunk_relative_pos.x] = block_type;
    }

    pub fn set_block_id_byindex(&mut self, index: usize, id: u16) {
        self.block_id[index] = id;
    }

    pub fn set_block_id(&mut self, chunk_relative_pos: ChunkRelativePos, id: u16) {
        self.block_id[chunk_relative_pos.y * 64 + chunk_relative_pos.x] = id;
    }

    pub fn set_block_texture_index_byindex(&mut self, index: usize, texture_index: u8) {
        self.texture_index[index] = texture_index;
    }

    pub fn set_block_texture_index(&mut self, chunk_relative_pos: ChunkRelativePos, texture_index: u8) {
        self.texture_index[chunk_relative_pos.y * 64 + chunk_relative_pos.x] = texture_index;
    }

    pub fn set_block_damage_byindex(&mut self, index: usize, damage: u8) {
        self.damage[index] = damage;
    }

    pub fn set_block_damage(&mut self, chunk_relative_pos: ChunkRelativePos, damage: u8) {
        self.damage[chunk_relative_pos.y * 64 + chunk_relative_pos.x] = damage;
    }

    pub fn clear_block_byindex(&mut self, index: usize) {
        self.block_type[index] = BlockType::Air;
        self.block_id[index] = 0;
        self.texture_index[index] = 0;
        self.damage[index] = 0;
    }

    pub fn clear_block(&mut self, chunk_relative_pos: ChunkRelativePos) {
        let pos = chunk_relative_pos.y * 64 + chunk_relative_pos.x;
        self.block_type[pos] = BlockType::Air;
        self.block_id[pos] = 0;
        self.texture_index[pos] = 0;
        self.damage[pos] = 0;
    }

    pub fn filled_basic_tile() -> BlockArray {
        return BlockArray { 
            block_type: [BlockType::Tile; 4096],
            block_id: ([1; 4096]),
            texture_index: ([0; 4096]),
            damage: ([0; 4096])
        }
    }

    pub fn filled_basic_wall() -> BlockArray {
        return BlockArray { 
            block_type: [BlockType::Wall; 4096],
            block_id: ([1; 4096]),
            texture_index: ([0; 4096]),
            damage: ([0; 4096])
        }
    }

    pub fn filled_basic_air() -> BlockArray {
        return BlockArray { 
            block_type: [BlockType::Air; 4096],
            block_id: ([0; 4096]),
            texture_index: ([0; 4096]),
            damage: ([0; 4096])
        }
    }

    pub fn len(&self) -> usize {
        return 4096;
    }
}

// The fore and middle ground never have walls, while the background has only walls
#[derive(Debug, PartialEq)]
pub enum LayerType {
    Foreground,
    Middleground,
    Background
}