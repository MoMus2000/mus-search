use std::fs::{File, ReadDir};
use std::io::{self, Read};

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
            use poppler::Document;

            let mut content = Vec::new();

            let mut file = std::io::BufReader::new(File::open(path).unwrap());
            file.read_to_end(&mut content).unwrap();

            let pdf = Document::from_data(&content, None).map_err(|_| {
                eprintln!("ERROR: could not read file")
            }).unwrap();

            let n = pdf.n_pages();
            for i in 0..n {
                let page = pdf.page(i).expect(&format!("{i} is within the bounds of the range of the page"));
                if let Some(content) = page.text() {
                    contents.push_str(content.as_str());
                }
            }

            let split_by_paragraph : Vec<&str> = contents.split("\n").collect();
            let name : Vec<&str> = path_string.split("/").collect();
            let path_name: Vec<&str> = path.split("/").collect();

            let title = name[2];
            let loc : i32= name[4].parse::<i32>().unwrap();
            
            if title == path_name[2]{
                println!("{}", split_by_paragraph.len());
                return Ok(split_by_paragraph.get(loc as usize).unwrap().to_string())
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