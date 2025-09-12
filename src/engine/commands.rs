use sysinfo::System;

use crate::{app::App, engine::command_registry::{error_command_not_found, error_not_enough_arguments, DebugCommand}};

pub fn create_general_commands() -> Vec<DebugCommand> {
    let mut commands = Vec::new();

    commands.push(DebugCommand {
        name: "help",
        aliases: &["h", "m"],
        description: "Prints the help menu.",
        execute: |_app, _args| {
            print_debug_menu(_app);
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
    
    return commands;
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

fn print_aliases(app: &mut App, target_name: &str) {
    let target_command = &app.command_registry.get(target_name).unwrap();
    let aliases = target_command.aliases;

    println!("Available aliases for {target_name}:");
    for string in aliases {
        print!("{string}, ")
    }
    println!("{}", target_command.name);
}