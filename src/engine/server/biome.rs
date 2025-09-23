use noise_functions::{CellDistance, CellValue, Noise, OpenSimplex2, Perlin, Sample, Simplex, Value, ValueCubic};

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
    pub noise_generator: Vec<Box<dyn Sample<2>>>,
}

impl Biome {
    pub fn from_schema(schema: BiomeSchema, seed: i32) -> Self {
        let mut noise_generator: Vec<Box<dyn Sample<2> + 'static>> = Vec::new();
        for noise_fn in &schema.noise_functions {
            let generator: Box<dyn Sample<2> + 'static> = match noise_fn.noise_type {
                NoiseTypes::CellDistance => Box::new(CellDistance::default().seed(seed)) as Box<dyn Sample<2> + 'static>,
                NoiseTypes::CellValue => Box::new(CellValue::default().seed(seed)) as Box<dyn Sample<2> + 'static>,
                NoiseTypes::OpenSimplex2 => Box::new(OpenSimplex2.seed(seed)) as Box<dyn Sample<2> + 'static>,
                NoiseTypes::Perlin => Box::new(Perlin.seed(seed)) as Box<dyn Sample<2> + 'static>,
                NoiseTypes::Simplex => Box::new(Simplex.seed(seed)) as Box<dyn Sample<2> + 'static>,
                NoiseTypes::Value => Box::new(Value.seed(seed)) as Box<dyn Sample<2> + 'static>,
                NoiseTypes::ValueCubic => Box::new(ValueCubic.seed(seed)) as Box<dyn Sample<2> + 'static>,
            };
            let generator = Box::new(generator.frequency(noise_fn.frequency).mul(noise_fn.amplitude)) as Box<dyn Sample<2> + 'static>;
            let generator = match &noise_fn.fbm {
                Some(config) => Box::new(generator.fbm(config.octaves, config.gain, config.lacunarity)) as Box<dyn Sample<2> + 'static>,
                None => generator,
            };
            noise_generator.push(generator);
        }
        Biome {
            biome_config: schema.biome_config,
            noise_schema: schema.noise_functions,
            noise_generator,
        }
    }
}