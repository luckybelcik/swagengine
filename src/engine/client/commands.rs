use crate::engine::command_registry::{error_not_enough_arguments, error_wrong_type, DebugCommand};

pub fn create_client_commands() -> Vec<DebugCommand> {
    let mut commands = Vec::new();

    commands.push(DebugCommand {
        name: "framerate",
        aliases: &["fps"],
        description: "Prints the average framerate. Sampled since program start, or last resetfps execution.",
        execute: |_app, _args| {
            println!("Average FPS: {:.2}", _app.time.average_fps());
        },
    });

    commands.push(DebugCommand {
        name: "setfps",
        aliases: &["setfps", "setframerate"],
        description: "Sets the target framerate.",
        execute: |_app, _args| {
            if let Some(arg) = _args.first() {
                if let Ok(new_cap) = arg.parse::<u32>() {
                    _app.app_config.frame_cap = new_cap;
                    println!("Frame cap set to {new_cap}");
                } else {
                    error_wrong_type();
                }
            } else {
                error_not_enough_arguments();
            }      
        },
    });

    commands.push(DebugCommand {
        name: "resetfps",
        aliases: &["rfps"],
        description: "Resets the average framerate sample frames.",
        execute: |_app, _args| {
            _app.time.reset_average_fps();
            println!("Reset average framerate sample frames")
        },
    });

    commands.push(DebugCommand {
        name: "togglevsync",
        aliases: &["tvsync"],
        description: "Toggles VSync.",
        execute: |app, _args| {
            app.app_config.vsync = !app.app_config.vsync;
            let bool = app.app_config.vsync;
            println!("VSync toggled to {bool}");
        }
    });

    return commands;
}