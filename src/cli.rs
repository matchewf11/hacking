use clap::{
    Parser,
    Subcommand,
};
use crate::{Hurler, Error};

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
        /// Base to the url
        base: String,

        /// Whether or not to pretty pring the output
        #[arg(short, long)]
        format: bool,

        /// --path "/foo/bar"
        #[arg(short, long)]
        path: Option<String>,

        /// Query Args
        #[arg(short, long)]
        query: Vec<String>,
    },
    Test {
        input: String,
    },
    Post {
        base: String,
        json: Vec<String>,
    },
}

pub async fn start() -> Result<(), Error> {
    let args = Cli::parse();
    let hurler = Hurler::new();

    match args.command {
        Commands::Get { base, path, format, query } => {
            hurler.get(base).await
        }
        _ => todo!(),
    }
}
