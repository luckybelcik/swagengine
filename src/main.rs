mod app;
mod engine;
mod common;

use std::{collections::HashMap, sync::{mpsc::{Receiver, Sender}, LazyLock}, time::{Duration, Instant}};

use app::App;
use winit::{event_loop::{EventLoop, ControlFlow}};
use clap::Parser;

use crate::{common::Environment, engine::{command_registry::{self, CommandEnvironment, CommandRegistry, DebugCommand, DebugCommandWithArgs}, server::{constants::TICK_RATE, server::Server}}};


fn main() {
    env_logger::init();

    let (tx_console_to_client, rx_console_to_client) = std::sync::mpsc::channel::<DebugCommandWithArgs>();
    let (tx_console_to_server, rx_console_to_server) = std::sync::mpsc::channel::<DebugCommandWithArgs>();
    let (tx_server_to_client, rx_server_to_client) = std::sync::mpsc::channel::<String>();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app: App = App::new(rx_console_to_client, rx_server_to_client);
    
    // Spawn the server thread
    spawn_server_thread(tx_server_to_client, rx_console_to_server);

    // Spawn a thread that reads terminal input
    spawn_console_thread(tx_console_to_client, tx_console_to_server);

    event_loop.run_app(&mut app).unwrap();
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The environment type (e.g., "client", "server", or "both").
    #[arg(short, long, default_value = "both")]
    env: Environment,
}

static ENVIRONMENT: LazyLock<Environment> = LazyLock::new(|| {
    let args = Args::parse();
    args.env
});

pub fn get_environment() -> &'static Environment {
    &ENVIRONMENT
}

fn spawn_server_thread(tx: Sender<String>, rx_console_to_server: Receiver<DebugCommandWithArgs>) {
    std::thread::spawn(move || {
        println!("Server thread spawned");

        let tick_duration = Duration::from_micros(1_000_000 / TICK_RATE);
        println!("Game tick loop started at {} TPS.", TICK_RATE);

        let mut server = Server::start_server(rx_console_to_server);
        let mut next_tick = Instant::now();
        let mut _ticks: u128 = 0;

        loop {
            // Logic
            server.process_commands();
            server.on_tick();
            
            // Send a dummy message to the main thread to show it's ticking
            // Later on, this will be a message with updated game state
            tx.send("tick".to_string()).unwrap();

            // Increment tick count
            _ticks += 1;

            // Sleep until the next tick
            next_tick += tick_duration;
            let now = Instant::now();
            if next_tick > now {
                std::thread::sleep(next_tick - now);
            } else {
                next_tick = now + tick_duration;
            }
        }
    });
}

fn spawn_console_thread(tx_to_client: Sender<DebugCommandWithArgs>, tx_to_server: Sender<DebugCommandWithArgs>) {
    std::thread::spawn(move || {
        println!("Terminal input thread spawned");
        use std::io::{self, BufRead};
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let cmd = line.unwrap().to_lowercase();

            let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let cmd_name = parts[0];
            let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

            if let Some(command) = get_global_command_registry().get(cmd_name).copied() {
                let cmd = DebugCommandWithArgs {
                    debug_command: command,
                    command_args: args,
                };
                match command.command_environment {
                    CommandEnvironment::Client => {tx_to_client.send(cmd).unwrap()},
                    CommandEnvironment::Server => {tx_to_server.send(cmd).unwrap()},
                    CommandEnvironment::Main => {command_registry::handle_main_command(&cmd)},
                }
            } else {
                println!("Unknown command. Type 'help' for a list.");
            }
        }
        println!("Terminal input thread shut down");
    });
}

static COMMAND_REGISTRIES: LazyLock<CommandRegistry> = LazyLock::new(|| {
    let mut registry = HashMap::new();
    registry.extend(command_registry::build_registry(CommandEnvironment::Client));
    registry.extend(command_registry::build_registry(CommandEnvironment::Server));
    registry.extend(command_registry::build_registry(CommandEnvironment::Main));
    return CommandRegistry {
        global_registry: registry,
    };
});

pub fn get_global_command_registry<'a>() -> &'a HashMap<&'static str, DebugCommand> {
    return &COMMAND_REGISTRIES.global_registry;
}