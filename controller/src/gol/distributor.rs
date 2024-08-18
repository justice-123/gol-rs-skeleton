use crate::gol::event::{Event, State};
use crate::gol::Params;
use crate::gol::io::IoCommand;
use crate::gol::distributor::controller_handler_client::ControllerHandlerClient;
use crate::util::cell::CellValue;
use crate::util::traits::AsBytes;
use anyhow::Result;
use log::info;
use tokio::sync::mpsc::{Sender, UnboundedSender, UnboundedReceiver};


pub struct DistributorChannels {
    pub events: Option<Sender<Event>>,
    pub io_command: Option<UnboundedSender<IoCommand>>,
    pub io_idle: Option<UnboundedReceiver<bool>>,
    pub io_filename: Option<UnboundedSender<String>>,
    pub io_output: Option<UnboundedSender<CellValue>>,
    pub io_input: Option<UnboundedReceiver<CellValue>>,
}

pub async fn remote_distributor(
    params: Params,
    mut channels: DistributorChannels
) -> Result<()> {
    let events = channels.events.as_ref().unwrap();
    let io_command = channels.io_command.as_ref().unwrap();
    let io_idle = channels.io_idle.as_mut().unwrap();

    let turn = 0;

    // Example for tonic RPC
    example_rpc_call().await?;

    io_command.send(IoCommand::IoCheckIdle)?;
    io_idle.recv().await;
    events.send(
        Event::StateChange {
            completed_turns: turn,
            new_state: State::Quitting,
        }
    ).await?;
    Ok(())
}


// Example for tonic RPC
tonic::include_proto!("gol_proto");

async fn example_rpc_call() -> Result<()> {
    let mut client = ControllerHandlerClient::connect("http://127.0.0.1:8030").await?;
    // Create a 3x3 world
    let world =
        vec![vec![CellValue::Alive, CellValue::Alive, CellValue::Alive],
             vec![CellValue::Dead, CellValue::Dead, CellValue::Dead],
             vec![CellValue::Alive, CellValue::Alive, CellValue::Alive]];

    // Convert Vec<Vec<CellValue>> to Vec<u8> (bytes)
    let bytes = world.iter().flat_map(|row| row.as_bytes().to_vec()).collect();
    assert_eq!(bytes, vec![255, 255, 255, 0, 0, 0, 255, 255, 255]);

    // Push the world to the server and receive the response (number of alive cells) by RPC call
    // the RPC call `push_world()` is defined in `proto/stub.proto`
    let response = client.push_world(
        tonic::Request::new(World {
            width: 3,
            height: 3,
            cell_values: bytes,
        })
    ).await;

    // Handle response
    match response {
        Ok(response) => {
            let msg = response.into_inner();
            info!("response: {:?}", msg);
            assert_eq!(
                msg.cells_count as usize,
                world.iter().flatten().filter(|cell| cell.is_alive()).count()
            );
        },
        Err(e) => log::error!("Server error: {}", e),
    }

    // Another example of closing the server by RPC call
    client.shutdown_broker(tonic::Request::new(Empty { })).await?;

    Ok(())
}
