mod app;
mod engine;
mod common;

use std::{sync::LazyLock, time::{Duration, Instant}};

use app::App;
use winit::{event_loop::{EventLoop, ControlFlow}};
use clap::Parser;

use crate::{common::Environment, engine::server::{constants::TICK_RATE, server::Server}};


fn main() {
    env_logger::init();

    let (tx_console, rx_console) = std::sync::mpsc::channel::<String>();
    let (tx_server, rx_server) = std::sync::mpsc::channel::<String>();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app: App = App::new(rx_console, rx_server);
    
    // Spawn the server thread
    std::thread::spawn(move || {
        println!("Server thread spawned");

        let tick_duration = Duration::from_micros(1_000_000 / TICK_RATE);
        println!("Game tick loop started at {} TPS.", TICK_RATE);

        let mut server = Server::start_server();

        let mut next_tick = Instant::now();

        loop {
            let mut ticks: u128 = 0;
            // Logic
            for dimension in server.dimensions.values_mut() {
                dimension.load_chunks();
            }

            if ticks % 60 == 0 {
                println!("60 ticks one second!");
            }
            
            // Send a dummy message to the main thread to show it's ticking
            // Later on, this will be a message with updated game state
            tx_server.send("tick".to_string()).unwrap();

            // Increment tick count
            ticks += 1;

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

    // Spawn a thread that reads terminal input
    std::thread::spawn(move || {
        println!("Terminal input thread spawned");
        use std::io::{self, BufRead};
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let cmd = line.unwrap().to_lowercase();

            tx_console.send(cmd).unwrap();
        }
        println!("Terminal input thread shut down");
    });

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