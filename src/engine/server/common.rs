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
pub struct Block {
    pub block_type: BlockType,
    pub block_id: u16,
    pub texture_index: u8,
    pub damage: u8,
}

impl Block {
    pub fn basic_tile() -> Block {
        return Block { 
            block_type: (BlockType::Tile),
            block_id: (1),
            texture_index: (0),
            damage: (0)
        }
    }

    pub fn basic_wall() -> Block {
        return Block { 
            block_type: (BlockType::Wall),
            block_id: (1),
            texture_index: (0),
            damage: (0)
        }
    }

    pub fn basic_air() -> Block {
        return Block { 
            block_type: (BlockType::Air),
            block_id: (0),
            texture_index: (0),
            damage: (0)
        }
    }
}

// The fore and middle ground never have walls, while the background has only walls

#[derive(Debug, PartialEq)]
pub enum LayerType {
    Foreground,
    Middleground,
    Background
}