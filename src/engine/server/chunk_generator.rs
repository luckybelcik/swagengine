use std::{collections::HashSet, sync::{mpsc::{Receiver, Sender}, Arc, RwLock}, time::{Instant}};

use glam::IVec2;
use rayon::iter::{IntoParallelRefIterator};
use rayon::iter::ParallelIterator;

use crate::engine::server::{biome::BiomeRegistry, chunk::Chunk, constants::{CHUNK_BLOCK_COUNT, GPU_CHUNKGEN_THRESHOLD, PARALLEL_CHUNKGEN_THRESHOLD}, data::schema_definitions::DimensionSchema, noise::noise_sampler::NoiseSampler};

pub type ThreadlocalDimensionSchema = Arc<DimensionSchema>;

pub struct ChunkGenerator {
    chunks_awaiting_generation: HashSet<IVec2>,
    chunkpos_sender: Sender<Generate>,
}

impl ChunkGenerator {
    pub fn new(biome_registry: BiomeRegistry, dimension_schema: DimensionSchema, dimension_seed: i32) -> (ChunkGenerator, Receiver<(Chunk, IVec2)>) {
        let (generator_sender, generator_listener) = std::sync::mpsc::channel::<(Chunk, IVec2)>();
        let (chunkpos_sender, chunkpos_listener) = std::sync::mpsc::channel::<Generate>();

        let arc_registry = Arc::new(RwLock::new(biome_registry));
        let thread_registry = arc_registry.clone();

        let arc_dimension = Arc::new(dimension_schema);
        let thread_dimension = arc_dimension.clone();

        let arc_noise_sampler = Arc::new(pollster::block_on(NoiseSampler::new(dimension_seed, thread_dimension)));
        let thread_noise_sampler = arc_noise_sampler.clone();

        std::thread::spawn(move || {
            println!("Chunk generator thread spawned");

            loop {
                let command = match chunkpos_listener.recv() {
                    Ok(cmd) => cmd,
                    Err(_) => {
                        println!("Server listener disconnected. Generator shutting down.");
                        break;
                    }
                };

                match command {
                    Generate::Chunk(first_coords) => {
                        let mut batch: Vec<IVec2> = vec![first_coords];
                        while let Ok(Generate::Chunk(coords)) = chunkpos_listener.try_recv() {
                            batch.push(coords);
                        }

                        let batch_size = batch.len();

                        let generated_chunks: Vec<(IVec2, Chunk)> = {
                            let mut chunks: Vec<(IVec2, Chunk)> = Vec::new();

                            let registry_guard = thread_registry.read().unwrap();
                            let biome_map = &registry_guard.biome_map; 

                            // Use GPU generation if enabled
                            #[cfg(feature = "gpu-server")]
                            if batch_size >= GPU_CHUNKGEN_THRESHOLD {
                                println!("Processing huge batch of {} chunks on the GPU.", batch_size);
                                todo!("GPU Noise generation not implemented yet!");
                            }

                            // Use multithreaded generation is batch size big enough
                            if batch_size >= PARALLEL_CHUNKGEN_THRESHOLD {
                                println!("Processing large batch of {} chunks in parallel.", batch_size);
                                chunks = batch.par_iter()
                                    .map(|&coords| (coords, Chunk::generate_chunk(&coords, biome_map, &thread_noise_sampler, dimension_seed))).collect();
                            }

                            // Fallback - simple sequential generation
                            if batch_size < PARALLEL_CHUNKGEN_THRESHOLD {
                                println!("Processing small batch of {} chunks sequentially.", batch_size);
                                chunks = batch.into_iter()
                                    .map(|coords| (coords, Chunk::generate_chunk(&coords, &biome_map, &thread_noise_sampler, dimension_seed))).collect();
                            }

                            chunks
                        };

                        for (pos, chunk) in generated_chunks {
                            let _ = generator_sender.send((chunk, pos));
                        }
                    },

                    Generate::Test(chunk_limit) => {
                        let start_time = Instant::now();
                        let mut chunks_generated = 0;
                        let mut chunk_pos = IVec2::new(0, 0);
                        let registry_guard = thread_registry.read().unwrap();
                        let biome_map = &registry_guard.biome_map; 

                        while chunks_generated < chunk_limit {
                            let _: Chunk = Chunk::generate_chunk(&chunk_pos, &biome_map, &thread_noise_sampler, dimension_seed); 
                            chunks_generated += 1;

                            chunk_pos.x += 1;
                            if chunk_pos.x >= 100 {
                                chunk_pos.x = 0;
                                chunk_pos.y += 1;
                            }
                        }

                        let elapsed = start_time.elapsed();
                        println!("Chunk generation test finished. {} chunks generated in {:?}", chunk_limit, elapsed);

                        let total_nanos = elapsed.as_nanos();
                        let total_millis = total_nanos / 1000000;

                        println!("Generated {chunk_limit} chunks in {total_millis} millis ({total_nanos} nanoseconds)");

                        let avg_per_chunk = total_millis as f64 / chunk_limit as f64;
                        let avg_per_block = (total_nanos as f64 / chunk_limit as f64) / CHUNK_BLOCK_COUNT as f64;

                        println!("Thats {avg_per_chunk}ms per chunk, {avg_per_block}ns per block ({CHUNK_BLOCK_COUNT} blocks in chunk)");
                    }
                }
            }
        });

        return (ChunkGenerator {
            chunks_awaiting_generation: HashSet::new(),
            chunkpos_sender,

        }, generator_listener)
    }

    pub fn load_chunk(&mut self, chunk_pos: &IVec2) {
        // If chunk not already scheduled for generation, generate it
        if self.chunks_awaiting_generation.insert(chunk_pos.clone()) {
            let _ = self.chunkpos_sender.send(Generate::Chunk(*chunk_pos));
        }
    }

    pub fn run_test(&self, chunk_limit: u32) {
        let _ = self.chunkpos_sender.send(Generate::Test(chunk_limit));
    }
}

enum Generate {
    Chunk(IVec2),
    Test(u32)
}