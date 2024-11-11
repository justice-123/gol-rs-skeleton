use crate::gol::event::{Event, State};
use crate::gol::{Params, io::IoCommand};
use crate::util::cell::{CellCoord, CellValue};
use crate::util::cell::CellValue::{Alive, Dead};
use anyhow::Result;
use flume::{Receiver, Sender};
use sdl2::keyboard::Keycode;

pub struct DistributorChannels {
    pub events: Option<Sender<Event>>,
    pub key_presses: Option<Receiver<Keycode>>,
    pub io_command: Option<Sender<IoCommand>>,
    pub io_idle: Option<Receiver<bool>>,
    pub io_filename: Option<Sender<String>>,
    pub io_input: Option<Receiver<CellValue>>,
    pub io_output: Option<Sender<CellValue>>,
}

fn get_alive_cells(world: &Vec<Vec<CellValue>>, params: &Params) -> Vec<CellCoord> {
    let mut alive_cells = Vec::new();
    for y in 0..params.image_height {
        for x in 0..params.image_width {
            if world[y][x] == Alive {
                alive_cells.push(CellCoord { x, y });
            }
        }
    }
    alive_cells
}

pub fn distributor(
    params: Params,
    channels:  &DistributorChannels,
) -> Result<()> {
    //we need to use as_ref to access the value inside the option
    let events = channels.events.as_ref().expect("events channel missing").clone();
    let io_command = channels.io_command.as_ref().expect("io_command channel missing").clone();
    let io_filename = channels.io_filename.as_ref().expect("io_filename channel missing").clone();

    // Create a 2D vector to store the initial world state
    let mut matrix: Vec<Vec<CellValue>> = vec![vec![Dead; params.image_width]; params.image_height];
    let imagename = format!("{}x{}", params.image_width, params.image_height);

    // we have to use a block to avoid immutable borrowing
    {
        let io_input = channels.io_input.as_ref().expect("io_input channel missing");

        io_command.send(IoCommand::IoInput)?;
        io_filename.send(imagename)?;

        for y in 0..params.image_height {
            for x in 0..params.image_width {
                matrix[y][x] = io_input.recv()?;
            }
        }
    }

    events.send(Event::StateChange {
        completed_turns: 0,
        new_state: State::Executing,
    })?;

    let mut turn = 0;
    while turn < params.turns {
        // calculate new alive cells from the current world state
        let new_alive = calculate_new_alive(&matrix, &params);

        // output the current state
        make_output(&new_alive, &params, channels)?;

        // update the current world state for the next iteration
        matrix = new_alive;
        turn += 1;
    }

    events.send(Event::FinalTurnComplete {
        completed_turns: turn as u32,
        alive: get_alive_cells(&matrix, &params),
    })?;


    {
        let io_idle = channels.io_idle.as_ref().expect("io_idle channel missing");

        // Ensure Io has completed any output before exiting
        io_command.send(IoCommand::IoCheckIdle)?;
        io_idle.recv()?;
    } // `io_idle` immutable borrow ends here

    events.send(Event::StateChange {
        completed_turns: turn as u32,
        new_state: State::Quitting,
    })?;

    Ok(())
}

pub fn make_output(
    world: &Vec<Vec<CellValue>>,
    params: &Params,
    channels: &DistributorChannels,
) -> Result<()> {

    let io_command = channels.io_command.as_ref().expect("io_command channel missing").clone();
    let io_filename = channels.io_filename.as_ref().expect("io_filename channel missing").clone();
    let io_output = channels.io_output.as_ref().expect("io_output channel missing").clone();

    io_command.send(IoCommand::IoOutput)?;
    io_filename.send("out".to_string())?;

    for y in 0..params.image_height {
        for x in 0..params.image_width {
            io_output.send(world[y][x])?;
        }
    }


    {
        let io_idle = channels.io_idle.as_ref().expect("io_idle channel missing");
        io_command.send(IoCommand::IoCheckIdle)?;
        io_idle.recv()?;
    }

    Ok(())
}

fn calculate_new_alive(world: &Vec<Vec<CellValue>>, params: &Params) -> Vec<Vec<CellValue>> {
    let mut neighbours = vec![vec![0; params.image_width]; params.image_height];

    for y in 0..params.image_height {
        for x in 0..params.image_width {
            if world[y][x] == Alive {
                for i in -1..=1 {
                    for j in -1..=1 {
                        if i == 0 && j == 0 {
                            continue;
                        }

                        let x_coord = ((x as isize + i + params.image_width as isize)
                            % params.image_width as isize) as usize;
                        let y_coord = ((y as isize + j + params.image_height as isize)
                            % params.image_height as isize) as usize;

                        neighbours[y_coord][x_coord] += 1;
                    }
                }
            }
        }
    }

    let mut new_world = world.clone();

    for y in 0..params.image_height {
        for x in 0..params.image_width {
            let num_neighbours = neighbours[y][x];
            if world[y][x] == Alive {
                if num_neighbours < 2 || num_neighbours > 3 {
                    new_world[y][x] = Dead;
                }
                //
            } else {
                if num_neighbours == 3 {
                    new_world[y][x] = Alive;
                }
            }
        }
    }

    new_world
}
