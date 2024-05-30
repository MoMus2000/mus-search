use std::io::Result;
use std::net::TcpListener;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Arc;
use actix_files::NamedFile;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use mime_guess::from_path;
use indicatif::ProgressFinish;
use indicatif::{ProgressBar, ProgressStyle};

type TF = HashMap::<String, usize>;
type TFIndex = HashMap::<PathBuf, (usize, TF)>;

mod snowball;
mod lexer;
mod search;
mod index;
mod reverse_search;
mod model;

use std::env;

pub async fn run(listener: TcpListener) -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <command>", args[0]);
        eprintln!("Commands: index, search");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "index" => index::index(args[2].as_str().to_string())?,
        "search" => search::search(None)?,
        "serve" => serve(listener).await?,
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Commands: index, search");
            std::process::exit(1);
        }
    }

    Ok(())
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6969")
        .expect("Failed to bind port 6969");
    run(listener).await.unwrap();
    Ok(())
}

struct AppState {
    read: Arc<TFIndex>,
    doc_freq: Arc<TF>,
}

pub async fn serve(tcp_listener : TcpListener) -> Result<()> {
    let json_output = std::io::BufReader::new(File::open("./tf_index.json").unwrap());
    let doc_freq_json_output = std::io::BufReader::new(File::open("./tf_doc_index.json").unwrap());
    let read : TFIndex = serde_json::from_reader(json_output).unwrap();
    let doc_freq : TF = serde_json::from_reader(doc_freq_json_output).unwrap();

    let app_state = web::Data::new(AppState {
        read: Arc::new(read),
        doc_freq: Arc::new(doc_freq),
    });


    let title = r#"
     __  __             _____     
    |  \/  |_   _ ___  / ___|  ___  __ _ _ __ ___| |__  
    | |\/| | | | / __| \___ \ / _ \/ _` | '__/ __| '_ \ 
    | |  | | |_| \__ \  ___) |  __/ (_| | | | (__| | | |
    |_|  |_|\__,_|___/ |____/ \___|\__,_|_|  \___|_| |_|"#;

    println!("{}", title);

    let logo = r#"
                     _
                    / \      _-'
                _/|  \-''- _ /
                __-' { |          \
                /             \
                /       "o.  |o }
                |            \ ;
                            ',
                    \_         __\
                    ''-_    \.//
                        / '-____'
                    /
                    _'
                _-'"#;

    println!("{}", logo);
    println!("Loaded index ..");
    println!("Serving @{}", tcp_listener.local_addr().unwrap());


    HttpServer::new(move || {
        App::new()
        .app_data(app_state.clone())
        .route("/api/mus_search", web::post().to(handle_search))
        .route("/api/files/{filename:.*}", web::get().to(serve_file))
        .service(actix_files::Files::new("/", "./src/ui").index_file("index.html"))
    })
    .listen(tcp_listener)?
    .run().await?;
    Ok(())
}

use serde::{Deserialize, Serialize};

// Define the struct that matches the JSON structure
#[derive(Debug, Deserialize)]
struct QueryPayload {
    query: String,
}

async fn handle_search(payload : web::Json<QueryPayload>, data: web::Data<AppState>) -> impl Responder{
    let read = &data.read;
    let doc_freq = &data.doc_freq;

    let query = &payload.query;

    let bar = ProgressBar::new(read.len() as u64);

    bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let output = search::handle_api_search(&bar, Some(query.to_string()), read, doc_freq);

    #[derive(Debug, Serialize)]
    struct ResponsePayload{
        response: Vec<String>,
        meta_data: Vec<String>
    }

    let response_payload = ResponsePayload{
        response: output.0,
        meta_data: output.1
    };

    let json = serde_json::to_string(&response_payload).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(json)
}

async fn serve_file(path: web::Path<(String,)>) -> Result<NamedFile> {
    let file_path: PathBuf = format!("./data/{}", path.into_inner().0).parse().unwrap();

    // Check if the file exists and is a file (not a directory)
    if file_path.is_file() {
        let mime_type = from_path(&file_path).first_or_octet_stream();
        let mut named_file = NamedFile::open(&file_path)?;
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        named_file = named_file.set_content_disposition(actix_web::http::header::ContentDisposition {
            disposition: actix_web::http::header::DispositionType::Inline,
            parameters: vec![actix_web::http::header::DispositionParam::Filename(file_name)],
        });
        
        named_file = named_file.set_content_type(mime_type);

        Ok(named_file)
    } else {
        Ok(NamedFile::open("./static/404.html")?)
    }
}