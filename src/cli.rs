use crate::{
    Error, Wurler,
    execute::run_tests,
    parser::suite::parse,
    preproc::preproc,
};
use clap::{Args, Parser, Subcommand};
use reqwest::Method;
use std::io::{self, Read};

// <https://docs.rs/clap/latest/clap/_cookbook/git_derive/index.html>

// ── Shared request flags ───────────────────────────────────────────────────────

/// Flags common to every HTTP request.
///
/// Derived as [`Args`] so it can be embedded with `#[command(flatten)]` in both
/// CLI subcommands and [`TestRequest`] (the test-suite parser).  Keeping the
/// definition in one place guarantees that the same flag names, short aliases,
/// and parsing rules apply everywhere.
#[derive(Args, Clone)]
pub(crate) struct RequestFlags {
    /// URL to request (may be a bare host — `http://` is prepended if needed)
    pub(crate) base: String,

    /// Path to append to the URL
    #[arg(short, long)]
    pub(crate) path: Option<String>,

    /// Query args as `key=value` (repeat for multiple)
    #[arg(short = 'q', long)]
    pub(crate) query: Vec<String>,

    /// Request headers as `key:value` (repeat for multiple)
    #[arg(long)]
    pub(crate) headers: Vec<String>,

    /// Cookies to send as `key=value` (repeat for multiple)
    #[arg(short, long)]
    pub(crate) cookies: Vec<String>,

    /// JSON body fields as `key=value` — dot notation builds nested objects,
    /// values are auto-typed (numbers, booleans, null, arrays).
    /// Repeat for multiple fields. (repeat for multiple)
    #[arg(short, long)]
    pub(crate) json: Vec<String>,
}

// ── Test-suite request parser ──────────────────────────────────────────────────

/// Parses a single request line from a `.wurl` test file.
///
/// The format mirrors the CLI exactly:
/// ```text
/// <method> <url> [--path <p>] [--query <k=v>]... [--json <k=v>]...
///                [--headers <k:v>]... [--cookies <k=v>]...
/// ```
/// Because this reuses [`RequestFlags`], every flag name, short alias, and
/// value-parsing rule is shared with `wurl get` / `wurl post` on the CLI.
#[derive(Parser)]
#[command(no_binary_name = true)]
pub(crate) struct TestRequest {
    /// HTTP method: get, post, put, patch, delete
    pub(crate) method: String,
    #[command(flatten)]
    pub(crate) args: RequestFlags,
}

impl TestRequest {
    /// Parse a raw test-value string (the tokens stored between the test name
    /// and the first `assert`/`end` by the suite parser) into a typed request.
    pub(crate) fn parse_str(value: &str) -> Result<Self, String> {
        let tokens = shell_split(value);
        Self::try_parse_from(tokens).map_err(|e| e.to_string())
    }
}

/// Split a string into argv-style tokens, handling double-quoted strings and
/// backslash escapes inside them.
///
/// `get "https://api.example.com/users" --query page=1`
/// → `["get", "https://api.example.com/users", "--query", "page=1"]`
pub(crate) fn shell_split(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = s.chars().peekable();

    loop {
        // Skip whitespace between tokens.
        while chars.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
            chars.next();
        }

        match chars.peek() {
            None => break,
            Some(&'"') => {
                chars.next(); // consume opening "
                let mut tok = String::new();
                loop {
                    match chars.next() {
                        None | Some('"') => break,
                        Some('\\') => {
                            if let Some(c) = chars.next() {
                                tok.push(c);
                            }
                        }
                        Some(c) => tok.push(c),
                    }
                }
                tokens.push(tok);
            }
            _ => {
                let mut tok = String::new();
                while chars.peek().map(|c| !c.is_whitespace()).unwrap_or(false) {
                    tok.push(chars.next().unwrap());
                }
                if !tok.is_empty() {
                    tokens.push(tok);
                }
            }
        }
    }

    tokens
}

// ── CLI ────────────────────────────────────────────────────────────────────────

/// A better curl CLI
#[derive(Parser)]
#[command(about = "Wurl is a better curl!", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Make a GET request
    Get {
        #[command(flatten)]
        args: RequestFlags,
    },
    /// Make a POST request
    Post {
        #[command(flatten)]
        args: RequestFlags,
    },
    /// Run a .wurl test suite (pass '-' to read from stdin)
    Test {
        /// Path to the .wurl file, or '-' to read from stdin
        input: String,
    },
    /// Print the cookbook
    Cook,
}

pub async fn start() -> Result<(), Error> {
    let cli = Cli::parse();
    let wurler = Wurler::new();

    match cli.command {
        Commands::Get { args } => {
            let url = format!("{}{}", args.base, args.path.clone().unwrap_or_default());
            let resp = wurler.send(Method::GET, &url, &args).await?;
            match &resp.body_json {
                Some(json) => Wurler::pretty_print(json),
                None => println!("{}", resp.body_raw),
            }
        }

        Commands::Post { args } => {
            let url = format!("{}{}", args.base, args.path.clone().unwrap_or_default());
            let resp = wurler.send(Method::POST, &url, &args).await?;
            match &resp.body_json {
                Some(json) => Wurler::pretty_print(json),
                None => println!("{}", resp.body_raw),
            }
        }

        Commands::Test { input } => {
            let input = if input == "-" {
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf).unwrap();
                buf
            } else {
                input
            };
            run_tests(parse(&preproc(&input)).unwrap()).await?;
        }

        Commands::Cook => println!("{}", include_str!("cookbook.txt")),
    }

    Ok(())
}
