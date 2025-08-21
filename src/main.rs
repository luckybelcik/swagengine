mod app;
mod engine;

use app::App;
use winit::{event_loop::{EventLoop, ControlFlow}};

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