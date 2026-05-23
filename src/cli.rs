use crate::{
    Error, Hurler,
    hurler::Ho,
    parser::{json::parse_json, suite::parse},
    preproc::preproc,
    execute::run_tests,
};
use clap::{Parser, Subcommand};
use std::io::{self, Read};

// <https://docs.rs/clap/latest/clap/_cookbook/git_derive/index.html>



/// A better curl CLI
#[derive(Parser)]
#[command(
    about = "Hurl is a better curl!",
    long_about = r##"
    grok
       .-""""""-.
     .'          '.
    /   O      O   \
   :                :
   |    \     /     |    H  H   U   U   RRRR   L
   :     '---'      :    H  H   U   U   R   R  L
    \              /     HHHH   U   U   RRRR   L
     '.          .'      H  H   U   U   R  R   L
       '-......-'        H  H   UUUUU   R   R  LLLLL
              |
             _|_
            /   \
           |  ~  |  <--- HURL
            \___/
             |||
            _|||_

        chatgpt
                          .-''''-.
                .'  .--.  '.
               /   /    \   \
              |   | 0  0 |   |
              |   |  /\  |   |
               \   \_==_/   /
                '._  --  _.'
                   |====|
                .--|    |--.
              .'   |    |   '.
             /    /|    |\    \
            /____/ |    | \____\
                   |    |
                  /      \
                 /  /\    \
                /  /  \    \
               /__/    \____\

                      \  \  \  \  \  \  \

                        hh   hh  uu   uu  rrrrrr   ll
                        hh   hh  uu   uu  rr   rr  ll
                        hhhhhhh  uu   uu  rrrrrr   ll
                        hh   hh  uu   uu  rr  rr   ll
                        hh   hh   uuuuu   rr   rr  lllll

                           ~ ~ ~ ~ ~ ~ ~ ~ ~ ~

        Alternative more chaotic version:

                 .-""""-.
           .'  .--.  '.
          /   /    \   \
         |   | x  x |   |
         |   |  --  |   |
          \   \____/   /
           '._.__._.'
              /|\
             / | \
            /  |  \
               |
              / \

                  \
                   \__
                      \___
                          \__hhhhh
                             uu   uu
                             uu   uu
                             uu   uu
                              uuuuu
                           rrrrrr
                           rr   rr
                           rrrrrr
                           rr  rr
                           rr   rr
                                      ll
                                      ll
                                      ll
                                      ll
                                      lllll

    gemini
    _.-'''''''-._
  .'  __    __   '.
 /   (o )  (o )    \
|    __      __     |
|   /  \    /  \    |       ____                 __     
 \  \__/    \__/   /       / __ \__  _______  __/ /_  v v 
  \               /       / /_/ / / / / ___/ / / / /
   '._  '---'  _.'       / __  / /_/ / /  / /_/ / /   ~~~~~
      '-.____.-'        /_/ /_/\__,_/_/   \__,_/_/  (______)_
        /    \                                        (___ __)
       /  ||  \     _ _ _ ___ _ ___ _ _ _ ___ _ _      (____)
      /   ||   \___/ / / /   / /   / / / /   / / \_____/ / /
     (____||____)~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    "##,
)]
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
    let hurler = Hurler::new();

    match args.command {
        Commands::Get {
            base,
            path,
            // format,
            query,
            headers,
            cookies,
        } => {
            hurler
                .get(Ho::new(
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
            hurler
                .post(Ho::new(
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
