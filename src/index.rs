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
            let mut file = std::io::BufReader::new(File::open(path).unwrap());
            if let Err(e) = file.read_to_string(&mut contents) {
                let mut buf = vec![];
                file.read_to_end (&mut buf)?;
                contents = String::from_utf8_lossy (&buf).to_string();
            }
        }

        let mut split_by_paragraph : Vec<&str> = contents.split("\r\n").collect();

        if split_by_paragraph.len() == 1{
            split_by_paragraph  = contents.split("\n").collect();
        }

        let mut i = 0;

        for paragraph in split_by_paragraph{
            let content = paragraph.chars().collect::<Vec<_>>();
            let mut tf= TF::new();
            for lexer in lexer::Lexer::new(&content){
                if let Some(freq) = tf.get_mut(&lexer){
                        *freq += 1;
                }else{
                        tf.insert(lexer, 1);
                }
            }
            let identifier = format!("{}/paragraph/{}", path, i);
            i += 1;
            println!("Indexed: {} with tokens {}", identifier, tf_index.len());
            tf_index.insert(identifier.into(), tf);
        }
    }

    println!("Saving file ..");
    let json_output = std::io::BufWriter::new(File::create("./tf_index.json").unwrap());
    serde_json::to_writer(json_output, &tf_index).unwrap();

    Ok(())
}