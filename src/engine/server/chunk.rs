use std::{collections::HashSet, ptr};

use glam::IVec2;

use crate::engine::{common::{Block, ChunkMesh, ChunkRelativePos}, components::alive::{EntityID, PlayerID}, server::{biome::{BiomeMap}, common::{BlockArray, BlockType, LayerType}, constants::{CHUNK_BLOCK_COUNT, CHUNK_SIZE}, data::schema_definitions::BlendingMode}};

pub struct HeapChunk {
    pub chunk: Box<Chunk>,
}

pub struct Chunk {
    pub foreground: BlockArray,
    pub middleground: BlockArray,
    pub background: BlockArray,
    total_block_count: u64,

    players: HashSet<PlayerID>,
    entites: HashSet<EntityID>,
}

impl Chunk {
    pub fn generate_chunk(position: &IVec2, biome_map: &BiomeMap) -> Chunk {
        let mut foreground = BlockArray::filled_basic_air();
        let chunk_world_x = position.x * CHUNK_SIZE as i32;
        let chunk_world_y = position.y * CHUNK_SIZE as i32;

        const BIOME_SPARSE_FACTOR: usize = 4;
        const BIOME_SPARSE_POINTS: usize = (CHUNK_SIZE as usize / BIOME_SPARSE_FACTOR) + 1;
        const BIOME_SPARSE_POINT_COUNT: usize = BIOME_SPARSE_POINTS * BIOME_SPARSE_POINTS;
        let sparse_temperature_points: [f32; BIOME_SPARSE_POINT_COUNT] = [50.0; BIOME_SPARSE_POINT_COUNT];
        let sparse_humidity_points: [f32; BIOME_SPARSE_POINT_COUNT] = [50.0; BIOME_SPARSE_POINT_COUNT];
        let mut temperature: [u8; CHUNK_BLOCK_COUNT as usize] = [0; CHUNK_BLOCK_COUNT as usize];
        let mut humidity: [u8; CHUNK_BLOCK_COUNT as usize] = [0; CHUNK_BLOCK_COUNT as usize];

        for i in 0..CHUNK_BLOCK_COUNT as usize {
            let x = i % CHUNK_SIZE as usize;
            let y = i / CHUNK_SIZE as usize;

            let sx1 = x / BIOME_SPARSE_FACTOR;
            let sy1 = y / BIOME_SPARSE_FACTOR;

            let sx2 = sx1 + 1;
            let sy2 = sy1 + 1;

            let tx = (x % BIOME_SPARSE_FACTOR) as f32 / BIOME_SPARSE_FACTOR as f32;
            let ty = (y % BIOME_SPARSE_FACTOR) as f32 / BIOME_SPARSE_FACTOR as f32;

            let get_sparse_point = |sx: usize, sy: usize, points: &[f32; BIOME_SPARSE_POINT_COUNT]| -> f32 {
                let index = sy * BIOME_SPARSE_POINTS + sx;
                points[index]
            };

            let p1_temp = get_sparse_point(sx1, sy1, &sparse_temperature_points);
            let p2_temp = get_sparse_point(sx2, sy1, &sparse_temperature_points);
            let p3_temp = get_sparse_point(sx1, sy2, &sparse_temperature_points);
            let p4_temp = get_sparse_point(sx2, sy2, &sparse_temperature_points);
                
            let lerp_top_temp = p1_temp * (1.0 - tx) + p2_temp * tx;
            let lerp_bottom_temp = p3_temp * (1.0 - tx) + p4_temp * tx;

            let final_temp = lerp_top_temp * (1.0 - ty) + lerp_bottom_temp * ty;

            temperature[i] = final_temp as u8;

            let p1_hum = get_sparse_point(sx1, sy1, &sparse_humidity_points);
            let p2_hum = get_sparse_point(sx2, sy1, &sparse_humidity_points);
            let p3_hum = get_sparse_point(sx1, sy2, &sparse_humidity_points);
            let p4_hum = get_sparse_point(sx2, sy2, &sparse_humidity_points);

            let lerp_top_hum = p1_hum * (1.0 - tx) + p2_hum * tx;
            let lerp_bottom_hum = p3_hum * (1.0 - tx) + p4_hum * tx;

            let final_hum = lerp_top_hum * (1.0 - ty) + lerp_bottom_hum * ty;

            humidity[i] = final_hum as u8;
        }

        // todo Random temperature and humidity generation

        let mut chunk_is_multibiome = false;
        let base_biome = biome_map.get_biome(temperature[0], humidity[0]);
        for i in 1..CHUNK_BLOCK_COUNT {
            let biome = biome_map.get_biome(temperature[i as usize], humidity[i as usize]);
            if !ptr::eq(base_biome, biome) {
                chunk_is_multibiome = true;
                break;
            }
        }

        let mut total_block_count = 0;

        if !chunk_is_multibiome {
            let mut precalculated_heights: [f32; CHUNK_SIZE as usize] = [0.0; CHUNK_SIZE as usize];
            let base_biome_generator = &base_biome.noise_generators;
            let base_biome_schema = &base_biome.noise_schema;

            for x in 0..CHUNK_SIZE {
                let world_x = x as i32 + chunk_world_x;

                let mut height = 0.0;
                let mut j = 0;

                for (config, generator) in base_biome_schema.iter().zip(base_biome_generator.iter()) {
                    let generated_height = generator.get_noise_2d(world_x as f32, j as f32 * 250.0) * config.amplitude;
                    height = apply_blending(height, generated_height, &config.blending_mode);
                    j += 1;
                }
                precalculated_heights[x as usize] = height;
            }

            for i in 0..CHUNK_BLOCK_COUNT as usize {
                let x = i % CHUNK_SIZE as usize;
                let y = i / CHUNK_SIZE as usize;
                let world_y = y as i32 + chunk_world_y;

                if generate_block_id(precalculated_heights[x as usize], world_y as f32, i, &mut foreground) {total_block_count += 1}
            }
        } else {
            for i in 0..CHUNK_BLOCK_COUNT as usize {
                let x = i % CHUNK_SIZE as usize;
                let y = i / CHUNK_SIZE as usize;
                let world_x = x as i32 + chunk_world_x;
                let world_y = y as i32 + chunk_world_y;
                let biome = biome_map.get_biome(temperature[x], humidity[x]);

                let mut height = 0.0;
                let mut j = 0;

                for (config, generator) in biome.noise_schema.iter().zip(biome.noise_generators.iter()) {
                    let generated_height = generator.get_noise_2d(world_x as f32, j as f32 * 250.0) * config.amplitude;
                    height = apply_blending(height, generated_height, &config.blending_mode);
                    j += 1;
                }

                if generate_block_id(height, world_y as f32, i, &mut foreground) {total_block_count += 1}
            }
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

fn generate_block_id(height: f32, world_y: f32, i: usize, layer: &mut BlockArray) -> bool {
    if height >= world_y as f32 {
        let tiles_below_surface = height as i32 - world_y as i32;
        let fg_block_id = match tiles_below_surface {
            0 => 2,     // Grass
            1..=5 => 1, // Dirt
            _ => 0,     // Stone
        };
        layer.set_block_id_byindex(i, fg_block_id);
        layer.set_block_type_byindex(i, BlockType::Tile);
        return true;
    } else if world_y <= 0.0 { // If above ground but below or at y0
        layer.set_block_id_byindex(i, 3); // 3 = water
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