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
    pub noise_functions: Vec<NoiseConfig>,
}

#[derive(Deserialize)]
pub struct BiomeConfig {
    pub temperature: u8,
    pub humidity: u8,
    pub surface_block: u32,
    pub subsurface_block: u32,
    pub base_block: u32,
}

#[derive(Deserialize)]
pub struct NoiseConfig {
    pub frequency: f32,
    pub amplitude: f32,
    pub noise_type: NoiseTypes,
    pub fbm: Option<FbmConfig>,
    pub blending_mode: BlendingMode,
    pub sparse: Option<SparseConfig>,
}

#[derive(Deserialize)]
pub enum NoiseTypes {
    Cellular,
    OpenSimplex2,
    Perlin,
    Value,
    ValueCubic,
}

#[derive(Deserialize)]
pub struct FbmConfig {
    pub octaves: u32,
    pub gain: f32,
    pub lacunarity: f32,
}

#[derive(Deserialize)]
pub struct SparseConfig {
    pub sparse_factor: u8,
}

#[derive(Deserialize)]
pub enum BlendingMode {
    Mix,
    MixPositive,
    MixNegative,
    Add,
    Subtract,
    Multiply,
}