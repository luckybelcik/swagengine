use std::collections::HashSet;

use glam::IVec2;
use noise_functions::{modifiers::Frequency, Noise, OpenSimplex2};

use crate::engine::{common::{Block, ChunkMesh, ChunkRelativePos, PacketChunk}, components::alive::{EntityID, PlayerID}, server::{common::{BasicNoiseGenerators, BlockArray, BlockType, LayerType}, constants::{CHUNK_BLOCK_COUNT, CHUNK_SIZE}}};

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
    pub fn generate_chunk(position: &IVec2, noise_generators: &BasicNoiseGenerators) -> Chunk {
        let mut foreground = BlockArray::filled_basic_air();
        let chunk_world_x = position.x * CHUNK_SIZE as i32;
        let chunk_world_y = position.y * CHUNK_SIZE as i32;

        // Stuff used for sparse generation and interpolation for the continental noise
        const CONTINENTAL_SPARSE_FACTOR: usize = 4;
        const CONTINENTAL_SPARSE_POINTS: usize = (CHUNK_SIZE as usize / CONTINENTAL_SPARSE_FACTOR) + 1;
        let mut sparse_continental_height: [f32; CONTINENTAL_SPARSE_POINTS] = [0.0; CONTINENTAL_SPARSE_POINTS];

        // Sample only in the sparse points
        for i in 0..CONTINENTAL_SPARSE_POINTS {
            let sparse_x = i * CONTINENTAL_SPARSE_FACTOR;
            let world_x = sparse_x as i32 + chunk_world_x;
            sparse_continental_height[i] = noise_generators.continental.sample2([world_x as f32, 100.0]) * 35.0;
        }

        let mut pregenerated_base_height: [f32; CHUNK_SIZE as usize] = [0.0; CHUNK_SIZE as usize];
        let mut pregenerated_continental_height: [f32; CHUNK_SIZE as usize] = [0.0; CHUNK_SIZE as usize];
        
        // Base height pregeneration && sparse point interpolation
        for i in 0..CHUNK_SIZE as usize {
            //* Pregeneration
            let x = i % CHUNK_SIZE as usize;
            let world_x = x as i32 + chunk_world_x;
            pregenerated_base_height[i] = noise_generators.base.sample2([world_x as f32, 0.0]) * 6.0;

            //* Interpolation
            // Get points & presampled values
            let index_p1 = i / CONTINENTAL_SPARSE_FACTOR;
            let index_p2 = index_p1 + 1;

            let val_p1 = sparse_continental_height[index_p1];
            let val_p2 = sparse_continental_height[index_p2];

            // Get interpolation factor & lerp
            let t = (x % CONTINENTAL_SPARSE_FACTOR) as f32 / CONTINENTAL_SPARSE_FACTOR as f32;
            let interpolated_value = val_p1 * (1.0 - t) + val_p2 * t;

            pregenerated_continental_height[x] = interpolated_value;
        }

        for i in 0..CHUNK_BLOCK_COUNT as usize {
            let x = i % CHUNK_SIZE as usize;
            let y = i / CHUNK_SIZE as usize;
            let world_y = y as i32 + chunk_world_y;

            let fg_height = pregenerated_base_height[x] + pregenerated_continental_height[x];

            // World painting!!! :D (adding block types)
            // If sampled tile is below ground
            if fg_height >= world_y as f32 {
                let tiles_below_surface = fg_height as i32 - world_y;
                let fg_block_id = match tiles_below_surface {
                    0 => 2,     // Grass
                    1..=5 => 1, // Dirt
                    _ => 0,     // Stone
                };
                foreground.set_block_id_byindex(i, fg_block_id);
                foreground.set_block_type_byindex(i, BlockType::Tile);
            } else if world_y <= 0 { // If above ground but below or at y0
                foreground.set_block_id_byindex(i, 3); // 3 = water
                foreground.set_block_type_byindex(i, BlockType::Tile);
            }
        }

        return Chunk { 
            foreground: (foreground),
            middleground: (BlockArray::filled_basic_air()),
            background: (BlockArray::filled_basic_air()),
            players: (HashSet::new()),
            entites: (HashSet::new())
        }
    }

    // evil almost-duplicate functions (they're {slightly} more performant and offer better sightreading)

    pub fn change_block_property_type(&mut self, chunk_relative_pos: ChunkRelativePos, layer: LayerType, new_type: BlockType) {
        self.get_block_array_mut(layer).set_block_type(chunk_relative_pos, new_type);
    }

    pub fn change_block_property_id(&mut self, chunk_relative_pos: ChunkRelativePos, layer: LayerType, new_id: u32) {
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

    pub fn to_mesh(&self) -> ChunkMesh {
        return ChunkMesh {
            foreground: convert_layer_to_aos(self.foreground),
            middleground: convert_layer_to_aos(self.middleground),
            background: convert_layer_to_aos(self.background)
        }
    }
}

fn convert_layer_to_aos(layer: BlockArray) -> [Block; CHUNK_BLOCK_COUNT as usize] {
    core::array::from_fn(|i| {
        Block {
            x: (i % CHUNK_SIZE as usize) as u8,
            y: (i / CHUNK_SIZE as usize) as u8,
            block_id: layer.block_id[i] as u32,
            block_type: layer.block_type[i] as u8,
            texture_index: layer.texture_index[i],
        }
    })
}