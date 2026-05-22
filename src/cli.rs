use clap::{
    Parser,
    Subcommand,
};

// <https://docs.rs/clap/latest/clap/_cookbook/git_derive/index.html>

#[derive(Parser)]
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

pub fn start() {
    let args = Cli::parse();

    match args.command {
        Commands::Get { path } => {
            println!("{path}");
        }
    }
}
