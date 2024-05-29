use std::io::Result;
use std::net::TcpListener;
use actix_web::{http, web, App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;


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

pub async fn serve(tcp_listener : TcpListener) -> Result<()> {
    println!("Serving ..");
    HttpServer::new(move || {
    App::new()
        .route("/api/mus_search", web::post().to(handle_search))
        .wrap(
            Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header(),
        )
    })
    .listen(tcp_listener)?
    .run().await?;
    Ok(())
}

use serde::Deserialize;

// Define the struct that matches the JSON structure
#[derive(Debug, Deserialize)]
struct QueryPayload {
    query: String,
}

async fn handle_search(payload : web::Json<QueryPayload>) -> impl Responder{
    let query = &payload.query;

    let output = search::handle_api_search(Some(query.to_string()));

    let json = serde_json::to_string(&output).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(json)
}