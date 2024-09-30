use crate::gol::event::{Event, State};
use crate::gol::Params;
use crate::gol::io::IoCommand;
use crate::util::cell::CellValue;
use anyhow::Result;
use sdl2::keyboard::Keycode;
use tokio::sync::mpsc::{Receiver, Sender, UnboundedReceiver, UnboundedSender};

pub struct DistributorChannels {
    pub events: Option<Sender<Event>>,
    pub key_presses: Option<Receiver<Keycode>>,
    pub io_command: Option<UnboundedSender<IoCommand>>,
    pub io_idle: Option<UnboundedReceiver<bool>>,
    pub io_filename: Option<UnboundedSender<String>>,
    pub io_output: Option<UnboundedSender<CellValue>>,
    pub io_input: Option<UnboundedReceiver<CellValue>>,
}

pub fn distributor(
    params: Params,
    mut channels: DistributorChannels
) -> Result<()> {
    let events = channels.events.as_ref().unwrap();
    let io_command = channels.io_command.as_ref().unwrap();
    let io_idle = channels.io_idle.as_mut().unwrap();

    // TODO: Create a 2D vector to store the world.

    let turn = 0;
    events.blocking_send(
        Event::StateChange { completed_turns: turn, new_state: State::Executing })?;

    // TODO: Execute all turns of the Game of Life.

    // TODO: Report the final state using FinalTurnCompleteEvent.


    // Make sure that the Io has finished any output before exiting.
    io_command.send(IoCommand::IoCheckIdle)?;
    io_idle.blocking_recv();

    events.blocking_send(
        Event::StateChange { completed_turns: turn, new_state: State::Quitting })?;
    Ok(())
}
