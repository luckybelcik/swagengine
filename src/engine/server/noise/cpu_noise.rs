use std::collections::HashSet;

use fastnoise_lite::{FastNoiseLite, NoiseType};
use fastrand::Rng;
use glam::IVec2;

use crate::engine::server::{chunk_generator::ThreadlocalDimensionSchema, constants::{BIOME_SAMPLE_POINT_AMOUNT, CHUNK_BLOCK_COUNT, CHUNK_SIZE}, noise::noise_util::{get_chunk_seed, interpolate_idw}};

pub struct CPUNoise {
    biome_sampling_noise: FastNoiseLite,
}

impl CPUNoise {
    pub fn new(world_seed: i32) -> CPUNoise {
        let mut biome_sampling_noise = FastNoiseLite::with_seed(world_seed * -1);
        biome_sampling_noise.set_frequency(Some(0.0001));
        biome_sampling_noise.set_noise_type(Some(NoiseType::OpenSimplex2));

        CPUNoise {
            biome_sampling_noise,
        }
    }

    pub fn get_temperature_and_humidity_map(&self, chunk_pos: &IVec2, world_seed: i32, dimension_schema: &ThreadlocalDimensionSchema)
    -> ([u8; CHUNK_BLOCK_COUNT as usize], [u8; CHUNK_BLOCK_COUNT as usize]) {
        let chunk_seed = get_chunk_seed(world_seed, chunk_pos);
        let chunk_world_pos = IVec2 { x: chunk_pos.x * CHUNK_SIZE as i32, y: chunk_pos.y * CHUNK_SIZE as i32 };
        
        // Get random biome points
        let (temperature_points, humidity_points) =
            get_biome_points(chunk_seed);

        // Sample the noise for those points
        let (sampled_temperature, sampled_humidity) =
            self.sample_noise_at_biome_points(temperature_points, humidity_points, &chunk_world_pos);

        // Interpolate the values from those points into a chunk-sized map
        generate_temperature_and_humidity_map(&chunk_world_pos, sampled_temperature, sampled_humidity, dimension_schema)
    }

    fn sample_noise_at_biome_points(&self, temperature_points: [IVec2; BIOME_SAMPLE_POINT_AMOUNT], humidity_points: [IVec2; BIOME_SAMPLE_POINT_AMOUNT], chunk_world_pos: &IVec2)
    ->  ([(IVec2, f32); BIOME_SAMPLE_POINT_AMOUNT], [(IVec2, f32); BIOME_SAMPLE_POINT_AMOUNT])
    {
        let empty_point: (IVec2, f32) = (IVec2 {x: 0, y:0}, 0.0);
        let mut sampled_temperature: [(IVec2, f32); _] = [empty_point; BIOME_SAMPLE_POINT_AMOUNT];
        let mut sampled_humidity: [(IVec2, f32); _] = [empty_point; BIOME_SAMPLE_POINT_AMOUNT];

        for i in 0..BIOME_SAMPLE_POINT_AMOUNT {
            let temperature_point = temperature_points[i];
            let world_x = (temperature_point.x + chunk_world_pos.x) as f32;
            let world_y = (temperature_point.y + chunk_world_pos.y) as f32;
            sampled_temperature[i] = (temperature_point, (self.biome_sampling_noise.get_noise_3d(world_x, world_y, 250.0) + 1.0 ) * 50.0 );


            let humidity_point = humidity_points[i];
            let world_x = (humidity_point.x + chunk_world_pos.x) as f32;
            let world_y = (humidity_point.y + chunk_world_pos.y) as f32;
            sampled_humidity[i] = (humidity_point, (self.biome_sampling_noise.get_noise_3d(world_x, world_y, 250.0) + 1.0 ) * 50.0 );
        }

        (sampled_temperature, sampled_humidity)
    }
}

fn get_biome_points(chunk_seed: u64) -> ([IVec2; BIOME_SAMPLE_POINT_AMOUNT], [IVec2; BIOME_SAMPLE_POINT_AMOUNT]){
    let empty_ivec = IVec2 { x: 0, y: 0};
    let mut temperature_points: [IVec2; BIOME_SAMPLE_POINT_AMOUNT] = [empty_ivec; BIOME_SAMPLE_POINT_AMOUNT];
    let mut humidity_points: [IVec2; BIOME_SAMPLE_POINT_AMOUNT] = [empty_ivec; BIOME_SAMPLE_POINT_AMOUNT];
    let mut rng = Rng::with_seed(chunk_seed);
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

fn generate_temperature_and_humidity_map(chunk_world_pos: &IVec2, sampled_temperature: [(IVec2, f32); BIOME_SAMPLE_POINT_AMOUNT], sampled_humidity: [(IVec2, f32); BIOME_SAMPLE_POINT_AMOUNT], dimension_schema: &ThreadlocalDimensionSchema)
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