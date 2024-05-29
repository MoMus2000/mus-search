use std::io::Result;

mod snowball;
mod lexer;
mod search;
mod index;
mod reverse_search;
mod model;

use std::env;

pub fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <command>", args[0]);
        eprintln!("Commands: index, search");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "index" => index::index(args[2].as_str().to_string())?,
        "search" => search::search()?,
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Commands: index, search");
            std::process::exit(1);
        }
    }

    Ok(())
}
