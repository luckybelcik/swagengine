use std::collections::HashMap;
use crate::engine::{client::{client::Client, commands::create_client_commands}, commands::create_main_commands, server::{commands::create_server_commands, server::Server}};

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum CommandEnvironment {
    Client,
    Server,
    Main,
}

pub enum CommandDependency<'a>{
    Client(&'a mut Client),
    Server(&'a mut Server),
    Main,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct DebugCommand {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub execute: fn(&mut CommandDependency, &Vec<String>),
    pub command_environment: CommandEnvironment,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct DebugCommandWithArgs {
    pub debug_command: DebugCommand,
    pub command_args: Vec<String>,
}

pub struct CommandRegistry {
    pub global_registry: HashMap<&'static str, DebugCommand>,
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

pub fn handle_client_command(client: &mut Client, command: &DebugCommandWithArgs) {
    let cmd: DebugCommand = command.debug_command;
    (cmd.execute)(&mut CommandDependency::Client(client), &command.command_args);
}

pub fn handle_server_command(server: &mut Server, command: &DebugCommandWithArgs) {
    let cmd: DebugCommand = command.debug_command;
    (cmd.execute)(&mut CommandDependency::Server(server), &command.command_args);
}

pub fn handle_main_command(command: &DebugCommandWithArgs) {
    let cmd: DebugCommand = command.debug_command;
    (cmd.execute)(&mut CommandDependency::Main, &command.command_args);
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