use std::{env::var, net::SocketAddr};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use dotenv::dotenv;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool, Postgres};
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
    dotenv().ok();
    let user = var("POSTGRES_USER").unwrap();
    let db = var("POSTGRES_DB").unwrap();
    let password = var("POSTGRES_PASSWORD").unwrap();
    println!("{}", password);
    let connection_string = format!(
        "postgres://{}:{}@host.docker.internal:5432/{}",
        user, password, db
    );

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await
        .unwrap(); // Panic if can't connect to database

    let app = Router::new()
        .route("/", get(index))
        .route("/casetable", get(case_table))
        .route("/cases", get(cases))
        .nest_service("/static", ServeDir::new("static")) //HTTP file directory access
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Serde serialization/deserialization and SQLx database row mapping
#[derive(Serialize, Deserialize, FromRow)]
struct Case {
    id: i32,
    message: String,
    status: String,
}

// Pagination struct for query params
#[derive(Deserialize)]
struct Pagination {
    page: Option<i32>,
}

async fn case_table(
    State(pool): State<PgPool>,
    pagination: Query<Pagination>,
) -> (StatusCode, Html<String>) {
    let mut page = pagination.page.unwrap_or(0);
    if page < 0 {
        page = 0
    }
    let url = format!("/casetable?page={}", page + 1);
    //$N for POSTGRES, ? for MySQL/SQlite
    let cases = sqlx::query_as::<Postgres, Case>("SELECT * FROM cases LIMIT 10 OFFSET $1")
        .bind(page * 10)
        .fetch_all(&pool)
        .await
        .unwrap_or(vec![]); // Query database for cases, if error -> empty vector
    let mut ctx = tera::Context::new();
    ctx.insert("cases", &cases);
    ctx.insert("page", &page);
    ctx.insert("url", &url);
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
