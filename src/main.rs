mod engine;

use std::{collections::HashMap, sync::{mpsc::{Receiver, Sender}, LazyLock}, thread::JoinHandle, time::{Duration, Instant}};

use winit::{event_loop::{EventLoop, ControlFlow}};

use crate::engine::{client::client::Client, command_registry::{self, CommandEnvironment, CommandRegistry, DebugCommand, DebugCommandWithArgs}, server::{constants::TICK_RATE, server::Server}};


fn main() {
    env_logger::init();

    let (tx_console_to_client, rx_console_to_client) = std::sync::mpsc::channel::<DebugCommandWithArgs>();
    let (tx_console_to_server, rx_console_to_server) = std::sync::mpsc::channel::<DebugCommandWithArgs>();
    let (tx_server_to_client, rx_server_to_client) = std::sync::mpsc::channel::<String>();

    // Spawn a thread that reads terminal input
    spawn_console_thread(tx_console_to_client, tx_console_to_server);

    // Spawn the server thread
    #[cfg(feature = "server")]
    {
        // Start server on a separate thread if also launching client
        #[cfg(feature = "client")]
        spawn_server_thread(tx_server_to_client, rx_console_to_server);

        // Start server in main if not launching client
        #[cfg(not(feature = "client"))]
        {
            println!("No client - initializing server on main thread");
            initialize_server(tx_server_to_client, rx_console_to_server);
        }
    }

    // Initialize client and start event loop
    #[cfg(feature = "client")]
    initialize_client(rx_console_to_client, rx_server_to_client);
}

fn initialize_client(rx_console_to_client: Receiver<DebugCommandWithArgs>, rx_server_to_client: Receiver<String>) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut client: Client = Client::new(rx_console_to_client, rx_server_to_client);

    println!("Started client with player UUID {}", client.get_uuid_string());

    event_loop.run_app(&mut client).unwrap();
}

fn spawn_server_thread(tx_server_to_client: Sender<String>, rx_console_to_server: Receiver<DebugCommandWithArgs>) -> JoinHandle<()> {
    return std::thread::spawn(move || {
        println!("Server thread spawned");

        initialize_server(tx_server_to_client, rx_console_to_server);
    });
}

fn initialize_server(tx_server_to_client: Sender<String>, rx_console_to_server: Receiver<DebugCommandWithArgs>) {
    let tick_duration = Duration::from_micros(1_000_000 / TICK_RATE);
    println!("Game tick loop started at {} TPS.", TICK_RATE);

    let mut server = Server::start_server(rx_console_to_server);
    let mut next_tick = Instant::now();
    let mut _ticks: u128 = 0;

    while server.is_running() {
        // Logic
        server.process_commands();
        server.on_tick();
        
        // Send a dummy message to the main thread to show it's ticking
        // Later on, this will be a message with updated game state
        tx_server_to_client.send("tick".to_string()).unwrap();

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
    #[cfg(feature = "client")]
    registry.extend(command_registry::build_registry(CommandEnvironment::Client));

    #[cfg(feature = "server")]
    registry.extend(command_registry::build_registry(CommandEnvironment::Server));
    
    registry.extend(command_registry::build_registry(CommandEnvironment::Main));
    return CommandRegistry {
        global_registry: registry,
    };
});

pub fn get_global_command_registry<'a>() -> &'a HashMap<&'static str, DebugCommand> {
    return &COMMAND_REGISTRIES.global_registry;
}