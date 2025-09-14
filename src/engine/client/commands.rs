use crate::engine::command_registry::{error_not_enough_arguments, error_wrong_type, CommandDependency, CommandEnvironment, DebugCommand};

pub fn create_client_commands() -> Vec<DebugCommand> {
    let mut commands = Vec::new();

    commands.push(DebugCommand {
        name: "framerate",
        aliases: &["fps"],
        description: "Prints the average framerate. Sampled since program start, or last resetfps execution.",
        execute: |dependency, _args| {
            if let CommandDependency::Client(client) = dependency {
                println!("Average FPS: {:.2}", client.time.average_fps());
            }
        },
        command_environment: CommandEnvironment::Client,
    });

    commands.push(DebugCommand {
        name: "setfps",
        aliases: &["setfps", "setframerate"],
        description: "Sets the target framerate.",
        execute: |dependency, _args| {
            if let CommandDependency::Client(client) = dependency {
                if let Some(arg) = _args.first() {
                    if let Ok(new_cap) = arg.parse::<u32>() {
                        client.client_config.frame_cap = new_cap;
                        println!("Frame cap set to {new_cap}");
                    } else {
                        error_wrong_type();
                    }
                } else {
                    error_not_enough_arguments();
                }
            }
        },
        command_environment: CommandEnvironment::Client,
    });

    commands.push(DebugCommand {
        name: "resetfps",
        aliases: &["rfps"],
        description: "Resets the average framerate sample frames.",
        execute: |dependency, _args| {
            if let CommandDependency::Client(client) = dependency {
                client.time.reset_average_fps();
                println!("Reset average framerate sample frames")
            }
        },
        command_environment: CommandEnvironment::Client,
    });

    commands.push(DebugCommand {
        name: "togglevsync",
        aliases: &["tvsync"],
        description: "Toggles VSync.",
        execute: |dependency, _args| {
            if let CommandDependency::Client(client) = dependency {
                client.client_config.vsync = !client.client_config.vsync;
                let bool = client.client_config.vsync;
                println!("VSync toggled to {bool}");
            }
        },
        command_environment: CommandEnvironment::Client,
    });

    return commands;
}