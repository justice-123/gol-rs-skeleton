use backtrace::Backtrace;
use log::{error, Level};
use std::panic;
use colored::Colorize;
use crate::util::args::PanicBehaviour;

pub fn init(level: Level, backtrace: bool, panic_behaviour: PanicBehaviour) {
    let level = std::env::var("RUST_LOG").unwrap_or(level.to_string());
    std::env::set_var("RUST_LOG", &level);
    if env_logger::try_init().is_ok() {
        panic::set_hook(Box::new(move |panic| {
            error!(target: "Main", "{}", panic.to_string().bright_red());
            if backtrace {
                error!("Backtrace: \n{:?}", Backtrace::new());
            }
            if matches!(panic_behaviour, PanicBehaviour::Exit) {
                std::process::exit(1);
            }
        }));
    }
}
