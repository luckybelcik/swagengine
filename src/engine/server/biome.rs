use fastnoise_lite::{FastNoiseLite, NoiseType};

use crate::engine::server::{constants::BIOME_MAP_GRID_SIZE, data::schema_definitions::{BiomeConfig, BiomeSchema, NoiseConfig, NoiseTypes}};

pub struct BiomeRegistry {
    pub biomes: Box<[Biome]>,
    pub biome_map: BiomeMap<'static>,
}

impl BiomeRegistry {
    pub fn new(biome_schemas: Vec<BiomeSchema>, seed: i32) -> Self {
        let biomes_vec: Vec<Biome> = biome_schemas.into_iter()
            .map(|s| Biome::from_schema(s, seed))
            .collect();

        let biomes_box = biomes_vec.into_boxed_slice();

        let biome_map = unsafe {
            BiomeMap::populate_biome_map(std::slice::from_raw_parts(
                biomes_box.as_ptr(),
                biomes_box.len()
            ))
        };

        Self {
            biomes: biomes_box,
            biome_map,
        }
    }
}

pub struct BiomeMap<'a> {
    pub map: [&'a Biome; 10000],
}

impl<'a> BiomeMap<'a> {
    pub fn populate_biome_map(loaded_biomes: &'a [Biome]) -> BiomeMap<'a> {
        let default_biome = loaded_biomes.get(0).expect("loaded_biomes cannot be empty");
        let mut biome_lookup: [&'a Biome; (BIOME_MAP_GRID_SIZE * BIOME_MAP_GRID_SIZE) as usize] =
            [default_biome; (BIOME_MAP_GRID_SIZE * BIOME_MAP_GRID_SIZE) as usize];

        for x in 0..BIOME_MAP_GRID_SIZE {
            for y in 0..BIOME_MAP_GRID_SIZE {
                let mut closest_biome: Option<&'a Biome> = None;
                let mut min_distance_sq = f64::MAX;

                for biome in loaded_biomes.iter() {
                    let dx = biome.biome_config.temperature as f64 - x as f64;
                    let dy = biome.biome_config.humidity as f64 - y as f64;
                    let distance_sq = dx * dx + dy * dy;

                    if distance_sq < min_distance_sq {
                        min_distance_sq = distance_sq;
                        closest_biome = Some(biome);
                    }
                }

                if let Some(biome) = closest_biome {
                    let index = (y * BIOME_MAP_GRID_SIZE + x) as usize;
                    biome_lookup[index] = biome;
                }
            }
        }

        BiomeMap { map: biome_lookup }
    }

    pub fn get_biome(&self, temperature: u8, humidity: u8) -> &'a Biome {
        let x = temperature.clamp(0, BIOME_MAP_GRID_SIZE as u8 - 1) as usize;
        let y = humidity.clamp(0, BIOME_MAP_GRID_SIZE as u8 - 1) as usize;

        let index = y * BIOME_MAP_GRID_SIZE as usize + x;
        
        self.map[index]
    }
}

pub struct Biome {
    pub biome_config: BiomeConfig,
    pub noise_schema: Vec<NoiseConfig>,
    pub noise_generators: Vec<FastNoiseLite>,
}

impl Biome {
    pub fn from_schema(schema: BiomeSchema, seed: i32) -> Self {
        let mut generators: Vec<FastNoiseLite> = Vec::new(); 
        for noise_fn in &schema.noise_functions {
            let mut generator = FastNoiseLite::new();
            match noise_fn.noise_type {
                NoiseTypes::Cellular => generator.set_noise_type(Some(NoiseType::Cellular)),
                NoiseTypes::OpenSimplex2 => generator.set_noise_type(Some(NoiseType::OpenSimplex2)),
                NoiseTypes::Perlin => generator.set_noise_type(Some(NoiseType::Perlin)),
                NoiseTypes::Value => generator.set_noise_type(Some(NoiseType::Value)),
                NoiseTypes::ValueCubic => generator.set_noise_type(Some(NoiseType::ValueCubic)),
            };

            generator.set_seed(Some(seed));
            generator.set_frequency(Some(noise_fn.frequency));
           
            match &noise_fn.fbm {
                Some(config) => {
                    generator.set_fractal_octaves(Some(config.octaves as i32));
                    generator.set_fractal_gain(Some(config.gain));
                    generator.set_fractal_lacunarity(Some(config.lacunarity));
                },
                _ => (),
            };

            generators.push(generator);
        }
        Biome {
            biome_config: schema.biome_config,
            noise_schema: schema.noise_functions,
            noise_generators: generators,
        }
    }
}