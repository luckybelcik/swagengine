#[cfg(not(feature = "gpu-server"))]
pub type ActiveNoise = crate::engine::server::noise::cpu_noise::CPUNoise;

#[cfg(feature = "gpu-server")]
pub type ActiveNoise = crate::engine::server::noise::gpu_noise::GPUNoise;

pub struct NoiseSampler {
    noise: ActiveNoise,
}

impl NoiseSampler {
    pub async fn new() -> NoiseSampler {
        #[cfg(feature = "gpu-server")]
        let noise = ActiveNoise::new().await;

        #[cfg(not(feature = "gpu-server"))]
        let noise = ActiveNoise::new();

        NoiseSampler {
            noise,
        }
    }
}