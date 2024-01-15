use core::panic;
use std::time::Duration;
use gol_rs::util::args::PanicBehaviour;
use log::{debug, Level};
use tokio::sync::mpsc;
use colored::Colorize;
use gol_rs::util::logger;
use gol_rs::gol::{Params, self, event::{Event, State}};
use crate::utils::common::deadline;
use crate::utils::io::read_alive_counts;

mod utils;

#[tokio::main]
async fn main() {
    logger::init(Level::Debug, false, PanicBehaviour::Exit);
    test_alive().await;
}

/// Count tests will automatically check the 512x512 cell counts for the first 5 messages.
/// You can manually check your counts by looking at CSVs provided in check/alive
async fn test_alive() {
    let start = std::time::Instant::now();
    let params = Params {
        turns: 100000000,
        threads: 1,
        image_width: 512,
        image_height: 512,
    };
    debug!(target: "Test", "{} - {:?}", "Testing Alive Count".cyan(), params);
    let alive_map = read_alive_counts(512, 512).unwrap();
    let (_key_presses_tx, key_presses_rx) = mpsc::channel(10);
    let (events_tx, mut events_rx) = mpsc::channel(1000);

    tokio::spawn(gol::run(params, events_tx.clone(), key_presses_rx));

    let mut ddl = deadline(Duration::from_secs(5), "No AliveCellsCount event received in 5 seconds");
    
    let mut succeed = 0;
    loop {
        let event = events_rx.recv().await;
        match event {
            Some(Event::AliveCellsCount { completed_turns, cells_count }) => {
                if completed_turns == 0 { 
                    continue 
                }
                ddl.abort();
                
                let expected = if completed_turns <= 10000 {
                    *alive_map.get(&completed_turns).unwrap()
                } else if completed_turns % 2 == 0 { 5565 } else { 5567 };
                
                assert_eq!(
                    cells_count, expected,
                    "At turn {} expected {} alive cells, got {} instead", completed_turns, expected, cells_count
                );
                succeed += 1;
                debug!(target: "Test", "Complete Turns {:<8} Alive Cells {:<8}", completed_turns.to_string().bright_green(), cells_count.to_string().bright_green());
                if succeed < 5 {
                    ddl = deadline(Duration::from_secs(3), "No AliveCellsCount event received in 3 seconds");
                } else {
                    break
                }
            },
            Some(Event::StateChange { new_state: State::Quitting, .. }) if succeed >= 5 => break,
            None => panic!("Not enough AliveCellsCount events received"),
            _ => (),
        }
    }
    println!("\ntest result: {}. {} passed; finished in {:.2}s\n", "ok".green(), 1, start.elapsed().as_secs_f32());
    std::process::exit(0);
}

