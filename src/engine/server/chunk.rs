use std::{collections::HashSet, ptr, sync::Arc};

use fastnoise_lite::FastNoiseLite;
use fastrand::Rng;
use glam::IVec2;

use crate::engine::{common::{Block, ChunkMesh, ChunkRelativePos}, components::alive::{EntityID, PlayerID}, server::{biome::BiomeMap, chunk_generator::BakedHeightsCache, common::{BlockArray, BlockType, LayerType}, constants::{BIOME_SAMPLE_POINT_AMOUNT, CHUNK_BLOCK_COUNT, CHUNK_SIZE}, data::schema_definitions::BlendingMode}};

pub struct Chunk {
    pub foreground: BlockArray,
    pub middleground: BlockArray,
    pub background: BlockArray,
    total_block_count: u64,

    players: HashSet<PlayerID>,
    entites: HashSet<EntityID>,
}

impl Chunk {
    pub fn generate_chunk(position: &IVec2, biome_map: &BiomeMap, heights_cache: &BakedHeightsCache, generic_noise: &Arc<FastNoiseLite>, seed: i32) -> Chunk {
        let mut foreground = BlockArray::filled_basic_air();
        let chunk_world_pos = IVec2 { x: position.x * CHUNK_SIZE as i32, y: position.y * CHUNK_SIZE as i32 };

        // Get random biome points
        let (temperature_points, humidity_points) = get_biome_points(position, seed);

        // Sample the noise for those points
        let (sampled_temperature, sampled_humidity) =
            sample_noise_at_biome_points(temperature_points, humidity_points, generic_noise, &chunk_world_pos);

        // Interpolate the values from those points into a chunk-sized map
        let (temperature_map, humidity_map) =
            get_temperature_and_humidity_map(sampled_temperature, sampled_humidity);

        let chunk_is_multibiome = check_if_chunk_multibiome(biome_map, &temperature_map, &humidity_map);

        let mut total_block_count = 0;

        // If chunk has only one biome, generate (or sample from cache) in 1D
        if !chunk_is_multibiome {
            // Get entry if it exists, otherwise calculate it and insert it
            let precalculated_heights: [f32; CHUNK_SIZE as usize] = *heights_cache.entry(position.x).or_insert_with(|| {
                let biome_of_chunk = biome_map.get_biome(temperature_map[0], humidity_map[0]);
                let base_biome_generator = &biome_of_chunk.noise_generators;
                let base_biome_schema = &biome_of_chunk.noise_schema;
                let mut heights: [f32; CHUNK_SIZE as usize] = [0.0; CHUNK_SIZE as usize]; 

                for x in 0..CHUNK_SIZE {
                    let world_x = x as i32 + chunk_world_pos.x;

                    let mut height = 0.0;
                    let mut j = 0;

                    for (config, generator) in base_biome_schema.iter().zip(base_biome_generator.iter()) {
                        let generated_height = generator.get_noise_2d(world_x as f32, j as f32 * 250.0) * config.amplitude;
                        height = apply_blending(height, generated_height, &config.blending_mode);
                        j += 1;
                    }
                    heights[x as usize] = height;
                }

                heights
            });
            

            for i in 0..CHUNK_BLOCK_COUNT as usize {
                let x = i % CHUNK_SIZE as usize;
                let y = i / CHUNK_SIZE as usize;
                let world_y = y as i32 + chunk_world_pos.y;

                if generate_block_id(precalculated_heights[x as usize], world_y as f32, i, &mut foreground) {total_block_count += 1}
            }
        // If chunk has more than one biome, sample biome for every block and then generate the height
        } else {
            for i in 0..CHUNK_BLOCK_COUNT as usize {
                let x = i % CHUNK_SIZE as usize;
                let y = i / CHUNK_SIZE as usize;
                let world_x = x as i32 + chunk_world_pos.x;
                let world_y = y as i32 + chunk_world_pos.y;
                let biome = biome_map.get_biome(temperature_map[x], humidity_map[x]);

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
fn get_terrain_height(x: i32, biome_map: &BiomeMap, generic_noise: &Arc<FastNoiseLite>, seed: i32) {

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

fn get_temperature_and_humidity_map(sampled_temperature: [(IVec2, f32); BIOME_SAMPLE_POINT_AMOUNT], sampled_humidity: [(IVec2, f32); BIOME_SAMPLE_POINT_AMOUNT])
-> ([u8; CHUNK_BLOCK_COUNT as usize], [u8; CHUNK_BLOCK_COUNT as usize])
{
    let mut temperature: [u8; CHUNK_BLOCK_COUNT as usize] = [0; CHUNK_BLOCK_COUNT as usize];
    let mut humidity: [u8; CHUNK_BLOCK_COUNT as usize] = [0; CHUNK_BLOCK_COUNT as usize];

    for i in 0..CHUNK_BLOCK_COUNT as usize {
        let x = i % CHUNK_SIZE as usize;
        let y = i / CHUNK_SIZE as usize;
        let block_pos = IVec2 { x: x as i32, y: y as i32 };

        let final_temp_f32 = interpolate_idw(block_pos, &sampled_temperature);
        let final_hum_f32 = interpolate_idw(block_pos, &sampled_humidity);

        temperature[i] = final_temp_f32 as u8;
        humidity[i] = final_hum_f32 as u8;
    }

    (temperature, humidity)
}

fn check_if_chunk_multibiome(biome_map: &BiomeMap, temperature_map: &[u8; CHUNK_BLOCK_COUNT as usize], humidity_map: &[u8; CHUNK_BLOCK_COUNT as usize])
-> bool
{
    let mut chunk_is_multibiome = false;
    let base_biome = biome_map.get_biome(temperature_map[0], humidity_map[0]);
    for i in 1..CHUNK_BLOCK_COUNT {
        let biome = biome_map.get_biome(temperature_map[i as usize], humidity_map[i as usize]);
        if !ptr::eq(base_biome, biome) {
            chunk_is_multibiome = true;
            break;
        }
    }

    chunk_is_multibiome
}