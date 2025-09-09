use std::collections::HashSet;

use fastnoise_lite::FastNoiseLite;

use crate::engine::{common::{ChunkRelativePos, IVec2}, components::alive::{EntityID, PlayerID}, server::common::{Block, BlockType, LayerType}};

pub struct Chunk {
    foreground: [Block; 4096],
    middleground: [Block; 4096],
    background: [Block; 4096],

    players: HashSet<PlayerID>,
    entites: HashSet<EntityID>,
}

impl Chunk {
    pub fn generate_chunk(noise: &FastNoiseLite, position: &IVec2) -> Chunk {
        let mut foreground = [Block::basic_air(); 4096];
        let chunk_world_x: usize = (position.x * 64) as usize;
        let chunk_world_y: usize = (position.y * 64) as usize;

        for i in 0..foreground.len() {
            let x = i % 64;
            let y = i / 64;

            let noise = noise.get_noise_2d((x + chunk_world_x) as f32, (y + chunk_world_y) as f32);
            let block_id = (((noise + 1.0) * 16.0) as u16);

            foreground[i] = Block {
                block_type: (BlockType::Tile),
                block_id: block_id,
                texture_index: (0),
                damage: (0)
            }
        }

        return Chunk { 
            foreground: ([Block::basic_tile(); 4096]),
            middleground: ([Block::basic_air(); 4096]),
            background: ([Block::basic_wall(); 4096]),
            players: (HashSet::new()),
            entites: (HashSet::new())
        }
    }

    pub fn get_block(&self, chunk_relative_pos: ChunkRelativePos, layer: LayerType) -> &Block {
        let array: &[Block; 4096] = self.get_layer_immutable(layer);
        return &array[chunk_relative_pos.y * 64 + chunk_relative_pos.x];
    }

    pub fn set_block(&mut self, chunk_relative_pos: ChunkRelativePos, layer: LayerType, new_block: Block) {
        if layer == LayerType::Background && new_block.block_type != BlockType::Wall {
            println!("Attempted to place non-wall block in wall layer");
            return;
        } else if layer != LayerType::Background && new_block.block_type == BlockType::Wall {
            println!("Attempted to place wall block in non-wall layer");
            return;
        }
        
        let array: &mut [Block; 4096] = self.get_layer_mutable(layer);

        array[chunk_relative_pos.y * 64 + chunk_relative_pos.x] = new_block;
    }

    pub fn clear_block(&mut self, chunk_relative_pos: ChunkRelativePos, layer: LayerType) {
        let array: &mut [Block; 4096] = self.get_layer_mutable(layer);
        
        let index: usize = chunk_relative_pos.y * 64 + chunk_relative_pos.x;

        let block: &mut Block = &mut array[index];
        
        match block.block_type {
            BlockType::Tile => {
                array[index].block_type = BlockType::Air;
            }
            BlockType::Sprite => {
                panic!("Sprite clearing not implemented yet");
            }
            BlockType::TileEntity => {
                panic!("TileEntity clearing not implemented yet");
            }
            _ => {
                return;
            }
        }
    }

    // evil almost-duplicate functions (they're {slightly} more performant and offer better sightreading)

    pub fn change_block_property_damage(&mut self, chunk_relative_pos: ChunkRelativePos, layer: LayerType, new_type: BlockType) {
        let array: &mut [Block; 4096] = self.get_layer_mutable(layer);

        array[chunk_relative_pos.y * 64 + chunk_relative_pos.x].block_type = new_type;
    }

    pub fn change_block_property_id(&mut self, chunk_relative_pos: ChunkRelativePos, layer: LayerType, new_id: u16) {
        let array: &mut [Block; 4096] = self.get_layer_mutable(layer);

        array[chunk_relative_pos.y * 64 + chunk_relative_pos.x].block_id = new_id;
    }

    pub fn change_block_property_texture_index(&mut self, chunk_relative_pos: ChunkRelativePos, layer: LayerType, new_texture_index: u8) {
        let array: &mut [Block; 4096] = self.get_layer_mutable(layer);

        array[chunk_relative_pos.y * 64 + chunk_relative_pos.x].texture_index = new_texture_index;
    }

    fn get_layer_mutable(&mut self, layer: LayerType) -> &mut [Block; 4096] {
        match layer {
            LayerType::Foreground => {
                return &mut self.foreground;
            },
            LayerType::Middleground => {
                return &mut self.middleground;
            },
            LayerType::Background => {
                return &mut self.background; 
            },
        }
    }

    fn get_layer_immutable(&self, layer: LayerType) -> &[Block; 4096] {
        match layer {
            LayerType::Foreground => {
                return &self.foreground;
            },
            LayerType::Middleground => {
                return &self.middleground;
            },
            LayerType::Background => {
                return &self.background; 
            },
        }
    }
}