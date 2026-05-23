use crate::{
    Error, Wurler,
    wurler::Wo,
    parser::{json::parse_json, suite::parse},
    preproc::preproc,
    execute::run_tests,
};
use clap::{Parser, Subcommand};
use std::io::{self, Read};

// <https://docs.rs/clap/latest/clap/_cookbook/git_derive/index.html>



/// A better curl CLI
#[derive(Parser)]
#[command(about = "Wurl is a better curl!", long_about = None)]
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
        // #[arg(short, long)]
        // format: bool,

        /// --path "/foo/bar"
        #[arg(short, long)]
        path: Option<String>,

        /// path args
        /// Query Args
        #[arg(short, long)]
        query: Vec<String>,

        /// headers
        #[arg(long)]
        headers: Vec<String>,
        #[arg(short, long)]
        cookies: Vec<String>,
    },
    Test {
        /// input
        input: String,
    },
    Post {
        /// input
        base: String,

        /// input
        #[arg(short, long)]
        json: Vec<String>,

        // /// input
        // #[arg(short, long)]
        // format: bool,
        /// input
        #[arg(short, long)]
        path: Option<String>,

        // path args
        /// Query Args
        #[arg(short, long)]
        query: Vec<String>,

        // headers
        /// input
        #[arg(long)]
        headers: Vec<String>,

        #[arg(short, long)]
        cookies: Vec<String>,
    },
}

pub async fn start() -> Result<(), Error> {
    let args = Cli::parse();
    let wurler = Wurler::new();

    match args.command {
        Commands::Get {
            base,
            path,
            // format,
            query,
            headers,
            cookies,
        } => {
            wurler
                .get(Wo::new(
                    format!("{}{}", base, path.unwrap_or_default()),
                    Option::None,
                    &query.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    &headers.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    &cookies.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                )?)
                .await?;
        }
        Commands::Test { input } => {

            let input = if &input == "-" {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input).unwrap();
                input
            } else {
                input
            };

            run_tests(parse(&preproc(&input)).unwrap()).await?;
        }
        Commands::Post {
            base,
            path,
            json,
            // format,
            query,
            headers,
            cookies,
        } => {
            let body = parse_json(&json.iter().map(|s| s.as_str()).collect::<Vec<_>>()).to_string();
            wurler
                .post(Wo::new(
                    format!("{}{}", base, path.unwrap_or_default()),
                    Option::Some(body),
                    &query.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    &headers.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    &cookies.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                )?)
                .await?;
        }
    }
    Ok(())
}
