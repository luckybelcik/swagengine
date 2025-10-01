use crate::engine::{command_registry::{error_dimension_not_found, error_not_enough_arguments, error_wrong_type, CommandDependency, CommandEnvironment, DebugCommand}, common::ServerPacket, server::{constants::CHUNK_BLOCK_COUNT, world::Dimension}};

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
        name: "switchcompressionstate",
        aliases: &["scs"],
        description: "Switches whether the server compresses or does not compress sent data.",
        execute: |dependency, _args| {
            if let CommandDependency::Server(server) = dependency {
                server.compress_sent_data = !server.compress_sent_data;

                println!("Compress sent data: {}", server.compress_sent_data)
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
        name: "resetdimension",
        aliases: &["rdim"],
        description: "Returns all dimension names.",
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

                let mut seed: i32 = 0;
                if let Some(arg) = _args.get(1) {
                    match arg.parse::<i32>() {
                        Ok(seed_in) => seed = seed_in,
                        Err(_) => {
                            error_wrong_type();
                            return;
                        }
                    }
                } else {
                    seed = fastrand::i32(..);
                    println!("No seed provided, using random: {seed}");
                }

                let name = dimension.name.clone();
                server.dimensions.remove(&name);
                let schema = server.get_dimension_schema(&name);

                match schema {
                    Some(dimension_schema) => {
                        let mut new_dimension = Dimension::from_schema(dimension_schema, seed);
                        new_dimension.name = name.clone();
                        server.dimensions.insert(name, new_dimension);
                    }
                    None => println!("No dimension found under the name: {}", &name)
                }
                server.send_packet(ServerPacket::ReloadChunks);
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

                dimension.chunk_load_speed_test(chunk_limit);
            }
        },
        command_environment: CommandEnvironment::Server,
    });

    return commands;
}