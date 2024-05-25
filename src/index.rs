use std::collections::HashMap;
use std::fs::{File, ReadDir};
use std::io::{self, Read};
use std::path::PathBuf;
use lopdf::Document;

use crate::lexer;

type TF = HashMap::<String, usize>;
type TFIndex = HashMap::<PathBuf, TF>;

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

pub fn index() -> io::Result<()>{
    let mut paths = String::new();
    let dir = std::fs::read_dir("./data")?;
    get_all_files(dir, &mut paths);
    let mut tf_index = TFIndex::new();
    for path in paths.lines(){
        let mut contents = String::new();
        if path.contains("pdf") {
            let doc = Document::load(path).unwrap();
            let mut full_text = String::new(); 
            for page_id in doc.page_iter() {
                let content = doc.get_page_content(page_id).unwrap();
                let text = String::from_utf8_lossy(&content).to_string();
                full_text.push_str(text.as_str());
            }
            contents = full_text;
        }
        else if path.contains(".git") {
            continue
        }
        else{
            let mut file = File::open(path)?;
            if let Err(e) = file.read_to_string(&mut contents) {
                eprintln!("Failed to read the file {}: {}", path, e);
                let mut buf = vec![];
                file.read_to_end (&mut buf)?;
                contents = String::from_utf8_lossy (&buf).to_string();
            }
        }
        let content = contents.chars().collect::<Vec<_>>();
        let mut tf= TF::new();
        for lexer in lexer::Lexer::new(&content){
           let content = lexer.chars().map(|x| x.to_ascii_uppercase()).collect::<String>();
           if let Some(freq) = tf.get_mut(&content){
                *freq += 1;
           }else{
                tf.insert(content, 1);
           }
        }

        tf_index.insert(path.into(), tf);

        println!("Indexed: {path} with tokens {}", tf_index.len())

    }

    println!("Saving file ..");
    let json_output = File::create("./tf_index.json").unwrap();
    serde_json::to_writer(json_output, &tf_index).unwrap();

    Ok(())
}