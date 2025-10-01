use glam::UVec2;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct DimensionSchema {
    pub name: String,
    pub size: UVec2,
    pub biome_map_adjustments: Option<BiomeMapAdjustments>,
}

#[derive(Deserialize, Clone)]
pub struct BiomeMapAdjustments {
    pub horizontal_temperature_variation: u8,
    pub vertical_temperature_variation: u8,
}

#[derive(Deserialize)]
pub struct BiomeSchema {
    pub biome_config: BiomeConfig,
    pub noise_functions: NoiseFunctions,
}

#[derive(Deserialize)]
pub struct BiomeConfig {
    pub temperature: u8,
    pub humidity: u8,
    pub biome_type: BiomeTypes,
    pub surface_block: u32,
    pub subsurface_block: u32,
    pub base_block: u32,
}

#[derive(Deserialize, PartialEq)]
pub enum BiomeTypes {
    Hot,
    Warm,
    Neutral,
    Cold,
    Freezing,
}

#[derive(Deserialize)]
pub struct NoiseFunctions {
    pub continental: NoiseConfig,
    pub mountainous: NoiseConfig,
    pub hilly: NoiseConfig,
    pub texture: NoiseConfig,
    pub cellular: NoiseConfig,
    pub gridlike: NoiseConfig,
}

#[derive(Deserialize, Clone, Copy)]
pub struct NoiseConfig {
    pub amplitude: f32,
    pub weight: f32,
    pub blending_mode: BlendingMode,
}

#[derive(Deserialize, Clone, Copy)]
pub enum BlendingMode {
    Mix,
    MixPositive,
    MixNegative,
    Add,
    Subtract,
    Multiply,
}