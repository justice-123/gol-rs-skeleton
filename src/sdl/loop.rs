use crate::gol::event::{Event, State};
use crate::sdl::window::Window;
use crate::util::args::Args;
use crate::util::averageturns::AverageTurns;
use anyhow::Result;
use log::info;
use sdl2::keyboard::Keycode;
use tokio::{sync::mpsc::{Receiver, Sender}, select};
use std::time::Duration;
use colored::Colorize;

pub async fn run(
    args: Args,
    mut events: Receiver<Event>,
    key_presses: Sender<Keycode>
) -> Result<()> {
    let mut sdl = Window::new(
        "Gol GUI",
        args.image_width as u32,
        args.image_height as u32,
    )?;
    
    let mut event_pump = sdl.take_event_pump()?;
    let mut dirty = false;
    let mut refresh_interval = tokio::time::interval(Duration::from_secs_f64(1_f64 / args.fps as f64));
    let mut avg_turns = AverageTurns::new();

    'sdl: loop { select! {
        _ = refresh_interval.tick() => {
            match event_pump.poll_event() {
                Some(sdl2::event::Event::Quit { .. } | sdl2::event::Event::KeyDown { keycode: Some(Keycode::Escape), ..})  => key_presses.send(Keycode::Q).await?,
                Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::P), .. }) => key_presses.send(Keycode::P).await?,
                Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::S), .. }) => key_presses.send(Keycode::S).await?,
                Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::Q), .. }) => key_presses.send(Keycode::Q).await?,
                Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::K), .. }) => key_presses.send(Keycode::K).await?,
                _ => (),
            }
            if dirty {
                sdl.render_frame()?;
                dirty = false;
            }
        },
        gol_event = events.recv() => {
            match gol_event {
                Some(Event::CellFlipped { cell, .. }) => sdl.flip_cell(cell.x as u32, cell.y as u32),
                Some(Event::CellsFlipped { cells, ..}) => cells.iter().for_each(|cell| sdl.flip_cell(cell.x as u32, cell.y as u32)),
                Some(Event::TurnComplete { .. }) => dirty = true,
                Some(Event::AliveCellsCount { completed_turns, cells_count }) => info!(target: "Event","Complete Turns {:<8} Alive Cells {:<8} Avg{:>5} turns/s", completed_turns, cells_count, avg_turns.get(completed_turns)),
                Some(Event::FinalTurnComplete { completed_turns, .. }) => info!(target: "Event","Complete Turns {:<8} {}", completed_turns, "Final Turn Complete".bright_green().bold()),
                Some(Event::ImageOutputComplete { completed_turns, filename }) => info!(target: "Event", "Complete Turns {:<8} {}", completed_turns, format!("File {} Output Done", filename).bright_green().bold()),
                Some(Event::StateChange { new_state, completed_turns }) => {
                    info!(target: "Event", "Complete Turns {:<8} {:<20}", completed_turns, new_state.to_string().bright_green().bold());
                    if let State::Quitting = new_state { break 'sdl }
                },
                None => break 'sdl,
            };
        }
    }}

    Ok(())
}

pub async fn run_headless(mut events: Receiver<Event>) -> Result<()> {
    let mut avg_turns = AverageTurns::new();
    loop {
        match events.recv().await {
            Some(Event::AliveCellsCount { completed_turns, cells_count }) => info!(target: "Event", "Complete Turns {:<8} Alive Cells {:<8} Avg{:>5} turns/s", completed_turns, cells_count, avg_turns.get(completed_turns)),
            Some(Event::FinalTurnComplete { completed_turns, .. }) => info!(target: "Event","Complete Turns {:<8} {}", completed_turns, "Final Turn Complete".bright_green().bold()),
            Some(Event::ImageOutputComplete { completed_turns, filename }) => info!(target: "Event", "Complete Turns {:<8} {}", completed_turns, format!("File {} Output Done", filename).bright_green().bold()),
            Some(Event::StateChange { new_state, completed_turns }) => {
                info!(target: "Event", "Complete Turns {:<8} {:<20}", completed_turns, new_state.to_string().bright_green().bold());
                if let State::Quitting = new_state { break }
            },
            None => break,
            _ => (),
        };
    }
    Ok(())
}
