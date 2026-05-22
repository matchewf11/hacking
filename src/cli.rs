use crate::{Error, Hurler, parser::parse_json};
use clap::{Parser, Subcommand};

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

        // :id
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
        Commands::Get {
            base,
            path,
            format,
            query,
        } => 
            hurler
                .get(base, &query.iter().map(|s| s.as_str()).collect::<Vec<_>>())
                .await,
        Commands::Test { input } => {
            todo!();
        }
        Commands::Post { base, json } => {
            let json = parse_json(
                &json
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
            )
            .to_string();
            hurler.post(base, json).await
        }
    }
}
