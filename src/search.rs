use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::io::Write; // Import the necessary modules
use std::sync::Arc;


use crate::reverse_search;
use crate::lexer;
use crate::model;

type TF = HashMap::<String, usize>;
type TFIndex = HashMap::<PathBuf, (usize, TF)>;

pub fn search(_ : Option<String>) -> io::Result<()>{
    println!("Reading files ..");

    let json_output = std::io::BufReader::new(File::open("./tf_index.json").unwrap());
    let doc_freq_json_output = std::io::BufReader::new(File::open("./tf_doc_index.json").unwrap());
    let read : TFIndex = serde_json::from_reader(json_output).unwrap();
    let doc_freq : TF = serde_json::from_reader(doc_freq_json_output).unwrap();

    println!("Number of files: {}", read.len());

    loop {
        print!("> "); // Print the prompt
        io::stdout().flush().unwrap(); // Make sure the prompt is displayed

        let mut input = String::new(); // Create a mutable string to hold the input
        io::stdin().read_line(&mut input).expect("Failed to read line"); // Read the input

        let input = input.trim(); // Trim any whitespace
        if input == "exit" { // Add a way to exit the loop
            break;
        }

        let bar = ProgressBar::new(read.len() as u64);

        bar.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("#>-"));

        let input = input.trim();
        
        let mut result: Vec<(String, f32)> = read.par_iter().map(|(path, (n, tf_table))| {
            let input_chars: Vec<_> = input.chars().collect();
            let mut total_tf = 0.0;
        
            for token in lexer::Lexer::new(&input_chars) {
                let tfs = model::term_freq(&token, *n,&tf_table);
                let itfs = model::inverse_document_freq(&token, read.len(), &doc_freq);
                if tfs.is_nan() || itfs.is_nan(){
                    break;
                }
                let score = tfs * itfs;
                total_tf += score;
            }

            bar.inc(1);
            bar.set_message(format!("Processing item {}", path.to_str().unwrap()));
        
            (path.to_str().unwrap().to_string(), total_tf)
        }).collect();

        bar.finish_with_message("done");

        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        for (path, val) in &result[0..10]{
            println!();
            println!("----------------------------------------------");
            println!("{} => {}", path, val);
            let p = path.to_string();
            let lines: Vec<String> = reverse_search::reverse_search(p)
            .unwrap()
            .lines()
            .take(50)
            .map(String::from)
            .collect();
            let result_string = lines.join("\n");
            println!("{}", result_string);
            println!("----------------------------------------------");
            println!();
        }
    }

    Ok(())
}

pub fn handle_api_search(search_param : Option<String>, read: &Arc<TFIndex>, doc_freq: &Arc<TF>) -> (Vec<String>, Vec<String>){

    let input = search_param.unwrap();

    let input = input.trim(); // Trim any whitespace

    let bar = ProgressBar::new(read.len() as u64);

    bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let input = input.trim();
    
    let mut result: Vec<(String, f32)> = read.par_iter().map(|(path, (n, tf_table))| {
        let input_chars: Vec<_> = input.chars().collect();
        let mut total_tf = 0.0;
    
        for token in lexer::Lexer::new(&input_chars) {
            let tfs = model::term_freq(&token, *n,&tf_table);
            let itfs = model::inverse_document_freq(&token, read.len(), &doc_freq);
            if tfs.is_nan() || itfs.is_nan(){
                break;
            }
            let score = tfs * itfs;
            total_tf += score;
        }

        bar.inc(1);
    
        (path.to_str().unwrap().to_string(), total_tf)
    }).collect();

    bar.finish_with_message("done");

    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut return_vec : Vec<String> = Vec::new();
    let mut return_meta: Vec<String> = Vec::new();

    for (path, score) in &result[0..10]{
        if score > &0f32 {
                let p = path.to_string();
                let lines: Vec<String> = reverse_search::reverse_search(p)
                .unwrap()
                .lines()
                .take(10)
                .map(String::from)
                .collect();
                return_vec.push(lines.join("\n"));
        
                return_meta.push(path.clone());
            };
        }

    (return_vec, return_meta)
}