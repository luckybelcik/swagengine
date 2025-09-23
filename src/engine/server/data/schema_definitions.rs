#[derive(Deserialize)]
pub struct Dimension {
    pub name: String,
    pub size: UVec2,
}

#[derive(Deserialize)]
pub struct Biome {
    pub noise_functions: Vec<NoiseConfig>,
}

#[derive(Deserialize)]
pub struct NoiseConfig {
    pub frequency: f32,
    pub amplitude: f32,
    pub noise_type: NoiseTypes,
    pub fbm: FbmOption,
    pub blending_mode: BlendingMode,
    pub sparse: SparseOption,
}

#[derive(Deserialize)]
pub enum NoiseTypes {
    CellDistance,
    CellDistanceSq,
    CellValue,
    OpenSimplex2,
    Perlin,
    Simplex,
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
#[serde(untagged)]
pub enum FbmOption {
    Disabled(bool),
    Enabled(FbmConfig),
}

#[derive(Deserialize)]
pub struct SparseConfig {
    pub sparse_factor: u8,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum SparseOption {
    Disabled(bool),
    Enabled(SparseConfig),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum BlendingMode {
    Add,
    Subtract,
    Multiply,
    Divide,
}