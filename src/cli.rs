use clap::{
    Parser,
    Subcommand,
};

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

pub fn start() {
    let args = Cli::parse();

    match args.command {
        Commands::Get { path } => {
            println!("{path}");
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
