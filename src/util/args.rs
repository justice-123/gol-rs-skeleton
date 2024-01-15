use std::fmt::Display;
use clap::{Parser, ValueEnum, builder::PossibleValue};

#[derive(Parser, Debug, Clone, Copy)]
pub struct Args {
    #[clap(
        short = 't',
        long,
        default_value_t = 8,
        help = "Specify the number of worker threads to use."
    )]
    pub threads: usize,

    #[clap(
        short = 'w',
        long = "width",
        default_value_t = 512,
        help = "Specify the width of the image."
    )]
    pub image_width: usize,

    #[clap(
        short = 'h',
        long = "height",
        default_value_t = 512,
        help = "Specify the height of the image."
    )]
    pub image_height: usize,

    #[clap(
        short = 'f',
        long = "fps",
        default_value_t = 60,
        help = "Specify the FPS of the SDL window."
    )]
    pub fps: usize,
    
    #[clap(
        long,
        default_value_t = 10000000,
        help = "Specify the number of turns to process."
    )]
    pub turns: usize,

    #[clap(
        long,
        default_value_t = false,
        help = "Disable the SDL window for running in a headless environment."
    )]
    pub headless: bool,

    #[clap(
        long,
        default_value_t = false,
        help = "Enable backtrace for debugging."
    )]
    pub backtrace: bool,

    #[clap(
        long = "panic",
        value_enum,
        default_value_t = PanicBehaviour::Exit,
        help = "Specify behaviour on panic."
    )]
    pub panic_behaviour: PanicBehaviour,
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

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            PanicBehaviour::Exit => Some(PossibleValue::new("exit")),
            PanicBehaviour::Ignore => Some(PossibleValue::new("ignore")),
        }
    }
}
