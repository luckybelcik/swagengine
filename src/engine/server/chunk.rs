use std::collections::HashSet;

use fastnoise_lite::FastNoiseLite;

use crate::engine::{common::{ChunkRelativePos, IVec2}, components::alive::{EntityID, PlayerID}, server::common::{BlockArray, BlockType, LayerType}};

pub struct Chunk {
    foreground: BlockArray,
    middleground: BlockArray,
    background: BlockArray,

    players: HashSet<PlayerID>,
    entites: HashSet<EntityID>,
}

impl Chunk {
    pub fn generate_chunk(noise: &FastNoiseLite, position: &IVec2) -> Chunk {
        let mut foreground = BlockArray::filled_basic_air();
        let chunk_world_x: usize = (position.x * 64) as usize;
        let chunk_world_y: usize = (position.y * 64) as usize;

        for i in 0..foreground.len() {
            let x = i % 64;
            let y = i / 64;

            let noise = noise.get_noise_2d((x + chunk_world_x) as f32, (y + chunk_world_y) as f32);
            let block_id = ((noise + 1.0) * 16.0) as u16;

            foreground.set_block_id_byindex(i, block_id);
        }

        return Chunk { 
            foreground: (foreground),
            middleground: (BlockArray::filled_basic_air()),
            background: (BlockArray::filled_basic_wall()),
            players: (HashSet::new()),
            entites: (HashSet::new())
        }
    }

    // evil almost-duplicate functions (they're {slightly} more performant and offer better sightreading)

    pub fn change_block_property_type(&mut self, chunk_relative_pos: ChunkRelativePos, layer: LayerType, new_type: BlockType) {
        self.get_block_array_mut(layer).set_block_type(chunk_relative_pos, new_type);
    }

    pub fn change_block_property_id(&mut self, chunk_relative_pos: ChunkRelativePos, layer: LayerType, new_id: u16) {
        self.get_block_array_mut(layer).set_block_id(chunk_relative_pos, new_id);
    }

    pub fn change_block_property_texture_index(&mut self, chunk_relative_pos: ChunkRelativePos, layer: LayerType, new_texture_index: u8) {
        self.get_block_array_mut(layer).set_block_texture_index(chunk_relative_pos, new_texture_index);
    }

    pub fn change_block_property_damage(&mut self, chunk_relative_pos: ChunkRelativePos, layer: LayerType, damage: u8) {
        self.get_block_array_mut(layer).set_block_damage(chunk_relative_pos, damage);
    }

    fn get_block_array_mut(&mut self, layer: LayerType) -> &mut BlockArray {
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

    fn get_block_array_immut(&self, layer: LayerType) -> &BlockArray {
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