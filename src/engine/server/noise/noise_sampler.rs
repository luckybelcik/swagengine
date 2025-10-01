#[cfg(not(feature = "gpu-server"))]
pub type ActiveNoise = crate::engine::server::noise::cpu_noise::CPUNoise;

#[cfg(feature = "gpu-server")]
pub type ActiveNoise = crate::engine::server::noise::gpu_noise::GPUNoise;

pub struct NoiseSampler {
    noise: ActiveNoise,
}