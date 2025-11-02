use crate::engine::server::chunk_generator::ThreadlocalDimensionSchema;
use crate::engine::server::common::{world_to_chunk_pos, world_to_chunk_pos_2d, world_to_local_pos, world_to_local_pos_2d};
use crate::engine::server::constants::{CHUNK_SIZE, HUMIDITY_INDEX, TEMPERATURE_INDEX};
#[cfg(not(feature = "gpu-server"))]
use crate::engine::server::constants::{NUM_1D_NOISE_LAYERS, NUM_2D_NOISE_LAYERS};
#[cfg(not(feature = "gpu-server"))]
use crate::engine::server::noise::common::NoiseLayer1D;
#[cfg(feature = "gpu-server")]
use crate::engine::server::noise::gpu_noise;

use dashmap::DashMap;
use crate::engine::server::noise::common::NoiseLayer2D;
use glam::IVec2;
use crate::engine::server::{noise::cpu_noise::CPUNoise};
use crate::engine::server::{constants::CHUNK_BLOCK_COUNT};

#[cfg(feature = "gpu-server")]
use crate::engine::server::noise::{gpu_noise::GPUNoise};

#[cfg(feature = "gpu-server")]
pub struct NoiseSampler {
    cpu_noise: CPUNoise,
    gpu_noise: GPUNoise,
    cache_1d: [DashMap<i32, NoiseLayer1D>; NUM_1D_NOISE_LAYERS],
    cache_2d: [DashMap<IVec2, NoiseLayer2D>; NUM_2D_NOISE_LAYERS], 
}

#[cfg(not(feature = "gpu-server"))]
pub struct NoiseSampler {
    cpu_noise: CPUNoise,
    cache_1d: [DashMap<i32, NoiseLayer1D>; NUM_1D_NOISE_LAYERS],
    cache_2d: [DashMap<IVec2, NoiseLayer2D>; NUM_2D_NOISE_LAYERS], 
}

impl NoiseSampler {
    pub async fn new(dimension_seed: i32, dimension_schema: ThreadlocalDimensionSchema) -> NoiseSampler {
        #[cfg(feature = "gpu-server")]
        let gpu_noise = GPUNoise::new().await;

        let cpu_noise = CPUNoise::new(dimension_seed, dimension_schema);

        #[cfg(feature = "gpu-server")]
        return NoiseSampler {
            cpu_noise,
            gpu_noise,
            cache_1d: std::array::from_fn(|_| DashMap::new()),
            cache_2d: std::array::from_fn(|_| DashMap::new()),
        };

        #[cfg(not(feature = "gpu-server"))]
        return NoiseSampler {
            cpu_noise,
            cache_1d: std::array::from_fn(|_| DashMap::new()),
            cache_2d: std::array::from_fn(|_| DashMap::new()),
        };
    }
    
    pub fn get_noise_1d(&self, world_pos_x: i32, noise_layer_index: usize) -> f32 {
        let chunk_relative_pos_x = world_to_local_pos(world_pos_x);
        let chunk_pos_x = world_to_chunk_pos(world_pos_x);

        let layer = *self.cache_1d[noise_layer_index].entry(chunk_pos_x).or_insert_with(|| {
            let mut layer_1d = NoiseLayer1D::new();
            for x in 0..CHUNK_SIZE as i32 {
                let world_pox_x_loop = x + chunk_pos_x * CHUNK_SIZE as i32;
                let value = self.cpu_noise.get_noise_layer_by_index(noise_layer_index).get_noise_2d(world_pox_x_loop as f32, 0.0);
                layer_1d.write(x, value);
            }

            layer_1d
        });

        return layer.read(chunk_relative_pos_x);
    }

    pub fn get_noise_layer_1d(&self, chunk_pos_x: i32, noise_layer_index: usize) -> NoiseLayer1D {
        let layer = *self.cache_1d[noise_layer_index].entry(chunk_pos_x).or_insert_with(|| {
            let mut layer_1d = NoiseLayer1D::new();
            for x in 0..CHUNK_SIZE as i32 {
                let world_pox_x_loop = x + chunk_pos_x * CHUNK_SIZE as i32;
                let value = self.cpu_noise.get_noise_layer_by_index(noise_layer_index).get_noise_2d(world_pox_x_loop as f32, 0.0);
                layer_1d.write(x, value);
            }

            layer_1d
        });

        return layer
    }

    pub fn get_noise_2d(&self, world_pos: &IVec2, noise_layer_index: usize) -> u8 {
        let chunk_relative_pos = world_to_local_pos_2d(*world_pos);
        let chunk_pos = world_to_chunk_pos_2d(*world_pos);

        let layer = *self.cache_2d[noise_layer_index].entry(chunk_pos).or_insert_with(|| {
            let (temperature, humidity) = self.cpu_noise.get_temperature_and_humidity_map(&chunk_pos);
            let temperature = NoiseLayer2D {layer: temperature};
            let humidity = NoiseLayer2D {layer: humidity};
            let return_value = match noise_layer_index {
                TEMPERATURE_INDEX => {temperature}
                HUMIDITY_INDEX => {humidity}
                _ => {panic!("2D Layer index not found")}
            };

            self.cache_2d[TEMPERATURE_INDEX].insert(chunk_pos, temperature);
            self.cache_2d[HUMIDITY_INDEX].insert(chunk_pos, humidity);

            return_value
        });

        layer.read(chunk_relative_pos)
    }

    pub fn get_noise_layer_2d(&self, chunk_pos: &IVec2, noise_layer_index: usize) -> NoiseLayer2D {
        let layer = match self.cache_2d[noise_layer_index].remove(&chunk_pos) {
            // key found and removed
            Some((_key, value)) => {
                value
            }

            // key not found, we just create the whole thing ourselves
            None => {
                let (temperature, humidity) = self.cpu_noise.get_temperature_and_humidity_map(&chunk_pos);
                let temperature = NoiseLayer2D {layer: temperature};
                let humidity = NoiseLayer2D {layer: humidity};
                let return_value = match noise_layer_index {
                    TEMPERATURE_INDEX => {temperature}
                    HUMIDITY_INDEX => {humidity}
                    _ => {panic!("2D Layer index not found")}
                };

                return_value
            }
        };

        layer
    }

    fn get_temperature_and_humidity_map(&self, chunk_pos: &IVec2)
    -> ([u8; CHUNK_BLOCK_COUNT as usize], [u8; CHUNK_BLOCK_COUNT as usize]) {
        self.cpu_noise.get_temperature_and_humidity_map(chunk_pos)
    }
}