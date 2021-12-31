use std::fs;

use rocket::get;
use rocket::State;
use rocket::http::Status;
use rocket::response::{content, status};
use rocket::serde::json::Json;
use rusqlite::params;

use crate::database::WordDb;

#[get("/frontend")]
pub fn frontend() -> Option<content::Html<String>> {
    match fs::read_to_string("static/index.html") {
        Ok(file) => Some(content::Html(file)),
        Err(_) => None
    }
}

#[get("/langs/<lang>/words/<word>")]
pub fn get_entries(db: &State<WordDb>, lang: &str, word: &str) -> status::Custom<content::Json<String>> {
    let conn = db.connect();

    let mut statement = conn.prepare(&format!(
        "SELECT content
        FROM {}_words
        WHERE word = ?",
        lang)
    ).unwrap();

    let mut rows = statement.query([word]).unwrap();

    let mut words = String::new();

    words.push('[');
    while let Some(row) = rows.next().unwrap() {
        let content: String = row.get(0).unwrap();
        words.push_str(&content);
        words.push(',');
    }
    // Remove last comma
    if words.pop().unwrap() == '[' {
        words.push('[');
    }
    words.push(']');

    status::Custom(Status::Ok, content::Json(words))
}

#[get("/langs/<lang>/words?<like>&<limit>&<offset>")]
pub fn get_entries_like(db: &State<WordDb>, lang: &str, like: &str, limit: usize, offset: usize) -> Json<Vec<String>> {
    let conn = db.connect();

    let mut statement = conn.prepare(&format!(
        "SELECT word
        FROM {}_words
        WHERE word LIKE ?
        ORDER BY length(word) ASC
        LIMIT ?
        OFFSET ?",
        lang)
    ).unwrap();

    let mut rows = statement.query(params![format!("%{}%", like), limit, offset]).unwrap();

    let mut words = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        words.push(row.get(0).unwrap());
    }

    Json(words)
}
