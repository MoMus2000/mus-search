use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};
use std::thread;
use std::time::Duration;
use std::str::FromStr;
use rayon::prelude::*;


use crate::reverse_search;
use crate::lexer;

type TF = HashMap::<String, usize>;
type TFIndex = HashMap::<PathBuf, TF>;

fn cosine_similarity(vec1: &Vec<f32>, vec2: &Vec<f32>) -> f32 {
    vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum()
}

fn normalize_vector(vector: &Vec<f32>) -> Vec<f32> {
    let norm = (vector.iter().map(|&x| x * x).sum::<f32>()).sqrt();
    if norm == 0.0 {
        vector.clone()
    } else {
        vector.iter().map(|&x| x / norm).collect()
    }
}


fn tfidf_vector(tf_table: &TF, idf: &HashMap<String, f32>) -> Vec<f32> {
    let mut vector = Vec::new();
    for (term, &tf_value) in tf_table {
        let tf = tf_value as f32;
        let idf_value = *idf.get(term).unwrap_or(&0.0);
        vector.push(tf * idf_value);
    }
    vector
}

fn compute_idf(documents: &TFIndex) -> HashMap<String, f32> {
    let mut idf : HashMap<String, f32> = HashMap::new();
    let total_docs = documents.len() as f32;

    for tf_table in documents.values() {
        for term in tf_table.keys() {
            *idf.entry(term.clone()).or_insert(0 as f32) += 1 as f32;
        }
    }

    for (term, count) in idf.iter_mut() {
        *count = (total_docs / *count as f32).log10();
    }

    idf
}

pub fn search(input: String) -> io::Result<()>{
    println!("Reading files ..");
    let json_output = std::io::BufReader::new(File::open("./tf_index.json").unwrap());
    let read : TFIndex = serde_json::from_reader(json_output).unwrap();

    println!("Number of files: {}", read.len());

    let bar = ProgressBar::new(read.len() as u64);

    bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let input = input.trim();
    
    let mut result = Vec::<(&str, f32)>::new();
    
    let idf = compute_idf(&read);

    let mut i = 0;

    let mut result: Vec<(String, f32)> = read.par_iter().map(|(path, tf_table)| {
        let mut query_vec: Vec<f32> = Vec::new();
        let input_chars: Vec<_> = input.chars().collect();
        let mut total_tf = 0.0;
    
        for token in lexer::Lexer::new(&input_chars) {
            let tfs = term_freq(&token, &tf_table);
            let itfs = inverse_document_freq(&token, &read);
            if tfs.is_nan() || itfs.is_nan(){
                continue;
            }
            let score = tfs * itfs;
            total_tf += score;
            query_vec.push(score);
        }
    
        let query_vec_normal = normalize_vector(&query_vec);
        let tf_f32: TF = tf_table.iter().map(|(k, &v)| (k.clone(), v as usize)).collect();
        let doc_vector = normalize_vector(&tfidf_vector(&tf_f32, &idf));
        let similarity = cosine_similarity(&query_vec_normal, &doc_vector);
    
        bar.inc(1);
        bar.set_message(format!("Processing item {}", path.to_str().unwrap()));
    
        (path.to_str().unwrap().to_string(), total_tf)
    }).collect();


    // for (path, tf_table) in &read {

    //     let mut query_vec : Vec<f32> = Vec::new();
    //     let input = input.chars().map(|x|x).collect::<Vec<_>>();
    //     let mut total_tf = 0 as f32;

    //     for token in lexer::Lexer::new(&input){
    //         let tfs = term_freq(&token, &tf_table);
    //         let itfs = inverse_document_freq(&token, &read);
    //         let score = tfs * itfs;
    //         total_tf += score;
    //         query_vec.push(score);
    //     }

    //     let query_vec_normal = normalize_vector(&query_vec);

    //     let tf_f32: TF = tf_table.iter().map(|(k, &v)| (k.clone(), v as usize)).collect();
    //     let doc_vector = normalize_vector(&tfidf_vector(&tf_f32, &idf));

    //     let similarity = cosine_similarity(&query_vec_normal, &doc_vector);

    //     bar.inc(1);
    //     bar.set_message(format!("Processing item {}", i + 1));

    //     result.push((path.to_str().unwrap(), similarity));
    // }

    bar.finish_with_message("done");

    // let idf = compute_idf(&read);

    // let query_vector : Vec<f32> = result.iter().map(|(_, val)| *val).collect();
    // let query_vector_normalized = normalize_vector(&query_vector);

    // let mut result = Vec::<(&str, f32)>::new();

    // for (path, tf_table) in &read {
    //     let tf_f32: TF = tf_table.iter().map(|(k, &v)| (k.clone(), v as usize)).collect();
    //     let doc_vector = normalize_vector(&tfidf_vector(&tf_f32, &idf));
    //     let similarity = cosine_similarity(&query_vector_normalized, &doc_vector);
    //     println!("Similarity {}", similarity);
    //     result.push((path.to_str().unwrap(), similarity));
    // }

    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (path, val) in &result[0..10]{
        println!();
        println!("----------------------------------------------");
        println!("{} => {}", path, val);
        let p = path.to_string();
        let lines: Vec<String> = reverse_search::reverse_search(p)
        .unwrap()
        .lines()
        .take(10)
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