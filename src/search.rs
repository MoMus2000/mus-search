use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::reverse_search;
use crate::lexer;

type TF = HashMap::<String, usize>;
type TFIndex = HashMap::<PathBuf, TF>;

pub fn search(input: String) -> io::Result<()>{
    println!("Reading files ..");
    let json_output = std::io::BufReader::new(File::open("./tf_index.json").unwrap());
    let read : TFIndex = serde_json::from_reader(json_output).unwrap();

    println!("Number of files: {}", read.len());

    let input = input.trim();
    
    let mut result = Vec::<(&str, f32)>::new();
    
    for (path, tf_table) in &read {
        let input = input.chars().map(|x|x.to_ascii_uppercase()).collect::<Vec<_>>();
        let mut total_tf = 0 as f32;
        for token in lexer::Lexer::new(&input){
            let tfs = term_freq(&token, &tf_table);
            let itfs = inverse_document_freq(&token, &read);
            if tfs.is_nan() || itfs.is_nan(){
                break
            }
            let score = tfs * itfs;
            total_tf += score;
        }

        result.push((path.to_str().unwrap(), total_tf));
    }

    result.sort_by(|(_, rank1), (_, rank2)| rank1.partial_cmp(rank2).expect(&format!("{rank1} and {rank2} are not comparable")));
    result.reverse();

    for (path, val) in &result[0..10]{
        println!();
        println!("----------------------------------------------");
        println!("{} => {}", path, val);
        let p = path.to_string();
        let lines: Vec<String> = reverse_search::reverse_search(p)
        .unwrap()
        .lines()
        .take(5)
        .map(String::from)
        .collect();
        let result_string = lines.join("\n");
        println!("{}", result_string);
        println!("----------------------------------------------");
        println!();
    }


    Ok(())
}

fn term_freq(term: &str, document: &TF) -> f32{
    let a = document.get(term).cloned().unwrap_or(0) as f32;
    let b = document.iter().map(|(_, f)| *f).sum::<usize>() as f32;
    a / b
}

fn inverse_document_freq(term: &str, document: &TFIndex) -> f32 {
    let d = document.len() as f32;
    let m = document.values().filter(|tf| tf.contains_key(term)).count().max(1) as f32;
    (d / m).log10()
}