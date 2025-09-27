use fastnoise_lite::{FastNoiseLite, NoiseType};

use crate::engine::server::{constants::{BIOME_IDW_POWER, BIOME_MAP_GRID_SIZE, BIOME_TRANSITION_THRESHOLD, MAX_BIOME_BLEND_DISTANCE}, data::schema_definitions::{BiomeConfig, BiomeSchema, NoiseConfig, NoiseTypes}};

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
    pub map: [(&'a Biome, &'a Biome, u8); 10000],
}

impl<'a> BiomeMap<'a> {
    pub fn populate_biome_map(loaded_biomes: &'a [Biome]) -> BiomeMap<'a> {
        let default_biome = loaded_biomes.get(0).expect("loaded_biomes cannot be empty");
        let mut biome_lookup: [(&'a Biome, &'a Biome, u8); (BIOME_MAP_GRID_SIZE * BIOME_MAP_GRID_SIZE) as usize] =
            [(default_biome, default_biome, 100); (BIOME_MAP_GRID_SIZE * BIOME_MAP_GRID_SIZE) as usize];

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

                let d_a = min_distance_sq.sqrt();
                let d_b = second_min_distance_sq.sqrt();
                
                let blend_percentage = if d_a < BIOME_TRANSITION_THRESHOLD && d_b > BIOME_TRANSITION_THRESHOLD {
                    100 
                } else if second_min_distance_sq == f64::MAX || d_a.abs() < 0.001 { 
                    100 
                } else {
                    let inv_influence_a = 1.0 / d_a.powf(BIOME_IDW_POWER).max(f64::EPSILON);
                    let inv_influence_b = 1.0 / d_b.powf(BIOME_IDW_POWER).max(f64::EPSILON);

                    let total_influence = inv_influence_a + inv_influence_b;

                    let ratio_a = inv_influence_a / total_influence;
                    (ratio_a * 100.0).round().clamp(0.0, 100.0) as u8
                };

                let index = (y * BIOME_MAP_GRID_SIZE + x) as usize;

                if x == y {
                    print!("target: {}temp {}hum || ", closest_biome.biome_config.temperature, closest_biome.biome_config.humidity);
                    print!("second best: {}temp {}hum || ", second_closest_biome.biome_config.temperature, second_closest_biome.biome_config.humidity);
                    print!("current: {}temp {}hum || ", x, y);
                    println!("blend factor: {}%", blend_percentage);
                }
            
                biome_lookup[index] = (closest_biome, second_closest_biome, blend_percentage);
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

    pub fn get_blend_percentage(&self, temperature: u8, humidity: u8) -> u8 {
        let x = temperature.clamp(0, BIOME_MAP_GRID_SIZE as u8 - 1) as usize;
        let y = humidity.clamp(0, BIOME_MAP_GRID_SIZE as u8 - 1) as usize;
    
        let index = y * BIOME_MAP_GRID_SIZE as usize + x;
        
        self.map[index].2
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