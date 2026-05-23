use crate::{Error, Hurler, hurler::Ho, parser::{json::parse_json, suite::parse}, test::run_tests};
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

        // path args
        /// Query Args
        #[arg(short, long)]
        query: Vec<String>,

        // headers
        headers: Vec<String>,
    },
    Test {
        input: String,
    },
    Post {
        base: String,
        json: Vec<String>,

        #[arg(short, long)]
        format: bool,

        #[arg(short, long)]
        path: Option<String>,

        // path args
        /// Query Args
        #[arg(short, long)]
        query: Vec<String>,

        // headers
        headers: Vec<String>,
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
            headers,
        } => {
            hurler
                .get(Ho::new(
                    format!("{}{}", base, path.unwrap_or_default()),
                    Option::None,
                    &query.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    &headers.iter().map(|s| s.as_str()).collect::<Vec<_>>(), // should this even be an option
                )?)
                .await?;
        }
        Commands::Test { input } => {
            run_tests(parse(&input).unwrap())?;
        }
        Commands::Post {
            base,
            path,
            json,
            format,
            query,
            headers,
        } => {
            let body = parse_json(&json.iter().map(|s| s.as_str()).collect::<Vec<_>>()).to_string();
            hurler
                .post(Ho::new(
                    format!("{}{}", base, path.unwrap_or_default()),
                    Option::Some(body),
                    &query.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    &headers.iter().map(|s| s.as_str()).collect::<Vec<_>>(), // should this even be an option
                )?)
                .await?;
        }
    }
    Ok(())
}
