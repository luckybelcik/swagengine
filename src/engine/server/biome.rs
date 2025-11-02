use crate::engine::server::{constants::{BIOME_MAP_GRID_SIZE, CELLULAR_NINDEX, CONTINENTAL_NINDEX, GRIDLIKE_NINDEX, HILLY_NINDEX, MOUNTAINOUS_NINDEX, NUM_1D_NOISE_LAYERS, TEXTURE_NINDEX}, data::schema_definitions::{BiomeConfig, BiomeSchema, NoiseConfig}};

pub struct BiomeRegistry {
    pub biomes: Box<[Biome]>,
    pub biome_map: BiomeMap<'static>,
}

impl BiomeRegistry {
    pub fn new(biome_schemas: Vec<BiomeSchema>) -> Self {
        let biomes_vec: Vec<Biome> = biome_schemas.into_iter()
            .map(|schema| Biome::from_schema(schema))
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
    pub map: [(&'a Biome, &'a Biome); 10000],
}

impl<'a> BiomeMap<'a> {
    pub fn populate_biome_map(loaded_biomes: &'a [Biome]) -> BiomeMap<'a> {
        let default_biome = loaded_biomes.get(0).expect("loaded_biomes cannot be empty");
        let mut biome_lookup: [(&'a Biome, &'a Biome); (BIOME_MAP_GRID_SIZE * BIOME_MAP_GRID_SIZE) as usize] =
            [(default_biome, default_biome); (BIOME_MAP_GRID_SIZE * BIOME_MAP_GRID_SIZE) as usize];

        for x in 0..BIOME_MAP_GRID_SIZE {
            for y in 0..BIOME_MAP_GRID_SIZE {
                let mut closest_biome: &'a Biome = default_biome;
                let mut second_closest_biome: &'a Biome = default_biome;
                let mut min_distance_sq = f64::MAX;
                let mut second_min_distance_sq = f64::MAX;

                for biome in loaded_biomes.iter() {
                    let dx = biome.biome_config.temperature as f64 - x as f64;
                    let dy = biome.biome_config.humidity as f64 - y as f64;
                    let distance_sq = dx * dx + dy * dy;

                    if distance_sq < min_distance_sq {
                        second_min_distance_sq = min_distance_sq;
                        second_closest_biome = closest_biome;
                        
                        min_distance_sq = distance_sq;
                        closest_biome = biome;
                    } else if distance_sq < second_min_distance_sq {
                        second_min_distance_sq = distance_sq;
                        second_closest_biome = biome;
                    }
                }

                if std::ptr::eq(closest_biome, second_closest_biome) && loaded_biomes.len() > 1 {
                    second_min_distance_sq = f64::MAX;
                    for biome in loaded_biomes.iter() {
                        if !std::ptr::eq(closest_biome, biome) {
                            let dx = biome.biome_config.temperature as f64 - x as f64;
                            let dy = biome.biome_config.humidity as f64 - y as f64;
                            let distance_sq = dx * dx + dy * dy;

                            if distance_sq < second_min_distance_sq {
                                second_min_distance_sq = distance_sq;
                                second_closest_biome = biome;
                            }
                        }
                    }
                }

                let index = (y * BIOME_MAP_GRID_SIZE + x) as usize;
            
                biome_lookup[index] = (closest_biome, second_closest_biome);
            }
        }

        BiomeMap { map: biome_lookup }
    }

    pub fn get_best_biome(&self, temperature: u8, humidity: u8) -> &'a Biome {
        let x = temperature.clamp(0, BIOME_MAP_GRID_SIZE as u8 - 1) as usize;
        let y = humidity.clamp(0, BIOME_MAP_GRID_SIZE as u8 - 1) as usize;
    
        let index = y * BIOME_MAP_GRID_SIZE as usize + x;
        
        self.map[index].0
    }

    pub fn get_second_best_biome(&self, temperature: u8, humidity: u8) -> &'a Biome {
        let x = temperature.clamp(0, BIOME_MAP_GRID_SIZE as u8 - 1) as usize;
        let y = humidity.clamp(0, BIOME_MAP_GRID_SIZE as u8 - 1) as usize;
    
        let index = y * BIOME_MAP_GRID_SIZE as usize + x;
        
        self.map[index].1
    }
}

pub struct Biome {
    pub biome_config: BiomeConfig,
    pub noise_schema: [NoiseConfig; 6],
}

impl Biome {
    pub fn from_schema(schema: BiomeSchema) -> Self {
        let empty_config = NoiseConfig { amplitude: 0.0, weight: 0.0, blending_mode: super::data::schema_definitions::BlendingMode::Add };
        let mut noise_schema: [NoiseConfig; NUM_1D_NOISE_LAYERS] = [empty_config; NUM_1D_NOISE_LAYERS];
        noise_schema[CONTINENTAL_NINDEX] = schema.noise_functions.continental;
        noise_schema[MOUNTAINOUS_NINDEX] = schema.noise_functions.mountainous;
        noise_schema[HILLY_NINDEX] = schema.noise_functions.hilly;
        noise_schema[TEXTURE_NINDEX] = schema.noise_functions.texture;
        noise_schema[CELLULAR_NINDEX] = schema.noise_functions.cellular;
        noise_schema[GRIDLIKE_NINDEX] = schema.noise_functions.gridlike;

        Biome {
            biome_config: schema.biome_config,
            noise_schema: noise_schema,
        }
    }
}