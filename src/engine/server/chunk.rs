use std::{collections::HashSet, sync::{Arc}};

use glam::IVec2;

use crate::engine::{common::{Block, ChunkMesh, ChunkRelativePos}, components::alive::{EntityID, PlayerID}, server::{biome::{Biome, BiomeMap}, common::{BlockArray, BlockType, LayerType}, constants::{CHUNK_BLOCK_COUNT, CHUNK_SIZE, HUMIDITY_INDEX, TEMPERATURE_INDEX}, data::schema_definitions::{BiomeConfig, BiomeTypes, BlendingMode}, noise::{noise_sampler::NoiseSampler}}};

pub struct Chunk {
    pub foreground: BlockArray,
    pub middleground: BlockArray,
    pub background: BlockArray,
    total_block_count: u64,

    players: HashSet<PlayerID>,
    entites: HashSet<EntityID>,
}

impl Chunk {
    pub fn generate_chunk(chunk_pos: &IVec2, biome_map: &BiomeMap, noise_sampler: &Arc<NoiseSampler>, seed: i32) -> Chunk {
        let mut foreground = BlockArray::filled_basic_air();
        let chunk_world_pos = IVec2 { x: chunk_pos.x * CHUNK_SIZE as i32, y: chunk_pos.y * CHUNK_SIZE as i32 };

        let temperature_map = noise_sampler.get_noise_layer_2d(&chunk_pos, TEMPERATURE_INDEX);
        let humidity_map = noise_sampler.get_noise_layer_2d(&chunk_pos, HUMIDITY_INDEX);

        // Get terrain height
        let heights: [f32; CHUNK_SIZE as usize] = get_terrain_heights(&chunk_world_pos, biome_map, noise_sampler);

        let mut total_block_count = 0;

        for i in 0..CHUNK_BLOCK_COUNT as usize {
            let x = i % CHUNK_SIZE as usize;
            let y = i / CHUNK_SIZE as usize;
            let world_y = y as i32 + chunk_world_pos.y;

            let biome_to_use = biome_map.get_best_biome(temperature_map.read_index(i), humidity_map.read_index(i));

            if generate_block_id(heights[x as usize], world_y as f32, i, &mut foreground,
                &biome_to_use.biome_config) {total_block_count += 1}
        }

        return Chunk { 
            foreground,
            middleground: (BlockArray::filled_basic_air()),
            background: (BlockArray::filled_basic_air()),
            total_block_count,
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

    pub fn get_total_block_count(&self) -> u64 {
        self.total_block_count
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

fn generate_block_id(height: f32, world_y: f32, i: usize, layer: &mut BlockArray, biome_config: &BiomeConfig) -> bool {
    if height >= world_y as f32 {
        let tiles_below_surface = height as i32 - world_y as i32;
        let fg_block_id = match tiles_below_surface {
            0 => biome_config.surface_block,
            1..=5 => biome_config.subsurface_block,
            _ => biome_config.base_block,
        };
        layer.set_block_id_byindex(i, fg_block_id);
        layer.set_block_type_byindex(i, BlockType::Tile);
        return true;
    } else if world_y <= 0.0 { // If above ground but below or at y0
        // If surface level and cold, place ice
        if world_y == 0.0 && biome_config.biome_type == BiomeTypes::Cold {
            layer.set_block_id_byindex(i, 6); // 6 = ice
            layer.set_block_type_byindex(i, BlockType::Tile);
            return true;
        } 

        // If above -5 and warm, no water
        if world_y > -5.0 && biome_config.biome_type == BiomeTypes::Warm {
            return false;
        }

        // Always return ice if freezing
        if biome_config.biome_type == BiomeTypes::Freezing {
            layer.set_block_id_byindex(i, 6); // 6 = ice
            layer.set_block_type_byindex(i, BlockType::Tile);
            return true;
        }

        // Never place water if hot
        if biome_config.biome_type == BiomeTypes::Hot {
            return false;
        }

        // Just place water if no other case fulfilled
        layer.set_block_id_byindex(i, 3); // 3 = watar
        layer.set_block_type_byindex(i, BlockType::Tile);
        return true;
    }
    return false;
}

fn apply_blending(height: f32, generated_height: f32, blending_mode: &BlendingMode) -> f32 {
    match blending_mode {
        BlendingMode::Mix => return height + generated_height,
        BlendingMode::MixPositive => height + generated_height.max(0.0),
        BlendingMode::MixNegative => height + generated_height.min(0.0),
        BlendingMode::Add => height + generated_height.abs(),
        BlendingMode::Subtract => height - generated_height.abs(),
        BlendingMode::Multiply => height * generated_height,
    }
}

fn get_terrain_heights(chunk_world_pos: &IVec2, biome_map: &BiomeMap, noise_sampler: &Arc<NoiseSampler>)
-> [f32; CHUNK_SIZE as usize] {
    let chunk_pos = IVec2 { x: chunk_world_pos.x / CHUNK_SIZE as i32, y: chunk_world_pos.y / CHUNK_SIZE as i32 };
    
    let heights: [f32; CHUNK_SIZE as usize] = {
        // We need the biomes at world y0 for terrain sampling
        let chunk_pos_y0 = IVec2 {x: chunk_pos.x, y: 0};
        let filler_biome = biome_map.get_best_biome(0, 0);
        let biomes: [&Biome; CHUNK_SIZE as usize] = {
            let temperature_map = noise_sampler.get_noise_layer_2d(&chunk_pos_y0, TEMPERATURE_INDEX);
            let humidity_map = noise_sampler.get_noise_layer_2d(&chunk_pos_y0, HUMIDITY_INDEX);

            let mut biomes = [filler_biome; CHUNK_SIZE as usize];
            for i in 0..CHUNK_SIZE as usize {
                biomes[i] = biome_map.get_best_biome(temperature_map.read_index(i), humidity_map.read_index(i))
            }
            biomes
        };

        let mut heights: [f32; CHUNK_SIZE as usize] = [0.0; CHUNK_SIZE as usize]; 

        for x in 0..CHUNK_SIZE {
            let world_x = x as i32 + chunk_world_pos.x;
            let current_biome = biomes[x as usize];
            let current_biome_schema = &current_biome.noise_schema;

            let mut height = 0.0;
            let mut j = 0;

            for config in current_biome_schema.iter() {
                let generated_height = noise_sampler.get_noise_1d(world_x, j) * config.amplitude;
                height = apply_blending(height, generated_height, &config.blending_mode);
                j += 1;
            }
            heights[x as usize] = height;
        }

        heights
    };

    heights
}