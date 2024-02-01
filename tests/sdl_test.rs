use std::{collections::HashMap, time::Duration, future::Future};
use anyhow::Result;
use clap::{Command, Arg};
use colored::Colorize;
use log::{debug, Level};
use sdl2::keyboard::Keycode;
use tokio::{sync::{mpsc::{self, Sender, Receiver}, watch, oneshot}, select};
use gol_rs::{args::PanicBehaviour, gol::{Params, self, event::{Event, State}}, util::{logger, cell::GolCell}};
use utils::{io::{read_alive_counts, read_alive_cells}, visualise::assert_eq_board, sdl, common::deadline};

mod utils;

#[tokio::main]
async fn main() {
    let start = std::time::Instant::now();
    let command = Command::new("Gol")
        .arg(Arg::new("mode")
                .value_parser(["headless", "sdl"])
                .default_value("headless"))
        .get_matches();
    logger::init(Level::Debug, false, PanicBehaviour::Exit);
    let headless = matches!(command.get_one::<String>("mode").unwrap().as_str(), "headless");
    let passed_tests = test_sdl(headless).await.unwrap();
    println!("\ntest result: {}. {} passed; finished in {:.2}s\n", "ok".green(), passed_tests, start.elapsed().as_secs_f32());
    std::process::exit(0);
}

/// Sdl tests program behaviour on key presses
async fn test_sdl(headless: bool) -> Result<usize> {
    let params: Params = Params {
        turns: 100000000,
        threads: 8,
        image_width: 512,
        image_height: 512,
    };
    debug!(target: "Test", "{} - {:?}", "Testing Sdl".cyan(), params);

    let (key_presses_tx, key_presses_rx) = mpsc::channel(10);
    let (key_presses_forward_tx, key_presses_forward_rx) = mpsc::channel(10);
    let (events_tx, events_rx) = mpsc::channel(1000);
    let (events_forward_tx, events_forward_rx) = mpsc::channel(1000);
    let (gol_done_tx, gol_done_rx) = oneshot::channel();

    let gol = tokio::spawn(async move { Ok(gol_done_tx.send(gol::run(params, events_tx, key_presses_forward_rx).await.unwrap())) });
    let tester = tokio::spawn(Tester::start(params, key_presses_tx, events_forward_rx, gol_done_rx));
    let (gol, sdl, tester) = if headless {
        let sdl = sdl::run_headless(events_rx, key_presses_rx, events_forward_tx, key_presses_forward_tx);
        tokio::join!(gol, sdl, tester)
    } else {
        let sdl = sdl::run(params, "Gol GUI - Test Sdl", events_rx, key_presses_rx, events_forward_tx, key_presses_forward_tx);
        tokio::join!(gol, sdl, tester)
    };
    sdl.and(gol?).and(tester?).and(Ok(1))
}

struct Tester {
    params: Params,
    key_presses: Sender<Keycode>,
    events: Receiver<Event>,
    events_watcher: watch::Receiver<Option<Event>>,
    turn: u32,
    world: Vec<Vec<u8>>,
    alive_map: HashMap<u32, u32>,
}

impl Tester {
    async fn start(
        params: Params,
        key_presses: Sender<Keycode>,
        events: Receiver<Event>,
        gol_done: oneshot::Receiver<()>,
    ) -> Result<()> {
        let (watcher_tx, watcher_rx) = watch::channel::<Option<Event>>(None);
        let mut tester = Tester {
            params,
            key_presses,
            events,
            events_watcher: watcher_rx,
            turn: 0,
            world: vec![vec![0_u8; params.image_width]; params.image_height],
            alive_map: read_alive_counts(params.image_width as u32, params.image_height as u32)?,
        };

        tokio::spawn(tester.test_pause(Duration::from_secs(3)));
        tokio::spawn(tester.test_output(Duration::from_secs(12)));
        let quitting = tokio::spawn(tester.test_quitting(Duration::from_secs(16)));
        let deadline = deadline(Duration::from_secs(25), "Your program should complete this test within 20 seconds. Is your program deadlocked?");
        
        let mut cell_flipped_received = false;
        let mut turn_complete_received = false;

        loop {
            select! {
                gol_event = tester.events.recv() => {
                    match gol_event {
                        Some(Event::CellFlipped { completed_turns, cell }) => {
                            cell_flipped_received = true;
                            assert!(completed_turns == tester.turn || completed_turns == tester.turn + 1,
                                "Expected completed {} turns, got {} instead", tester.turn, completed_turns);
                            tester.world[cell.y][cell.x] = !tester.world[cell.y][cell.x];
                        },
                        Some(Event::CellsFlipped { completed_turns, cells }) => {
                            cell_flipped_received = true;
                            assert!(completed_turns == tester.turn || completed_turns == tester.turn + 1,
                                "Expected completed {} turns, got {} instead", tester.turn, completed_turns);
                            cells.iter().for_each(|cell| tester.world[cell.y][cell.x] = !tester.world[cell.y][cell.x]);
                        },
                        Some(Event::TurnComplete { completed_turns }) => {
                            turn_complete_received = true;
                            tester.turn += 1;
                            assert_eq!(completed_turns, tester.turn,
                                "Expected completed {} turns, got {} instead", tester.turn, completed_turns);
                            tester.test_alive();
                            tester.test_gol();
                        },
                        e @ Some(Event::ImageOutputComplete { .. }) => watcher_tx.send(e).unwrap(),
                        e @ Some(Event::StateChange { .. }) => watcher_tx.send(e).unwrap(),
                        e @ Some(Event::FinalTurnComplete { .. }) => watcher_tx.send(e).unwrap(),
                        Some(_) => (),
                        None => {
                            if !cell_flipped_received {
                                panic!("No CellFlipped events received");
                            }
                            if !turn_complete_received {
                                panic!("No TurnComplete events received");
                            }
                            quitting.await.unwrap();
                            gol_done.await.unwrap();
                            deadline.abort();
                            break
                        },
                    }
                },
            }
        }

        Ok(())
    }

    fn test_alive(&self) {
        let alive_count = self.world.iter().flatten().filter(|&&cell| cell == 0xFF_u8).count();
        let expected = if self.turn <= 10000 { *self.alive_map.get(&self.turn).unwrap() } 
            else if self.turn % 2 == 0 { 5565 } else { 5567 };
        assert_eq!(
            alive_count, expected as usize,
            "At turn {} expected {} alive cells, got {} instead", self.turn, expected, alive_count
        );
    }

    fn test_gol(&self) {
        if self.turn == 0 || self.turn == 1 || self.turn == 100 {
            let path = format!("check/images/{}x{}x{}.pgm", self.params.image_width, self.params.image_height, self.turn);
            let expected_alive = read_alive_cells(path, self.params.image_width, self.params.image_height).unwrap();
            let alive_cells = self.world.iter().enumerate()
                .flat_map(|(y, row)| 
                    row.iter().enumerate()
                        .filter(|&(_, &cell)| cell != 0_u8)
                        .map(move |(x, _)| GolCell { x, y }))
                .collect::<Vec<GolCell>>();
            assert_eq_board(self.params, &alive_cells, &expected_alive); 
        }
    }

    fn test_output(&self, delay: Duration) -> impl Future<Output = ()> {
        let key_presses = self.key_presses.clone();
        let mut event_watcher = self.events_watcher.clone();
        let (width, height) = (self.params.image_width, self.params.image_height); 
        async move {
            tokio::time::sleep(delay).await;
            debug!(target: "Test", "{}", "Testing image output".cyan());
            key_presses.send(Keycode::S).await.unwrap();
            tokio::time::timeout(Duration::from_secs(4), async {
                let event = event_watcher.wait_for(|e| matches!(e, Some(Event::ImageOutputComplete { .. }))).await.unwrap().to_owned();
                if let Some(Event::ImageOutputComplete { completed_turns, filename }) = event {
                    assert_eq!(filename.to_owned(), format!("{}x{}x{}", width, height, completed_turns), "Filename is not correct");
                }
            }).await
            .expect("No ImageOutput events received in 4 seconds");
        }
    }
    
    fn test_pause(&self, delay: Duration) -> impl Future<Output = ()> {
        let key_presses = self.key_presses.clone();
        let mut event_watcher = self.events_watcher.clone();
        let test_output = self.test_output(Duration::from_secs(2));
        async move {
            tokio::time::sleep(delay).await;
            debug!(target: "Test", "{}", "Testing Pause key pressed".cyan());
            key_presses.send(Keycode::P).await.unwrap();
            tokio::time::timeout(Duration::from_secs(2), async {
                event_watcher.wait_for(|e| 
                    matches!(e, Some(Event::StateChange { new_state: State::Pause, .. }))).await.unwrap()
            }).await.expect("No Pause events received in 2 seconds");
            
            test_output.await;

            tokio::time::sleep(Duration::from_secs(2)).await;
            debug!(target: "Test", "{}", "Testing Pause key pressed again".cyan());
            key_presses.send(Keycode::P).await.unwrap();
            tokio::time::timeout(Duration::from_secs(2), async {
                event_watcher.wait_for(|e| 
                    matches!(e, Some(Event::StateChange { new_state: State::Executing, .. }))).await.unwrap();
            }).await.expect("No Executing events received in 2 seconds");
        }
    }

    fn test_quitting(&self, delay: Duration) -> impl Future<Output = ()> {
        let key_presses = self.key_presses.clone();
        let mut event_watcher = self.events_watcher.clone();
        async move {
            tokio::time::sleep(delay).await;
            debug!(target: "Test", "{}", "Testing Quit key pressed".cyan());
            key_presses.send(Keycode::Q).await.unwrap();
            tokio::time::timeout(Duration::from_secs(2), async {
                event_watcher.wait_for(|e| 
                    matches!(e, Some(Event::FinalTurnComplete { .. }))).await.unwrap();
            }).await.expect("No FinalTurnComplete events received in 2 seconds");

            tokio::time::timeout(Duration::from_secs(4), async {
                event_watcher.wait_for(|e| 
                    matches!(e, Some(Event::ImageOutputComplete { .. }))).await.unwrap();
            }).await.expect("No ImageOutput events received in 4 seconds");

            tokio::time::timeout(Duration::from_secs(2), async {
                event_watcher.wait_for(|e| 
                    matches!(e, Some(Event::StateChange { new_state: State::Quitting, .. }))).await.unwrap();
            }).await.expect("No Quitting events received in 2 seconds");
        }
    }

}

