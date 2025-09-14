use std::collections::HashMap;
use crate::{engine::{client::commands::create_client_commands, commands::create_main_commands, server::{commands::create_server_commands, server::Server}}, get_global_command_registry, App};

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum CommandEnvironment {
    Client,
    Server,
    Main,
}

pub enum CommandDependency<'a>{
    App(&'a mut App),
    Server(&'a mut Server),
    Main,
}

#[derive(Eq, PartialEq, Hash)]
pub struct DebugCommand {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub execute: fn(&mut CommandDependency, &[&str]),
    pub command_environment: CommandEnvironment,
}

pub fn build_registry<'a>(environment: CommandEnvironment) -> HashMap<&'static str, DebugCommand> {
    let mut mapped = HashMap::new();
    let mut commands = Vec::new();

    match environment {
        CommandEnvironment::Client => {commands.extend(create_client_commands())},
        CommandEnvironment::Server => {commands.extend(create_server_commands())},
        CommandEnvironment::Main => {commands.extend(create_main_commands())},
    }

    for cmd in commands {
        if cmd.command_environment != environment {
            continue;
        }
        for alias in cmd.aliases {
            mapped.insert(*alias, DebugCommand {
                name: cmd.name,
                aliases: cmd.aliases,
                description: cmd.description,
                execute: cmd.execute,
                command_environment: cmd.command_environment,
            });
        }
        mapped.insert(cmd.name, cmd);
    }

    return mapped;
}

pub fn handle_client_command(app: &mut App, input: &str) {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let cmd_name = parts[0];
    let args = &parts[1..];

    if let Some(command) = get_global_command_registry().get(cmd_name) {
        (command.execute)(&mut CommandDependency::App(app), args);
    } else {
        println!("Unknown command. Type 'help' for a list.");
    }
}

pub fn handle_server_command(server: &mut Server, input: &str) {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let cmd_name = parts[0];
    let args = &parts[1..];

    if let Some(command) = get_global_command_registry().get(cmd_name) {
        (command.execute)(&mut CommandDependency::Server(server), args);
    } else {
        println!("Unknown command. Type 'help' for a list.");
    }
}

pub fn handle_main_command(app: &mut App, server: &mut Server, input: &str) {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let cmd_name = parts[0];
    let args = &parts[1..];

    if let Some(command) = get_global_command_registry().get(cmd_name) {
        (command.execute)(&mut CommandDependency::Main, args);
    } else {
        println!("Unknown command. Type 'help' for a list.");
    }
}

pub fn error_wrong_type() {
    println!("Failed to execute command - your input was the wrong type.")
}

pub fn error_not_enough_arguments() {
    println!("Failed to execute command - not enough arguments.")
}

pub fn error_command_not_found() {
    println!("Failed to execute command - the command you were looking for could not be found.")
}

pub fn error_dimension_not_found() {
    println!("Failed to execute command - the dimension you were looking for could not be found.")
}

pub fn error_server_not_started() {
    println!("Failed to execute command - the server has not been started.")
}