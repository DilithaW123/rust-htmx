use std::net::SocketAddr;

use axum::{http::StatusCode, response::Html, routing::get, Router};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tera::Tera;
use tower_http::services::ServeDir;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t, // Parse all templates in the "templates" directory
            Err(e) => { // If error -> exit
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html", ".sql"]);
        tera // Return the Tera instance (implicitly)
    };
}

#[tokio::main]
async fn main() {
    println!("Starting server");
    let app = Router::new()
        .route("/", get(index))
        .route("/casetable", get(case_table))
        .route("/cases", get(cases))
        .nest_service("/static", ServeDir::new("static")); //HTTP file directory access
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Serialize, Deserialize)]
struct Case {
    id: i32,
    message: String,
    status: String,
}

async fn case_table() -> (StatusCode, Html<String>) {
    let cases = vec![
        Case {
            id: 1,
            message: "Test".to_string(),
            status: "Open".to_string(),
        },
        Case {
            id: 2,
            message: "Test2".to_string(),
            status: "Closed".to_string(),
        },
    ];
    let mut ctx = tera::Context::new();
    ctx.insert("cases", &cases);
    match TEMPLATES.render("components/casetable.html", &ctx) {
        Ok(s) => (StatusCode::OK, Html(s)),
        Err(e) => {
            println!("Template error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Template error".to_string()),
            )
        }
    }
}

async fn cases() -> (StatusCode, Html<String>) {
    let ctx = tera::Context::new();
    match TEMPLATES.render("cases.html", &ctx) {
        Ok(s) => (StatusCode::OK, Html(s)),
        Err(e) => {
            println!("Template error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Template error".to_string()),
            )
        }
    }
}

async fn index() -> (StatusCode, Html<String>) {
    let ctx = tera::Context::new();
    match TEMPLATES.render("index.html", &ctx) {
        Ok(s) => (StatusCode::OK, Html(s)),
        Err(e) => {
            println!("Template error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Template error".to_string()),
            )
        }
    }
}
