use std::fmt::Display;
use clap::{builder::PossibleValue, ArgAction, Parser, ValueEnum};

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
        default_value_t = false,
        help = "Enable backtrace for debugging."
    )]
    pub backtrace: bool,

    #[arg(
        long = "panic",
        value_enum,
        default_value_t = PanicBehaviour::Exit,
        help = "Specify behaviour on panic."
    )]
    pub panic_behaviour: PanicBehaviour,

    #[arg(
        long,
        action = ArgAction::HelpLong
    )]
    help: Option<bool>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PanicBehaviour {
    #[default]
    Exit,
    Ignore,
}

impl Display for PanicBehaviour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ValueEnum for PanicBehaviour {
    fn value_variants<'a>() -> &'a [Self] {
        &[PanicBehaviour::Exit, PanicBehaviour::Ignore]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            PanicBehaviour::Exit => Some(PossibleValue::new("exit")),
            PanicBehaviour::Ignore => Some(PossibleValue::new("ignore")),
        }
    }
}
