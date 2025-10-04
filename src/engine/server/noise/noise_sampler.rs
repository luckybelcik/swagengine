use std::sync::Arc;

use glam::IVec2;

use crate::engine::server::{constants::CHUNK_BLOCK_COUNT, data::schema_definitions::DimensionSchema, noise::cpu_noise::CPUNoise};

#[cfg(feature = "gpu-server")]
use crate::engine::server::noise::{gpu_noise::GPUNoise};

#[cfg(feature = "gpu-server")]
pub struct NoiseSampler {
    cpu_noise: CPUNoise,
    gpu_noise: GPUNoise,
}

#[cfg(not(feature = "gpu-server"))]
pub struct NoiseSampler {
    cpu_noise: CPUNoise,
}

impl NoiseSampler {
    pub async fn new(dimension_seed: i32) -> NoiseSampler {
        #[cfg(feature = "gpu-server")]
        let gpu_noise = GPUNoise::new().await;

        let cpu_noise = CPUNoise::new(dimension_seed);

        #[cfg(feature = "gpu-server")]
        return NoiseSampler {
            cpu_noise,
            gpu_noise,
        };

        #[cfg(not(feature = "gpu-server"))]
        return NoiseSampler {
            cpu_noise,
        };
    }

    pub fn get_temperature_and_humidity_map(&self, chunk_pos: &IVec2, world_seed: i32, dimension_schema: &Arc<DimensionSchema>)
    -> ([u8; CHUNK_BLOCK_COUNT as usize], [u8; CHUNK_BLOCK_COUNT as usize]) {
        self.cpu_noise.get_temperature_and_humidity_map(chunk_pos, world_seed, dimension_schema)
    }

    pub fn get_noise_2d(x: f32, y: f32, noise_map_index: usize) -> f32 {
        
    }
}