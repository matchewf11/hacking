use clap::{
    Parser,
    Subcommand,
};
use crate::hurler::{Error, Hurler};

// <https://docs.rs/clap/latest/clap/_cookbook/git_derive/index.html>

/// A better curl CLI
#[derive(Parser)]
#[command(about = "Hurl is a better curl!", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get a url
    Get {
        /// Path to the url
        path: String,

        /// Whether or not to pretty pring the output
        #[arg(short, long)]
        pretty: bool,
    }
}

pub fn start() -> Result<(), Error> {
    let args = Cli::parse();
    let hurler = Hurler::new();

    match args.command {
        Commands::Get { path, pretty } => {
            let _ = pretty;

            hurler.get(path)
        }
    }
}
