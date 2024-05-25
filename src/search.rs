use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::lexer;

type TF = HashMap::<String, usize>;
type TFIndex = HashMap::<PathBuf, TF>;

pub fn search() -> io::Result<()>{
    println!("Reading files ..");
    let json_output = File::open("./tf_index.json").unwrap();
    let read : TFIndex = serde_json::from_reader(json_output).unwrap();

    println!("Number of files: {}", read.len());

    let mut input = String::new();

    print!("> ");
    io::stdout().flush().expect("Failed to flush stdout");

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let input = input.trim();

    println!("{}", input);

    let mut result = Vec::<(&str, f32)>::new();

    for (path, tf_table) in &read {
        let input = input.chars().map(|x|x.to_ascii_uppercase()).collect::<Vec<_>>();
        let mut total_tf = 0 as f32;
        for token in lexer::Lexer::new(&input){
            let score = term_freq(&token, &tf_table) * inverse_document_freq(&token, &read);
            total_tf += score;
        }

        result.push((path.to_str().unwrap(), total_tf));
    }

    result.sort_by(|(_, rank1), (_, rank2)| rank1.partial_cmp(rank2).expect(&format!("{rank1} and {rank2} are not comparable")));
    result.reverse();

    for (path, val) in &result[0..10]{
        println!("{} => {}", path, val)
    }


    Ok(())
}

fn term_freq(term: &str, document: &TF) -> f32{
    let mut sum = 1;
    for (_, f) in document{
        sum += f;
    }
    *document.get(term).unwrap_or(&0) as f32  / sum as f32
}

fn inverse_document_freq(term: &str, document: &TFIndex) -> f32 {
    let n = document.len() as f32;

    let mut num_of_occurences_of_t = 1 as f32;

    for (_, document_table) in document{
        if document_table.contains_key(term) {
            num_of_occurences_of_t += 1 as f32;
        }
    }

    if num_of_occurences_of_t == 0f32 {
        num_of_occurences_of_t += 1f32;
    }

    (n / num_of_occurences_of_t).log10()
}