use std::collections::HashSet;

use crate::engine::{common::ChunkRelativePos, components::alive::{EntityID, PlayerID}, server::common::{Block, BlockType, LayerType}};

pub struct Chunk {
    foreground: [Block; 4096],
    middleground: [Block; 4096],
    background: [Block; 4096],

    players: HashSet<PlayerID>,
    entites: HashSet<EntityID>,
}

impl Chunk {
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