use std::fs;

use reqwest;
use rusqlite::Connection;
use rusqlite::params;

use crate::language::Language;
use crate::entry::WiktionaryEntries;

/// A database of Wiktionary entries
pub struct WordDb {
    connection: String
}

impl WordDb {
    pub fn new(db_path: &str) -> Self {
        Self {
            connection: String::from(db_path)
        }
    }

    pub fn connect(&self) -> Connection {
        Connection::open(&self.connection).unwrap()
    }

    pub fn clean_tables(&mut self, lang: &Language) {
        let mut connection = self.connect();
        let transaction = connection.transaction().unwrap();

        transaction.execute(&format!("DROP TABLE IF EXISTS {}_words", &lang.code), []).unwrap();
        transaction.execute(&format!("DROP TABLE IF EXISTS {}_types", &lang.code), []).unwrap();

        transaction.execute(&format!("
        CREATE TABLE {}_types (
                id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                name TINYTEXT UNIQUE NOT NULL
        )", &lang.code), []).unwrap();

        for type_ in &lang.types {
            transaction.execute(&format!("
            INSERT INTO {}_types ( name )
            VALUES (
                ?
            )", &lang.code), [type_]).unwrap();
        }

        transaction.execute(&format!("
        CREATE TABLE {}_words (
                id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                word TINYTEXT NOT NULL,
                type_id INTEGER NOT NULL,
                content MEDIUMTEXT NOT NULL,
                FOREIGN KEY (type_id)
                    REFERENCES {}_types (id)
        )", &lang.code, &lang.code), []).unwrap();

        transaction.execute(&format!("
        CREATE INDEX word_index
        ON {}_words (word)
        ", &lang.code), []).unwrap();

        transaction.commit().unwrap();
    }

    pub fn insert_entries(&mut self, lang: &Language, entries: WiktionaryEntries) {
        let mut connection = self.connect();
        let transaction = connection.transaction().unwrap();

        for entry in entries {
            transaction.execute(&format!("
            INSERT INTO {}_words ( word, content, type_id )
            VALUES (
                    ?, ?,
                    (SELECT id FROM {}_types WHERE name = ?)
            )", &lang.code, &lang.code),
                params![entry.word,
                        entry.parsed_json.to_string(),
                        entry.type_]
            ).unwrap();
        }

        transaction.commit().unwrap();
    }

    pub async fn upgrade_lang(&mut self, lang: &Language) {
        println!("Trying to read cached data...");
        let cached_data = fs::read_to_string("Polish.json");
        let mut request = None;

        if let Err(_) = cached_data {
            request = Some(reqwest::get("https://kaikki.org/dictionary/Polish/kaikki.org-dictionary-Polish.json"));
        }

        println!("Cleaning tables...");
        self.clean_tables(lang);

        let data;
        if let Some(request) = request {
            // Actually, the request was sent before
            println!("Requesting data...");
            data = request.await.unwrap().text().await.unwrap();
            println!("Caching data...");
            fs::write("Polish.json", &data).unwrap();
        }
        else {
            data = cached_data.unwrap();
        }

        println!("Parsing data...");
        let entries = WiktionaryEntries::parse_data(data);

        println!("Inserting data...");
        self.insert_entries(lang, entries);

        println!("Done");
    }
}
