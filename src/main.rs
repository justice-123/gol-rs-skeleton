use anyhow::Result;
use clap::Parser;
use log::{info, Level};
use sdl2::keyboard::Keycode;
use tokio::{try_join, sync::mpsc::{self, Sender}};
use gol_rs::args::Args;
use gol_rs::gol;
use gol_rs::sdl;
use gol_rs::util::logger;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    logger::init(Level::Info, args.backtrace, args.panic_behaviour);

    info!(target: "Main", "{:<10} {}", "Threads", args.threads);
    info!(target: "Main", "{:<10} {}", "Width", args.image_width);
    info!(target: "Main", "{:<10} {}", "Height", args.image_height);
    info!(target: "Main", "{:<10} {}", "Turns", args.turns);

    let (key_presses_tx, key_presses_rx) = mpsc::channel(10);
    let (events_tx, events_rx) = mpsc::channel(1000);

    tokio::spawn(sigint(key_presses_tx.clone()));

    let gol = gol::run(args, events_tx, key_presses_rx);
    if !args.headless {
        try_join!(gol, sdl::r#loop::run(args, events_rx, key_presses_tx))?;
    } else {
        try_join!(gol, sdl::r#loop::run_headless(events_rx))?;
    }

    Ok(())
}

async fn sigint(key_presses_tx: Sender<Keycode>) {
    tokio::signal::ctrl_c().await.unwrap();
    key_presses_tx.send(Keycode::Q).await.unwrap();
}
