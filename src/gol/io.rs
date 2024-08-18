use crate::gol::Params;
use crate::util::{cell::CellValue, traits::AsBytes};
use anyhow::{Context, Result};
use log::trace;
use tokio::{sync::mpsc::{UnboundedReceiver, UnboundedSender}, fs::File, io::{AsyncReadExt, AsyncWriteExt, BufWriter}};

#[derive(Debug, PartialEq, Eq)]
pub enum IoCommand {
    IoCheckIdle,
    IoInput,
    IoOutput,
}

pub struct IoChannels {
    pub command: Option<UnboundedReceiver<IoCommand>>,
    pub idle: Option<UnboundedSender<bool>>,
    pub filename: Option<UnboundedReceiver<String>>,
    pub input: Option<UnboundedSender<CellValue>>,
    pub output: Option<UnboundedReceiver<CellValue>>,
}

struct IoState {
    params: Params,
    channels: IoChannels,
}

pub async fn start_io(params: Params, channels: IoChannels) {
    let mut io = IoState { params, channels };
    let mut command = io.channels.command.take().unwrap();
    let idle = io.channels.idle.take().unwrap();
    'io: loop {
        match command.recv().await {
            Some(IoCommand::IoInput) => io.read_pgm_image().await.unwrap(),
            Some(IoCommand::IoOutput) => io.write_pgm_image().await.unwrap(),
            Some(IoCommand::IoCheckIdle) => idle.send(true).unwrap(),
            None => break 'io,
        }
    }
}

impl IoState {
    async fn read_pgm_image(&mut self) -> Result<()> {
        let filename = self
            .channels
            .filename
            .as_mut()
            .context("Missing filename channel")?
            .recv()
            .await
            .context("Sender of IoFilename channel is unexpectedly closed")?;

        let path = format!("images/{}.pgm", filename);
        let mut buffer = Vec::new();
        File::open(path).await?.read_to_end(&mut buffer).await?;
        let pgm = image::load_from_memory(&buffer)?;

        assert_eq!(
            pgm.width(),
            self.params.image_width as u32,
            "Incorrect width"
        );
        assert_eq!(
            pgm.height(),
            self.params.image_height as u32,
            "Incorrect height"
        );

        for byte in pgm.into_bytes() {
            self.channels
                .input
                .as_ref()
                .context("Missing input channel")?
                .send(CellValue::from(byte))?;
        }

        trace!("File {} input done!", filename);
        Ok(())
    }

    async fn write_pgm_image(&mut self) -> Result<()> {
        std::fs::create_dir_all("out")?;
        let filename = self
            .channels
            .filename
            .as_mut()
            .context("Missing filename channel")?
            .recv()
            .await
            .unwrap();
        let path = format!("out/{}.pgm", filename);
        let file = File::create(path).await?;

        let mut writer = BufWriter::new(file);
        writer.write_all("P5".as_bytes()).await?;
        writer.write_all("\n".as_bytes()).await?;
        writer.write_all(self.params.image_width.to_string().as_bytes()).await?;
        writer.write_all(" ".to_string().as_bytes()).await?;
        writer.write_all(self.params.image_height.to_string().as_bytes()).await?;
        writer.write_all("\n".as_bytes()).await?;
        writer.write_all(255_usize.to_string().as_bytes()).await?;
        writer.write_all("\n".as_bytes()).await?;

        let mut world = vec![CellValue::Dead; self.params.image_width * self.params.image_height];
        let output_ch = self.channels.output.as_mut().context("Missing output channel")?;
        for i in world.iter_mut() {
            *i = output_ch.recv().await.unwrap();
        }
        writer.write_all(world.as_bytes()).await?;
        writer.flush().await?;
        trace!("File {} output done!", filename);
        Ok(())
    }
}
