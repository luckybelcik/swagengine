mod app;
mod engine;
mod common;

use std::{sync::LazyLock};

use app::App;
use winit::{event_loop::{EventLoop, ControlFlow}};
use clap::Parser;

use crate::common::Environment;


fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let (tx, rx) = std::sync::mpsc::channel::<String>();
    
    let mut app: App = App::new(rx);

    // Spawn a thread that reads terminal input
    std::thread::spawn(move || {
        use std::io::{self, BufRead};
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let cmd = line.unwrap().to_lowercase();

            tx.send(cmd).unwrap();
        }
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