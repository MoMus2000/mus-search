use std::fs::{File, ReadDir};
use std::io::{self, Read};
use lopdf::Document;

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

pub fn reverse_search(path_string: String) -> io::Result<String>{
    let mut paths = String::new();
    let dir = std::fs::read_dir("./data")?;
    get_all_files(dir, &mut paths);
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
            if let Err(_) = file.read_to_string(&mut contents) {
                let mut buf = vec![];
                file.read_to_end (&mut buf)?;
                contents = String::from_utf8_lossy(&buf).to_string();

            }
        }
        let mut split_by_paragraph : Vec<&str> = contents.split("\r\n").collect();
        if split_by_paragraph.len() == 1{
            split_by_paragraph  = contents.split("\n").collect();
        }

        let name : Vec<&str> = path_string.split("/").collect();
        let path_name: Vec<&str> = path.split("/").collect();

        let title = name[2];
        let loc : i32= name[4].parse::<i32>().unwrap();
        
        if title == path_name[2]{
            println!("{}", split_by_paragraph.len());
            return Ok(split_by_paragraph.get(loc as usize).unwrap().to_string())
        }

    }

    Ok(String::new())
}