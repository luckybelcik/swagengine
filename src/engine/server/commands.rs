use crate::{engine::{command_registry::{error_dimension_not_found, error_not_enough_arguments, error_wrong_type, CommandDependency, CommandEnvironment, DebugCommand}, server::constants::CHUNK_BLOCK_COUNT}};

pub fn create_server_commands() -> Vec<DebugCommand> {
    let mut commands = Vec::new();

    commands.push(DebugCommand {
        name: "stopserver",
        aliases: &["exitserver"],
        description: "Stops the server.",
        execute: |dependency, _args| {
            if let CommandDependency::Server(server) = dependency {
                server.stop();
            }
        },
        command_environment: CommandEnvironment::Server,
    });

    commands.push(DebugCommand {
        name: "dimensions",
        aliases: &["dims"],
        description: "Returns all dimension names.",
        execute: |dependency, _args| {
            if let CommandDependency::Server(server) = dependency {
                let keys = server.get_dimension_keys();
                for key in keys {
                    println!("{}", key);
                }
            }
        },
        command_environment: CommandEnvironment::Server,
    });

    commands.push(DebugCommand {
        name: "testchunkspeed",
        aliases: &["tcs"],
        description: "Generates chunks for 5 seconds then returns the count.",
        execute: |dependency, _args| {
            if let CommandDependency::Server(server) = dependency {  
                let Some(dimension_arg) = _args.first() else {
                    error_not_enough_arguments();
                    return;
                };

                let Some(dimension) = server.get_dimension(dimension_arg) else {
                    error_dimension_not_found();
                    return;
                };

                let mut chunk_limit: u32 = 10000;
                if let Some(arg) = _args.get(1) {
                    match arg.parse::<u32>() {
                        Ok(limit) => chunk_limit = limit,
                        Err(_) => {
                            error_wrong_type();
                            return;
                        }
                    }
                } else {
                    println!("No limit provided, defaulting to {chunk_limit}");
                }

                let mut test_count: u32 = 1;
                if let Some(arg) = _args.get(2) {
                    match arg.parse::<u32>() {
                        Ok(count) => test_count = if count == 0 { 1 } else { count },
                        Err(_) => {
                            error_wrong_type();
                            return;
                        }
                    }
                } else {
                    println!("No test loop count provided, defaulting to {test_count}");
                }

                let mut total_nanos: u128 = 0;

                for _ in 0..test_count {
                    let duration: std::time::Duration = dimension.chunk_load_speed_test(chunk_limit);
                    total_nanos += duration.as_nanos();
                }

                let nanos_per_test = total_nanos / test_count as u128;
                let millis_per_test = nanos_per_test / 1000000;

                println!("Generated {chunk_limit} chunks in {millis_per_test} millis ({nanos_per_test} nanoseconds) ({test_count} tests averaged)");

                let avg_per_chunk = millis_per_test as f64 / chunk_limit as f64;
                let avg_per_block = (nanos_per_test as f64 / chunk_limit as f64) / CHUNK_BLOCK_COUNT as f64;

                println!("Thats {avg_per_chunk}ms per chunk, {avg_per_block}ns per block ({CHUNK_BLOCK_COUNT} blocks in chunk)");
            }
        },
        command_environment: CommandEnvironment::Server,
    });

    return commands;
}