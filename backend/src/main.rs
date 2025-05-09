use actix_cors::Cors;
use actix_web::{App, HttpServer, Result, error, get, http::header, post, web};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, query, query_as, sqlite::SqlitePoolOptions};

#[derive(Deserialize, Serialize, FromRow)]
struct WordEntry {
    word: String,
    meaning: String,
}

#[get("/api/words")]
async fn get_words(pool: web::Data<SqlitePool>) -> Result<web::Json<Vec<WordEntry>>> {
    let words: Vec<WordEntry> = query_as("select word, meaning from words")
        .fetch_all(pool.as_ref())
        .await
        .map_err(|_| error::ErrorInternalServerError("Failed to fetch words"))?;
    Ok(web::Json(words))
}

#[post("/api/words")]
async fn add_word(pool: web::Data<SqlitePool>, word: web::Json<WordEntry>) -> Result<()> {
    println!("add_word {} {}", word.word, word.meaning);

    query("insert into words(word, meaning) values (?, ?)")
        .bind(&word.word)
        .bind(&word.meaning)
        .execute(pool.as_ref())
        .await
        .map_err(|_| error::ErrorInternalServerError("Failed to insert word"))?;

    Ok(())
}

#[actix_web::main]
async fn main() {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("wordbook.db")
        .await
        .unwrap();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(["GET", "POST"])
            .allowed_headers([
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
                header::ORIGIN,
            ])
            .max_age(3000);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .service(get_words)
            .service(add_word)
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await
    .unwrap()
}
