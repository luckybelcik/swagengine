use std::collections::HashMap;
use crate::{App};
use sysinfo::{System};

#[derive(Eq, PartialEq, Hash)]
pub struct DebugCommand {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub execute: fn(&mut App, &[&str]),
}

fn create_commands() -> Vec<DebugCommand> {
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
        name: "help",
        aliases: &["h", "m"],
        description: "Prints the help menu.",
        execute: |_app, _args| {
            print_debug_menu(_app);
        }
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

    commands.push(DebugCommand {
        name: "memory",
        aliases: &["mem"],
        description: "Prints program memory usage.",
        execute: |_app, _args| {
            print_memory_usage();
        }
    });

    commands.push(DebugCommand {
        name: "alias",
        aliases: &["a"],
        description: ("Prints the aliases of a given command."),
        execute: |_app, _args| {
            if let Some(arg) = _args.first() {
                if _app.command_registry.contains_key(arg) {
                    print_aliases(_app, arg);
                } else {
                    error_command_not_found();
                }
            } else {
                error_not_enough_arguments();
            }
        }
    });

    commands.push(DebugCommand {
        name: "killprocess",
        aliases: &["kill"],
        description: "Kills the process. Warning - just close the window normally if you can.",
        execute: |_app, _args| {
            std::process::exit(0);
        }
    });

    commands.push(DebugCommand {
        name: "dimensions",
        aliases: &["dims"],
        description: "Returns all dimension names.",
        execute: |_app, _args| {
            let Some(server) = &_app.server else {
            error_server_not_started();
            return;
            };

            let keys = server.get_dimension_keys();
            for key in keys {
                println!("{}", key);
            }
        }
    });

    commands.push(DebugCommand {
        name: "testchunkspeed",
        aliases: &["tcs"],
        description: "Generates chunks for 5 seconds then returns the count.",
        execute: |_app: &mut App, _args: &[&str]| {
            let Some(server) = &mut _app.server else {
                error_server_not_started();
                return;
            };
        
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

            let mut nanos: u128 = 0;

            for _ in 0..test_count {
                let duration: std::time::Duration = dimension.chunk_load_speed_test(chunk_limit);
                nanos += duration.as_nanos();
            }

            nanos = nanos / test_count as u128;
            let millis = nanos / 1000000;

            println!("Generated {chunk_limit} chunks in an average of {millis} millis ({nanos} nanoseconds) ({test_count} tests averaged)");
            
            let avg_per_chunk = millis as f64 / chunk_limit as f64;
            
            println!("Thats {avg_per_chunk}ms per chunk");
        }
    });

    return commands;
}

pub fn build_registry() -> HashMap<&'static str, DebugCommand> {
    let mut mapped = HashMap::new();
    let commands = create_commands();

    for cmd in commands {
        for alias in cmd.aliases {
            mapped.insert(*alias, DebugCommand {
                name: cmd.name,
                aliases: cmd.aliases,
                description: cmd.description,
                execute: cmd.execute,
            });
        }
        mapped.insert(cmd.name, cmd);
    }

    return mapped;
}

pub fn handle_command(app: &mut App, input: &str) {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let cmd_name = parts[0];
    let args = &parts[1..];

    if let Some(command) = app.command_registry.get(cmd_name) {
        (command.execute)(app, args);
    } else {
        println!("Unknown command. Type 'help' for a list.");
    }
}

fn print_aliases(app: &mut App, target_name: &str) {
    let target_command = &app.command_registry.get(target_name).unwrap();
    let aliases = target_command.aliases;

    println!("Available aliases for {target_name}:");
    for string in aliases {
        print!("{string}, ")
    }
    println!("{}", target_command.name);
}

fn print_debug_menu(app: &mut App) {
    println!("Available commands:");
    let mut seen = std::collections::HashSet::new();
    for (_, cmd) in &app.command_registry {
        if seen.insert(cmd.description) {
            println!("- {:<12} {}", cmd.name, cmd.description);
        }
    }
}

fn print_memory_usage() {
    let mut sys = System::new_all();
    sys.refresh_all(); // Refresh system information

    if let Some(process) = sys.process(sysinfo::get_current_pid().expect("Failed to get current PID")) {
        let mem_kb = process.memory(); // Memory in KB (Resident Set Size)
        let virtual_mem_kb = process.virtual_memory(); // Virtual Memory in KB

        println!("Process Memory Usage:");
        println!("  Resident Set Size (RSS): {} KB (approx. physical RAM used)", mem_kb);
        println!("  Virtual Memory Size (VSZ): {} KB (total virtual memory mapped)", virtual_mem_kb);
        // sysinfo might also offer things like 'peak_memory()' depending on version/OS
    } else {
        println!("Could not retrieve process memory information.");
    }
}

fn error_wrong_type() {
    println!("Failed to execute command - your input was the wrong type.")
}

fn error_not_enough_arguments() {
    println!("Failed to execute command - not enough arguments.")
}

fn error_command_not_found() {
    println!("Failed to execute command - the command you were looking for could not be found.")
}

fn error_dimension_not_found() {
    println!("Failed to execute command - the dimension you were looking for could not be found.")
}

fn error_server_not_started() {
    println!("Failed to execute command - the server has not been started.")
}