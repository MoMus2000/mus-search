use std::collections::HashMap;
use std::fs::{File, ReadDir};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::net::TcpStream;
use serde::Serialize;

use crate::lexer;

type TF = HashMap::<String, usize>;
type TFIndex = HashMap::<PathBuf, (usize, TF)>;
type DocFreq = TF;

pub fn get_all_files(dir: ReadDir, paths: &mut String){
    for file in dir {
        let file = file.unwrap();
        let file_type = file.file_type().unwrap();
        if file_type.is_dir(){
            let dir = file.path();
            let dir = std::fs::read_dir(dir).unwrap();
            get_all_files(dir, paths);
        }
        else{
            paths.push_str(file.path().to_str().unwrap());
            paths.push_str("\n")
        }
    }
}

fn index_pdf(mut tf_index : TFIndex, mut doc_freq: DocFreq, path: &str) -> ( TFIndex, DocFreq ) {
    use poppler::Document;
    let mut content = Vec::new();

    File::open(path)
        .and_then(|mut file| file.read_to_end(&mut content))
        .map_err(|_| {
            eprintln!("ERROR: could not read file");
        }).unwrap();

    let pdf = Document::from_data(&content, None).map_err(|_| {
        eprintln!("ERROR: could not read file")
    }).unwrap();

    let n = pdf.n_pages();
    for i in 0..n {
        let page = pdf.page(i).expect(&format!("{i} is within the bounds of the range of the page"));
        if let Some(content) = page.text() {
            let mut count = 0;
            let mut tf= TF::new();
            let content = content.chars().collect::<Vec<_>>();
            for lexer in lexer::Lexer::new(&content){
                if let Some(freq) = tf.get_mut(&lexer){
                        *freq += 1;
                }else{
                        tf.insert(lexer, 1);
                }
                count += 1;
            }
            for t in tf.keys() {
                if let Some(f) = doc_freq.get_mut(t) {
                    *f += 1;
                } else {
                    doc_freq.insert(t.to_string(), 1);
                }
            }
            let identifier = format!("{}/page/{}", path, i);
            tf_index.insert(identifier.into(), (count, tf));
        }
    }
    ( tf_index, doc_freq )
}

fn index_text(mut tf_index : TFIndex, mut doc_freq: DocFreq, path: &str) -> (TFIndex, DocFreq){
    let mut contents = String::new();
    let mut file = std::io::BufReader::new(File::open(path).unwrap());
    if let Err(_) = file.read_to_string(&mut contents) {
        let mut buf = vec![];
        file.read_to_end (&mut buf).unwrap();
        contents = String::from_utf8_lossy (&buf).to_string();
    }
    let mut split_by_paragraph : Vec<&str> = contents.split("\r\n").collect();

    if split_by_paragraph.len() > 100{
        split_by_paragraph  = contents.split("\n").collect();
    }

    let mut i = 0;

    for paragraph in split_by_paragraph{
        let content = paragraph.chars().collect::<Vec<_>>();
        let mut tf= TF::new();
        let mut count = 0 ;
        for lexer in lexer::Lexer::new(&content){
            if let Some(freq) = tf.get_mut(&lexer){
                    *freq += 1;
            }else{
                    tf.insert(lexer, 1);
            }
            count += 1;
        }

        for t in tf.keys() {
            if let Some(f) = doc_freq.get_mut(t) {
                *f += 1;
            } else {
                doc_freq.insert(t.to_string(), 1);
            }
        }

        let identifier = format!("{}/paragraph/{}", path, i);
        // println!("Indexed: {} with tokens {}", identifier, tf_index.len());
        tf_index.insert(identifier.into(), (count, tf));
        i += 1;
    }

    ( tf_index, doc_freq )
}

async fn index_docx(tf_index : TFIndex, doc_freq: DocFreq, path: &str) -> (TFIndex, DocFreq){
    use reqwest::Client;

    let client = Client::new();

    // Read the file content
    let mut file = std::io::BufReader::new(File::open(path).unwrap());
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content).unwrap();

    // Prepare the request
    let response = client.post("http://localhost:2004/convert?convertTo=pdf")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(file_content)
        .send()
        .await
        .unwrap();

    if !response.status().is_success(){
        eprintln!("Error parsing file {} .. ", path);
        return (tf_index, doc_freq);
    }

    // Read the response body and write it to a file
    let path = path.replace("docx", "pdf");

    let mut output_file = std::io::BufWriter::new(File::create(&path).unwrap());
    let mut response_body = response.bytes().await.unwrap();
    output_file.write_all(&mut response_body).unwrap();


    let result  = index_pdf(tf_index, doc_freq, &path);

    ( result.0 , result.1 )
}

async fn index_ppt(tf_index : TFIndex, doc_freq: DocFreq, path: &str) -> (TFIndex, DocFreq){
    use reqwest::blocking::Client;

    let client = Client::new();

    // Read the file content
    let mut file = std::io::BufReader::new(File::open(path).unwrap());
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content).unwrap();

    // Prepare the request
    let response = client.post("http://localhost:2004/convert?convertTo=pdf")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(file_content)
        .send()
        .unwrap();

    if !response.status().is_success(){
        eprintln!("Error parsing file {} .. ", path);
        return (tf_index, doc_freq);
    }

    let path = path.replace("docx", "pdf");

    let result  = index_pdf(tf_index, doc_freq, &path);

    ( result.0 , result.1 )
}

pub async fn index(index_folder: String) -> io::Result<()>{

    match TcpStream::connect("127.0.0.1:2004") {
        Ok(_) => {
            println!("Conversion server is running");
        }
        Err(_) => {
            panic!("Conversion server @ port 2003 is down");
        }
    }
    
    println!("Indexing ..");

    let mut paths = String::new();
    let dir = std::fs::read_dir(index_folder)?;

    get_all_files(dir, &mut paths);

    let mut tf_index = TFIndex::new();
    let mut doc_freq = DocFreq::new();

    for path in paths.lines(){

        println!("{}", path);

        if path.contains(".pdf") {
            (tf_index, doc_freq) = index_pdf(tf_index, doc_freq, path);
        }

        else if path.contains(".git") {
            continue
        }

        else if path.contains(".txt") {
            (tf_index, doc_freq) = index_text(tf_index, doc_freq, path);
        }

        else if path.contains(".docx") || path.contains("doc"){
            (tf_index, doc_freq) = index_docx(tf_index, doc_freq, path).await;
        }

        else if path.contains(".ppt"){
            (tf_index, doc_freq) = index_ppt(tf_index, doc_freq, path).await;
        }

    }

    println!("Saving file ..");

    write_index("./tf_index.json", &tf_index);
    write_index("./tf_doc_index.json", &doc_freq);

    Ok(())
}

fn write_index<T: Serialize>(file_name: &str, file_to_write: T){
    let json_output = std::io::BufWriter::new(File::create(file_name).unwrap());
    serde_json::to_writer(json_output, &file_to_write).unwrap();
}