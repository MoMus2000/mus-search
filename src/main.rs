use std::collections::HashMap;
use std::fs::{File, ReadDir};
use std::io::{self, Read};
use std::path::PathBuf;
use serde_json::Result;

#[derive(Debug)]
struct Lexer<'a>{
    content: &'a [char]
}

impl <'a> Lexer<'a>{
    fn new(content: &'a [char]) -> Self{
        Self { content }
    }

    fn trim_left(&mut self){
        while self.content.len() > 0 && self.content[0].is_whitespace(){
            self.content = &self.content[1..]
        }
    }

    fn next_token(&mut self) -> Option<&'a [char]>{
        self.trim_left();

        if self.content.len() == 0 {
            return None
        }

        if self.content[0].is_numeric(){
            let mut n = 0;
            while n < self.content.len() && self.content[n].is_numeric(){
                n += 1;
            }
            let token = &self.content[0..n];
            self.content = &self.content[n..];
            return Some(token)
        }

        if self.content[0].is_alphabetic(){
            let mut n = 0;
            while n < self.content.len() && self.content[n].is_alphanumeric(){
                n += 1;
            }
            let token = &self.content[0..n];
            self.content = &self.content[n..];
            return Some(token)
        }
        let token = &self.content[0..1];
        self.content = &self.content[1..];
        return Some(token)
    }

}

impl<'a> Iterator for Lexer<'a>{
    type Item = &'a [char];

    fn next(&mut self) -> Option<Self::Item>{
        self.next_token()
    }
}

pub fn get_all_files(dir: ReadDir, paths: &mut String){
    for file in dir {
        let file = file.unwrap();
        let file_type = file.file_type().unwrap();
        if file_type.is_dir(){
            let dir = file.path();
            let dir = std::fs::read_dir(dir).unwrap();
            get_all_files(dir, paths);
        } else{
            paths.push_str(file.path().to_str().unwrap());
            paths.push_str("\n")
        }
    }
}

type TF = HashMap::<String, usize>;
type TFIndex = HashMap::<PathBuf, TF>;

pub fn search() -> io::Result<()>{
    println!("Reading files ..");
    let json_output = File::open("./tf_index.json").unwrap();
    let read : TFIndex = serde_json::from_reader(json_output).unwrap();

    println!("Number of files: {}", read.len());

    Ok(())
}

pub fn main() -> io::Result<()>{
    index()?;

    Ok(())
}

pub fn index() -> io::Result<()>{
    let mut paths = String::new();
    let dir = std::fs::read_dir("./data")?;
    get_all_files(dir, &mut paths);
    let mut tf_index = TFIndex::new();
    for path in paths.lines(){
        let mut contents = String::new();
        let mut file = File::open(path)?;
        if let Err(e) = file.read_to_string(&mut contents) {
            eprintln!("Failed to read the file {}: {}", path, e);
            continue;
        }
        let content = contents.chars().collect::<Vec<_>>();
        let mut tf= TF::new();
        for lexer in Lexer::new(&content){
           let content = lexer.iter().map(|x| x.to_ascii_uppercase()).collect::<String>();
           if let Some(freq) = tf.get_mut(&content){
                *freq += 1;
           }else{
                tf.insert(content, 1);
           }
        }

        tf_index.insert(path.into(), tf);

        // let mut stats = tf.iter().collect::<Vec<_>>();
        // stats.sort_by_key(|(_, f)| *f);
        // stats.reverse();

        // println!("File Path: {}", path);
        // for (t, f) in stats{
        //     println!("    {}=>{}", t,f);
        // }

    }

    // for (path, tf) in tf_index{
    //     println!("path {} has unique tokens {}", path.to_str().unwrap(), tf.len())
    // }

    println!("Saving file ..");
    let json_output = File::create("./tf_index.json").unwrap();
    serde_json::to_writer(json_output, &tf_index).unwrap();

    Ok(())
}