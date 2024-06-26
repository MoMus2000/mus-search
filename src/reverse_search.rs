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

        if path.contains("pdf") {
            use poppler::Document;
            
            let path_string_split: Vec<&str> = path_string.split("/").collect();
            
            let name_i_want = path_string_split[2];
            let current_path_string_split : Vec<&str> = path.split("/").collect();
            let page_number: i32= path_string_split[4].parse::<i32>().unwrap();
            
            let current_name_i_have = current_path_string_split[2];
            
            if name_i_want == current_name_i_have{
                let mut content = Vec::new();
    
                let mut file = std::io::BufReader::new(File::open(path).unwrap());
                file.read_to_end(&mut content).unwrap();
    
                let pdf = Document::from_data(&content, None).map_err(|_| {
                    eprintln!("ERROR: could not read file")
                }).unwrap();

                let content = pdf.page(page_number).unwrap().text().unwrap();
                return Ok(content.to_string())
            }

        }
        else{
            continue
        }
    }
    Ok(String::new())
}