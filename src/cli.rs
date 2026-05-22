use clap::{
    Parser,
    Subcommand,
};
use crate::hurler::{Error, Hurler};

// <https://docs.rs/clap/latest/clap/_cookbook/git_derive/index.html>

#[derive(Parser)]
#[command(about = "Hurl is a better curl!", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Get {
        path: String,
    }
}

pub fn start() -> Result<(), Error> {
    let args = Cli::parse();
    let hurler = Hurler::new();

    match args.command {
        Commands::Get { path } => {
            hurler.get(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_test() {

    }
}
