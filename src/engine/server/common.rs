#[derive(Debug, PartialEq)]
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

pub struct Block {
    pub block_type: BlockType,
    pub block_id: u16,
    pub texture_index: u8,
    pub damage: u8,
}

// The fore and middle ground never have walls, while the background has only walls

#[derive(Debug, PartialEq)]
pub enum LayerType {
    Foreground,
    Middleground,
    Background
}