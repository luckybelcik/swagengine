use std::collections::HashSet;

use glam::IVec2;
use noise_functions::{Noise, OpenSimplex2};

use crate::engine::{common::{ChunkRelativePos}, components::alive::{EntityID, PlayerID}, server::{common::{BlockArray, BlockType, LayerType}, constants::{CHUNK_BLOCK_COUNT, CHUNK_SIZE}}};

pub struct HeapChunk {
    pub chunk: Box<Chunk>,
}

pub struct Chunk {
    pub foreground: BlockArray,
    pub middleground: BlockArray,
    pub background: BlockArray,

    players: HashSet<PlayerID>,
    entites: HashSet<EntityID>,
}

impl Chunk {
    pub fn generate_chunk(position: &IVec2) -> Chunk {
        let mut foreground = BlockArray::filled_basic_air();
        let mut middleground = BlockArray::filled_basic_air();
        let chunk_world_x: usize = (position.x * CHUNK_SIZE as i32) as usize;
        let chunk_world_y: usize = (position.y * CHUNK_SIZE as i32) as usize;

        for i in 0..CHUNK_BLOCK_COUNT as usize {
            let x = i % CHUNK_SIZE as usize;
            let y = i / CHUNK_SIZE as usize;

            // Foreground sampling

            let fg_noise = OpenSimplex2.sample3([(x + chunk_world_x) as f32, (y + chunk_world_y) as f32, 0.0]);
            let fg_block_id = ((fg_noise + 1.0) * 16.0) as u16;

            foreground.set_block_id_byindex(i, fg_block_id);

            // Middleground sampling

            let mg_noise = OpenSimplex2.sample3([(x + chunk_world_x) as f32, (y + chunk_world_y) as f32, 1.0]);
            let mg_block_id = ((mg_noise + 1.0) * 16.0) as u16;

            middleground.set_block_id_byindex(i, mg_block_id);
        }

        return Chunk { 
            foreground: (foreground),
            middleground: (middleground),
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