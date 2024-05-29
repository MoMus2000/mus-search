use std::collections::HashMap;
use std::fs::{File, ReadDir};
use std::io::{self, Read};
use std::path::PathBuf;

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

pub fn index() -> io::Result<()>{
    println!("Indexing ..");
    let mut paths = String::new();
    let dir = std::fs::read_dir("./data")?;
    get_all_files(dir, &mut paths);
    let mut tf_index = TFIndex::new();
    let mut doc_freq = DocFreq::new();
    for path in paths.lines(){
        println!("{}", path);
        let mut contents = String::new();
        if path.contains("pdf") {
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
                    println!("{}", content);
                    contents.push_str(content.as_str());
                }
            }
        }
        else if path.contains(".git") {
            continue
        }
        else{
            let mut file = std::io::BufReader::new(File::open(path).unwrap());
            if let Err(_) = file.read_to_string(&mut contents) {
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
            i += 1;
            println!("Indexed: {} with tokens {}", identifier, tf_index.len());
            tf_index.insert(identifier.into(), (count, tf));
        }
    }

    for (a, (_, c)) in &tf_index{
        println!("{} {}", a.to_str().unwrap(), c.len());
    }

    println!("Saving file ..");
    let json_output = std::io::BufWriter::new(File::create("./tf_index.json").unwrap());
    let doc_freq_json_output = std::io::BufWriter::new(File::create("./tf_doc_index.json").unwrap());
    serde_json::to_writer(json_output, &tf_index).unwrap();
    serde_json::to_writer(doc_freq_json_output, &doc_freq).unwrap();

    Ok(())
}