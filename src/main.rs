use std::io::Result;

mod snowball;
mod lexer;
mod search;
mod index;

pub fn main() -> Result<()>{
    index::index()?;

    search::search()?;

    Ok(())
}
