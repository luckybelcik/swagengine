use std::{collections::HashSet, sync::mpsc::{Receiver, Sender}, time::{Duration, Instant}};

use glam::IVec2;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

use crate::engine::server::{biome::BiomeRegistry, chunk::Chunk, constants::CHUNK_BLOCK_COUNT};

pub struct ChunkGenerator {
    chunks_awaiting_generation: HashSet<IVec2>,
    chunkpos_sender: Sender<Generate>,
}

impl ChunkGenerator {
    pub fn new(biome_registry: BiomeRegistry) -> (ChunkGenerator, Receiver<Chunk>) {
        let (generator_sender, generator_listener) = std::sync::mpsc::channel::<Chunk>();
        let (chunkpos_sender, chunkpos_listener) = std::sync::mpsc::channel::<Generate>();

        std::thread::spawn(move || {
            println!("Chunk generator thread spawned");
            const PARALLEL_THRESHOLD: usize = 4;
            let biome_map = biome_registry.biome_map;

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

                        let generated_chunks: Vec<Chunk> = if batch_size >= PARALLEL_THRESHOLD {
                            println!("Processing large batch of {} chunks in parallel.", batch_size);
                            batch.par_iter().map(|&coords| Chunk::generate_chunk(&coords, &biome_map)).collect()
                        } else {
                            println!("Processing small batch of {} chunks sequentially.", batch_size);
                            batch.into_iter().map(|coords| Chunk::generate_chunk(&coords, &biome_map)).collect()
                        };

                        for chunk in generated_chunks {
                            let _ = generator_sender.send(chunk);
                        }
                    },

                    Generate::Test(chunk_limit) => {
                        let start_time = Instant::now();
                        let mut chunks_generated = 0;
                        let mut chunk_pos = IVec2::new(0, 0);

                        while chunks_generated < chunk_limit {
                            let _: Chunk = Chunk::generate_chunk(&chunk_pos, &biome_map); 
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
        self.chunks_awaiting_generation.insert(chunk_pos.clone());
        let _ = self.chunkpos_sender.send(Generate::Chunk(*chunk_pos));
    }

    pub fn run_test(&self, chunk_limit: u32) {
        let _ = self.chunkpos_sender.send(Generate::Test(chunk_limit));
    }
}

enum Generate {
    Chunk(IVec2),
    Test(u32)
}