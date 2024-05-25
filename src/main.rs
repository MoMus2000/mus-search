use std::collections::HashMap;
use std::fs::{File, ReadDir};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use lopdf::Document;
mod snowball;

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

    fn next_token(&mut self) -> Option<String>{
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
            return Some(token.iter().collect::<String>())
        }

        if self.content[0].is_alphabetic(){
            let mut n = 0;
            while n < self.content.len() && self.content[n].is_alphanumeric(){
                n += 1;
            }
            let token = &self.content[0..n];
            self.content = &self.content[n..];

            let tok = token.iter().collect::<String>();
            let mut env = crate::snowball::SnowballEnv::create(&tok);
            crate::snowball::algorithms::english_stemmer::stem(&mut env);
            let stemmed_term = env.get_current().to_string();

            return Some(stemmed_term)
        }
        let token = &self.content[0..1];
        self.content = &self.content[1..];
        return Some(token.iter().collect::<String>())
    }

}

impl<'a> Iterator for Lexer<'a>{
    type Item = String;

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
        }
        else{
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
        for token in Lexer::new(&input){
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

pub fn main() -> io::Result<()>{
    // index()?;

    search()?;

    Ok(())
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
        for lexer in Lexer::new(&content){
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