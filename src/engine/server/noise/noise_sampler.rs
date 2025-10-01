use crate::engine::server::noise::cpu_noise::{self, CPUNoise};

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
    pub async fn new() -> NoiseSampler {
        #[cfg(feature = "gpu-server")]
        let gpu_noise = GPUNoise::new().await;

        let cpu_noise = CPUNoise::new();

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
}