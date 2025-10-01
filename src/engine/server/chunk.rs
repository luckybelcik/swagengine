use std::{collections::HashSet, ptr, sync::{Arc, RwLock}};

use fastnoise_lite::FastNoiseLite;
use fastrand::Rng;
use glam::IVec2;

use crate::engine::{common::{Block, ChunkMesh, ChunkRelativePos}, components::alive::{EntityID, PlayerID}, server::{biome::{Biome, BiomeMap}, chunk_generator::{BakedHeightsCache, ThreadlocalDimensionSchema}, common::{BlockArray, BlockType, LayerType}, constants::{BIOME_SAMPLE_POINT_AMOUNT, CHUNK_BLOCK_COUNT, CHUNK_SIZE}, data::schema_definitions::{BiomeConfig, BiomeMapAdjustments, BiomeTypes, BlendingMode, DimensionSchema}, world::Dimension}};

pub struct Chunk {
    pub foreground: BlockArray,
    pub middleground: BlockArray,
    pub background: BlockArray,
    total_block_count: u64,

    players: HashSet<PlayerID>,
    entites: HashSet<EntityID>,
}

impl Chunk {
    pub fn generate_chunk(position: &IVec2, biome_map: &BiomeMap, dimension_schema: &ThreadlocalDimensionSchema, heights_cache: &BakedHeightsCache, generic_noise: &Arc<FastNoiseLite>, seed: i32) -> Chunk {
        let mut foreground = BlockArray::filled_basic_air();
        let chunk_world_pos = IVec2 { x: position.x * CHUNK_SIZE as i32, y: position.y * CHUNK_SIZE as i32 };

        // Get random biome points
        let (temperature_points, humidity_points) = get_biome_points(position, seed);

        // Sample the noise for those points
        let (sampled_temperature, sampled_humidity) =
            sample_noise_at_biome_points(temperature_points, humidity_points, generic_noise, &chunk_world_pos);

        // Interpolate the values from those points into a chunk-sized map
        let (temperature_map, humidity_map) =
            get_temperature_and_humidity_map(&chunk_world_pos, sampled_temperature, sampled_humidity, dimension_schema);

        // Get terrain height
        let heights: [f32; CHUNK_SIZE as usize] = get_terrain_heights(&chunk_world_pos, biome_map, &temperature_map, &humidity_map, generic_noise, dimension_schema, heights_cache, seed);

        let mut total_block_count = 0;
        let mut block_picker_rng = Rng::with_seed(get_chunk_seed(seed, position));

        for i in 0..CHUNK_BLOCK_COUNT as usize {
            let x = i % CHUNK_SIZE as usize;
            let y = i / CHUNK_SIZE as usize;
            let world_y = y as i32 + chunk_world_pos.y;
            let blend_percentage = biome_map.get_blend_percentage(temperature_map[i], humidity_map[i]);

            let biome_to_use = if block_picker_rng.u8(0..100) < blend_percentage {
                biome_map.get_best_biome(temperature_map[i], humidity_map[i])
            } else {
                biome_map.get_second_best_biome(temperature_map[i], humidity_map[i])
            };

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

#[inline]
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

fn get_chunk_seed(world_seed: i32, chunk_pos: &IVec2) -> u64 {
    let s = world_seed as u32 as u64;
    let xx = chunk_pos.x as u32 as u64;
    let yy = chunk_pos.y as u32 as u64;
    
    let key = s.wrapping_mul(0xC2B2AE3D27D4EB4F)
                ^ xx.wrapping_mul(0x165667B19E3779F9)
                ^ yy.wrapping_mul(0x9E3779B97F4A7C15);
    splitmix64(key)
}

const IDW_POWER: f32 = 2.0;

fn interpolate_idw(
    block_pos_local: IVec2, 
    sampled_points: &[(IVec2, f32)],
) -> f32 {
    let mut total_weight: f32 = 0.0;
    let mut weighted_sum: f32 = 0.0;

    const EPSILON: f32 = 0.0001; 

    for (point_pos, point_value) in sampled_points.iter() {
        let dx = block_pos_local.x as f32 - point_pos.x as f32;
        let dy = block_pos_local.y as f32 - point_pos.y as f32;
        
        let distance_sq = dx * dx + dy * dy;
        
        if distance_sq < EPSILON {
            return *point_value;
        }
        
        let distance = distance_sq.sqrt();
        
        let weight = 1.0 / distance.powf(IDW_POWER); 
        
        weighted_sum += weight * point_value;
        total_weight += weight;
    }

    if total_weight > EPSILON {
        weighted_sum / total_weight
    } else {
        sampled_points.first().map(|(_, v)| *v).unwrap_or(0.0) 
    }
}

// We only sample the 1D height noise at the y coordinate 0 to avoid weird artifacts
// If u wanna know why, DM me and I will explain
fn get_terrain_heights(chunk_world_pos: &IVec2, biome_map: &BiomeMap, temperature_map: &[u8; CHUNK_BLOCK_COUNT as usize], humidity_map: &[u8; CHUNK_BLOCK_COUNT as usize], generic_noise: &Arc<FastNoiseLite>, dimension_schema: &ThreadlocalDimensionSchema, heights_cache: &BakedHeightsCache, seed: i32)
-> [f32; CHUNK_SIZE as usize] {
    let chunk_pos = IVec2 { x: chunk_world_pos.x / CHUNK_SIZE as i32, y: chunk_world_pos.y / CHUNK_SIZE as i32 };
    // Check if terrain height is not yet cached
    let heights: [f32; CHUNK_SIZE as usize] = *heights_cache.entry(chunk_pos.x).or_insert_with(|| {
        // If it isn't cached, we run the logic

        // We need the biomes at world y0
        let filler_biome = biome_map.get_best_biome(0, 0);
        let biomes: [&Biome; CHUNK_SIZE as usize] = if chunk_world_pos.y == 0 {
            // If chunk is at world pos y0, we use it's temperature and humidity map
            let mut biomes = [filler_biome; CHUNK_SIZE as usize];
            for i in 0..CHUNK_SIZE as usize {
                biomes[i] = biome_map.get_best_biome(temperature_map[i as usize], humidity_map[i as usize])
            }
            biomes
        } else {
            // If chunk is not at world pos y0, we need to resample the temperature and humidity maps
            // for the chunk at y0
            // Get random biome points
            let chunk_pos_y0 = IVec2 { x: chunk_pos.x, y: 0 };
            let (temperature_points, humidity_points) = get_biome_points(&chunk_pos_y0, seed);

            // Sample the noise for those points
            let (sampled_temperature, sampled_humidity) =
                sample_noise_at_biome_points(temperature_points, humidity_points, generic_noise, &chunk_world_pos);

            // Interpolate the values from those points into a chunk-sized map
            let (temperature_map_y0, humidity_map_y0) =
                get_temperature_and_humidity_map(&chunk_pos_y0, sampled_temperature, sampled_humidity, dimension_schema);

            let mut biomes = [filler_biome; CHUNK_SIZE as usize];
            for i in 0..CHUNK_SIZE as usize {
                biomes[i] = biome_map.get_best_biome(temperature_map_y0[i as usize], humidity_map_y0[i as usize])
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

            for (config, generator) in current_biome_schema.iter().zip(current_biome_generators.iter()) {
                let generated_height = generator.get_noise_2d(world_x as f32, j as f32 * 250.0) * config.amplitude;
                height = apply_blending(height, generated_height, &config.blending_mode);
                j += 1;
            }
            heights[x as usize] = height;
        }

        heights
    });

    heights
}

fn get_biome_points(position: &IVec2, seed: i32) -> ([IVec2; BIOME_SAMPLE_POINT_AMOUNT], [IVec2; BIOME_SAMPLE_POINT_AMOUNT]){
    let empty_ivec = IVec2 { x: 0, y: 0};
    let mut temperature_points: [IVec2; BIOME_SAMPLE_POINT_AMOUNT] = [empty_ivec; BIOME_SAMPLE_POINT_AMOUNT];
    let mut humidity_points: [IVec2; BIOME_SAMPLE_POINT_AMOUNT] = [empty_ivec; BIOME_SAMPLE_POINT_AMOUNT];
    let mut rng = Rng::with_seed(get_chunk_seed(seed, position));
    let mut generated_coords: HashSet<IVec2> = HashSet::new();
    let mut i: usize = 0;   

    while i < BIOME_SAMPLE_POINT_AMOUNT * 2 {
        let x = rng.i32(0..(CHUNK_SIZE as i32));
        let y = rng.i32(0..(CHUNK_SIZE as i32));

        let point = IVec2::new(x, y);

        if generated_coords.insert(point) {
            if i < BIOME_SAMPLE_POINT_AMOUNT {
                temperature_points[i] = point;
            } else {
                humidity_points[i - BIOME_SAMPLE_POINT_AMOUNT] = point;
            }
        }

        i += 1;
    }

    (temperature_points, humidity_points)
}

fn sample_noise_at_biome_points(temperature_points: [IVec2; BIOME_SAMPLE_POINT_AMOUNT], humidity_points: [IVec2; BIOME_SAMPLE_POINT_AMOUNT], generic_noise: &Arc<FastNoiseLite>, chunk_world_pos: &IVec2)
->  ([(IVec2, f32); BIOME_SAMPLE_POINT_AMOUNT], [(IVec2, f32); BIOME_SAMPLE_POINT_AMOUNT])
{
    let empty_point: (IVec2, f32) = (IVec2 {x: 0, y:0}, 0.0);
    let mut sampled_temperature: [(IVec2, f32); _] = [empty_point; BIOME_SAMPLE_POINT_AMOUNT];
    let mut sampled_humidity: [(IVec2, f32); _] = [empty_point; BIOME_SAMPLE_POINT_AMOUNT];

    for i in 0..BIOME_SAMPLE_POINT_AMOUNT {
        let temperature_point = temperature_points[i];
        let world_x = (temperature_point.x + chunk_world_pos.x) as f32;
        let world_y = (temperature_point.y + chunk_world_pos.y) as f32;
        sampled_temperature[i] = (temperature_point, (generic_noise.get_noise_3d(world_x, world_y, 250.0) + 1.0 ) * 50.0 );

        
        let humidity_point = humidity_points[i];
        let world_x = (humidity_point.x + chunk_world_pos.x) as f32;
        let world_y = (humidity_point.y + chunk_world_pos.y) as f32;
        sampled_humidity[i] = (humidity_point, (generic_noise.get_noise_3d(world_x, world_y, 250.0) + 1.0 ) * 50.0 );
    }

    (sampled_temperature, sampled_humidity)
}

fn get_temperature_and_humidity_map(chunk_world_pos: &IVec2, sampled_temperature: [(IVec2, f32); BIOME_SAMPLE_POINT_AMOUNT], sampled_humidity: [(IVec2, f32); BIOME_SAMPLE_POINT_AMOUNT], dimension_schema: &ThreadlocalDimensionSchema)
-> ([u8; CHUNK_BLOCK_COUNT as usize], [u8; CHUNK_BLOCK_COUNT as usize])
{
    let mut temperature: [u8; CHUNK_BLOCK_COUNT as usize] = [0; CHUNK_BLOCK_COUNT as usize];
    let mut humidity: [u8; CHUNK_BLOCK_COUNT as usize] = [0; CHUNK_BLOCK_COUNT as usize];
    let full_world_size_x = (&dimension_schema.size.x * CHUNK_SIZE as u32) as f32;
    let full_world_size_y = (&dimension_schema.size.y * CHUNK_SIZE as u32) as f32;

    let (horiz_var, vert_var) = if let Some(adjustments) = &dimension_schema.biome_map_adjustments {
        (adjustments.horizontal_temperature_variation as f32, 
         adjustments.vertical_temperature_variation as f32)
    } else {
        (0.0, 0.0)
    };


    for i in 0..CHUNK_BLOCK_COUNT as usize {
        let x = i % CHUNK_SIZE as usize;
        let y = i / CHUNK_SIZE as usize;

        let world_x = x as f32 + chunk_world_pos.x as f32;
        let world_y = y as f32 + chunk_world_pos.y as f32;

        let half_world_size_x = full_world_size_x / 2.0;
        let half_world_size_y = full_world_size_y / 2.0;

        let relative_x = (world_x + half_world_size_x) / full_world_size_x;

        let horizontal_bias: f32 = if full_world_size_x > 0.0 {
            if relative_x < 1.0 / 3.0 {
                // Left third: Colder
                let bias_factor = 1.0 - (relative_x * 3.0); 
                bias_factor * horiz_var
            } else if relative_x >= 2.0 / 3.0 {
                // Right third: Warmer
                let bias_factor = (relative_x * 3.0) - 2.0; 
                -bias_factor * horiz_var
            } else {
                // Middle third: Neutral
                0.0
            }
        } else { 0.0 };

        let vertical_bias: f32 = if full_world_size_y > 0.0 && world_y > 10.0 {
            // Higher = colder
            let bias_factor = world_y / half_world_size_y;
            
            -bias_factor * vert_var
        } else {
            0.0
        };

        let total_bias = horizontal_bias + vertical_bias;

        let block_pos = IVec2 { x: x as i32, y: y as i32 };

        let final_temp_f32 = interpolate_idw(block_pos, &sampled_temperature);
        let final_hum_f32 = interpolate_idw(block_pos, &sampled_humidity);

        // Remap to 35-65
        let final_temp_f32 = ((final_temp_f32 - 50.0) * 0.3) + 50.0;

        temperature[i] = (final_temp_f32 + total_bias).clamp(0.0, 100.0).round() as u8;
        humidity[i] = final_hum_f32 as u8;
    }

    (temperature, humidity)
}

fn check_if_chunk_multibiome(biome_map: &BiomeMap, temperature_map: &[u8; CHUNK_BLOCK_COUNT as usize], humidity_map: &[u8; CHUNK_BLOCK_COUNT as usize])
-> bool
{
    let mut chunk_is_multibiome = false;
    let base_biome = biome_map.get_best_biome(temperature_map[0], humidity_map[0]);
    for i in 1..CHUNK_BLOCK_COUNT {
        let biome = biome_map.get_best_biome(temperature_map[i as usize], humidity_map[i as usize]);
        if !ptr::eq(base_biome, biome) {
            chunk_is_multibiome = true;
            break;
        }
    }

    chunk_is_multibiome
}