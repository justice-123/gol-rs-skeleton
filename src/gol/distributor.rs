use crate::gol::event::{Event, State};
use crate::gol::Params;
use crate::gol::io::IoCommand;
use crate::util::cell::CellValue;
use anyhow::Result;
use flume::{Receiver, Sender};
use sdl2::keyboard::Keycode;

use super::io::IoChannels;

pub struct DistributorChannels {
    pub events: Option<Sender<Event>>,
    pub key_presses: Option<Receiver<Keycode>>,
    pub io_command: Option<Sender<IoCommand>>,
    pub io_idle: Option<Receiver<bool>>,
    pub io_filename: Option<Sender<String>>,
    pub io_input: Option<Receiver<CellValue>>,
    pub io_output: Option<Sender<CellValue>>,
}

pub fn distributor(
    params: Params,
    mut channels: DistributorChannels
) -> Result<()> {
    let events = channels.events.take().unwrap();
    let key_presses = channels.key_presses.take().unwrap();
    let io_command = channels.io_command.take().unwrap();
    let io_idle = channels.io_idle.take().unwrap();

    // TODO: Create a 2D vector to store the world.

    let mut matrix: Vec<Vec<CellValue>> = vec![vec![CellValue::Dead; params.image_width]; params.image_height];



    let turn = 0;
    events.send(
        Event::StateChange { completed_turns: turn, new_state: State::Executing })?;


    // TODO: Execute all turns of the Game of Life.

    let imagename = format!("{}x{}", params.image_width, params.image_height);

    channels.io_command.unwrap().send(IoCommand::IoInput);
    channels.io_filename.unwrap().send(imagename).expect("failed to send file name");


    if let Some(io_input) = channels.io_input.take() {
        let mut matrix: Vec<Vec<CellValue>> = vec![vec![CellValue::default(); params.image_width]; params.image_height];
    
        for y in 0..params.image_height {
            for x in 0..params.image_width {
                match io_input.try_recv() {
                    Ok(data) => {
                        // Store the data in the 2D matrix
                        matrix[y][x] = data;
                        println!("Received data at ({}, {}): {:?}", y, x, data);
                    },
                    Err(flume::TryRecvError::Empty) => {
                        // Handle an empty channel if needed
                        println!("Channel is empty at ({}, {})", y, x);
                    },
                    Err(flume::TryRecvError::Disconnected) => {
                        // Handle a disconnected channel
                        println!("Channel is disconnected at ({}, {})", y, x);
                        break; // Exit the loop if the channel is disconnected
                    },
                }
            }
        }
    } else {
        println!("io_input channel is not present");
    }

    //we have the world written in now 

    // TODO: Report the final state using FinalTurnCompleteEvent.


    // Make sure that the Io has finished any output before exiting.
    io_command.send(IoCommand::IoCheckIdle)?;
    io_idle.recv()?;

    events.send(
        Event::StateChange { completed_turns: turn, new_state: State::Quitting })?;
    Ok(())
}

pub fn make_output(world: Vec<Vec<CellValue>>, params: Params, mut channels: DistributorChannels) {
    let events = channels.events.take().unwrap();
    let key_presses = channels.key_presses.take().unwrap();
    let io_command = channels.io_command.take().unwrap();
    let io_idle = channels.io_idle.take().unwrap();
    let io_filename = channels.io_filename.take().unwrap();
    io_command.send(IoCommand::IoOutput);
    io_filename.send("out");


}
