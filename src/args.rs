use clap::{ArgAction, Parser};

#[derive(Parser, Debug, Clone, Copy)]
#[clap(disable_help_flag = true)]
pub struct Args {
    #[arg(
        short = 't',
        long,
        default_value_t = 8,
        help = "Specify the number of worker threads to use."
    )]
    pub threads: usize,

    #[arg(
        short = 'w',
        long = "width",
        default_value_t = 512,
        help = "Specify the width of the image."
    )]
    pub image_width: usize,

    #[arg(
        short = 'h',
        long = "height",
        default_value_t = 512,
        help = "Specify the height of the image."
    )]
    pub image_height: usize,

    #[arg(
        short = 'f',
        long,
        default_value_t = 60,
        help = "Specify the FPS of the SDL window."
    )]
    pub fps: usize,

    #[arg(
        long,
        default_value_t = 10000000,
        help = "Specify the number of turns to process."
    )]
    pub turns: usize,

    #[arg(
        long,
        default_value_t = false,
        help = "Disable the SDL window for running in a headless environment."
    )]
    pub headless: bool,

    #[arg(
        long,
        action = ArgAction::HelpLong
    )]
    help: Option<bool>,
}
