use std::fs;
use std::fs::File;
use std::collections::HashSet;
use std::fmt;

use reqwest;
use rusqlite::{Connection, Transaction, ErrorCode};
use rusqlite::Error::SqliteFailure;
use rusqlite::params;
use serde_json::Value;
use serde_json::json;
use serde_json;

use crate::language::Language;
use crate::entry::{WiktionaryEntries, WiktionaryEntry};
use crate::entry::Form;
use crate::{DB_DIR, CACHE_DIR, MAJOR, MINOR, PATCH};
use crate::util;

pub enum DbError {
    AccessDenied,
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::AccessDenied => write!(f, "Access denied"),
        }
    }
}

/// A database of Wiktionary entries
pub struct WordDb {
    db_path: String,
    pub installed_langs: Vec<Language>,
    pub installable_langs: Vec<Language>
}

impl WordDb {
    pub fn new(db_name: &str) -> Self {
        let db_path = format!("{}/{}", DB_DIR, db_name);

        let conn = Connection::open(&db_path).unwrap();

        let mut installed_langs: Vec<Language> = Vec::new();

        let statement = conn.prepare(
            "SELECT code, name, major, minor, patch
            FROM langs"
        );

        if let Ok(mut statement) = statement {
            let mut rows = statement.query([]).unwrap();

            while let Some(row) = rows.next().unwrap() {
                installed_langs.push(Language::from_row(&row));
            }

            installed_langs.sort();
        }

        Self {
            db_path,
            installed_langs,
            installable_langs: Language::list_langs(),
        }
    }

    pub fn connect(&self) -> Connection {
        Connection::open(&self.db_path).unwrap()
    }

    pub fn list_available(&self) -> String {
        let mut list = String::new();

        for lang in &self.installable_langs {
            list.push_str(&format!(" - {} ({})\n", &lang.name, &lang.code));
        }

        list
    }

    pub fn get_lang(&self, code: &str) -> Option<Language> {
        for lang in &self.installable_langs {
            if lang.code == code {
                return Some(lang.clone())
            }
        }

        None
    }

    pub fn clean_tables(&mut self, lang: &Language) -> Result<(), DbError> {
        let mut conn = self.connect();
        let transaction = conn.transaction().unwrap();

        if let Err(e) = transaction.execute("
        CREATE TABLE IF NOT EXISTS langs (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            code TINYTEXT UNIQUE NOT NULL,
            name TINYTEXT NOT NULL,
            major INTEGER NOT NULL,
            minor INTEGER NOT NULL,
            patch INTEGER NOT NULL
        )", []) {
            match e {
                SqliteFailure(f, _) => match f.code {
                    ErrorCode::ReadOnly => {
                        return Err(DbError::AccessDenied)
                    },
                    _ => panic!("{}", e)
                },
                _ => panic!("{}", e)
            }
        }

        if let Err(e) = transaction.execute(
            "DELETE FROM langs WHERE code = ?", [&lang.code]) {
            match e {
                SqliteFailure(f, _) => match f.code {
                    ErrorCode::ReadOnly => {
                        return Err(DbError::AccessDenied)
                    },
                    _ => panic!("{}", e)
                },
                _ => panic!("{}", e)
            }
        };

        transaction.execute(&format!("DROP TABLE IF EXISTS {0}_words", &lang.code), []).unwrap();
        transaction.execute(&format!("DROP TABLE IF EXISTS {0}_types", &lang.code), []).unwrap();

        transaction.execute(&format!("
        CREATE TABLE {0}_types (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            name TINYTEXT UNIQUE NOT NULL
        )", &lang.code), []).unwrap();

        transaction.execute(&format!("
        CREATE TABLE {0}_words (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            word TINYTEXT NOT NULL,
            type_id INTEGER NOT NULL,
            content MEDIUMTEXT NOT NULL,
            FOREIGN KEY (type_id)
                REFERENCES {0}_types (id)
        )", &lang.code), []).unwrap();

        transaction.execute(&format!("
        CREATE INDEX {0}_word_index
        ON {0}_words (word)
        ", &lang.code), []).unwrap();

        transaction.commit().unwrap();

        Ok(())
    }

    pub fn insert_entry(&self, transaction: &Transaction, lang: &Language, entry: &WiktionaryEntry) {
        transaction.execute(&format!("
        INSERT INTO {0}_words ( word, content, type_id )
        VALUES (
                ?, ?,
                (SELECT id FROM {0}_types WHERE name = ?)
        )", &lang.code),
            params![entry.word,
                    entry.unparsed_json,
                    entry.type_]
        ).unwrap();
    }

    pub fn insert_entries(&mut self, lang: &Language, entries: &WiktionaryEntries) {
        let mut conn = self.connect();
        let transaction = conn.transaction().unwrap();

        for entry in entries.iter() {
            self.insert_entry(&transaction, lang, entry);
        }

        transaction.commit().unwrap();
    }

    /// Generate missing "form-of" entries
    pub fn generate_entries(&mut self, lang: &Language, entries: &WiktionaryEntries) {
        let mut conn = self.connect();
        let transaction = conn.transaction().unwrap();

        let mut statement = transaction.prepare(&format!(
            "SELECT {0}_words.content
            FROM {0}_words
            JOIN {0}_types
            ON {0}_types.id = {0}_words.type_id
            WHERE {0}_words.word = ?
            AND {0}_types.name = ?", &lang.code)
        ).unwrap();

        for entry in entries.iter() {
            if let Some(forms) = entry.parse_json()["forms"].as_array() {
                let mut forms_vec: Vec<Form> = Vec::new();

                for form in forms {
                    let form: Form = serde_json::from_value(form.clone()).unwrap();
                    forms_vec.push(form);
                }

                forms_vec = forms_vec.into_iter()
                                     .filter(|x|
                                         match &x.source {
                                             Some(src) => src == "Declension" || src == "Conjugation",
                                             None => false
                                         }
                                     ).collect();

                forms_vec.sort_by_key(|x| x.form.clone());

                let forms_group = forms_vec.group_by(|a, b| a.form == b.form);

                for forms in forms_group.into_iter() {
                    let mut entries = statement.query([&forms[0].form, &entry.type_]).unwrap();

                    if let None = entries.next().unwrap() {
                        let mut senses: Vec<Value> = Vec::new();

                        for form in forms {
                            let mut tags = match &form.tags {
                                Some(tags) => tags.clone(),
                                None => Vec::new()
                            };
                            tags.push(String::from("form-of"));
                            tags.push(String::from("auto-generated"));

                            senses.push(json!({
                                "form_of": [
                                    {
                                        "word": entry.word
                                    }
                                ],
                                "glosses": [
                                    match &form.tags {
                                        Some(tags) => tags.join(" "),
                                        None => String::from("")
                                    }
                                ],
                                "tags": tags
                            }));
                        }

                        let entry_json = json!({
                            "pos": entry.type_.clone(),
                            "word": forms[0].form.clone(),
                            "senses": senses
                        });

                        let new_entry = WiktionaryEntry::new(forms[0].form.clone(),
                                                             entry.type_.clone(),
                                                             entry_json.to_string());

                        self.insert_entry(&transaction, lang, &new_entry);
                    }
                }
            }
        }

        drop(statement);
        transaction.commit().unwrap();
    }

    fn insert_types(&mut self, lang: &Language, entries: &WiktionaryEntries) {
        let mut conn = self.connect();
        let transaction = conn.transaction().unwrap();

        let mut types = HashSet::new();

        for entry in entries.iter() {
            types.insert(&entry.type_);
        }

        for type_ in types {
            transaction.execute(&format!("
            INSERT INTO {0}_types ( name )
            VALUES (?)", &lang.code), [type_]).unwrap();
        }

        transaction.commit().unwrap();
    }

    fn insert_version(&mut self, lang: &Language) {
        let mut conn = self.connect();
        let transaction = conn.transaction().unwrap();

        transaction.execute("
        INSERT INTO langs (code, name, major, minor, patch)
        VALUES (?, ?, ?, ?, ?)
        ", params![&lang.code, &lang.name, MAJOR, MINOR, PATCH]).unwrap();

        transaction.commit().unwrap();
    }

    pub async fn upgrade_lang(&mut self, lang: &Language) -> Result<(), DbError> {
        util::try_create_dir(DB_DIR);

        println!("Trying to read cached data...");
        let cache_file = format!("{}/{}.json", CACHE_DIR, &lang.name);

        let mut cached_data = File::open(&cache_file);
        let mut request = None;

        if let Err(_) = cached_data {
            let url = format!("https://kaikki.org/dictionary/{0}/kaikki.org-dictionary-{0}.json",
                              &lang.name);
            request = Some(reqwest::get(url));
        }

        println!("Cleaning tables...");
        self.clean_tables(lang)?;

        if let Some(request) = request {
            // Actually, the request was sent before
            println!("Requesting data...");
            let data = request.await.unwrap().text().await.unwrap();

            println!("Caching data...");
            util::try_create_dir(CACHE_DIR);
            fs::write(&cache_file, &data).unwrap();

            cached_data = File::open(&cache_file);
        }

        println!("Parsing data...");
        let entries = WiktionaryEntries::parse_data(cached_data.unwrap());

        println!("Inserting types...");
        self.insert_types(lang, &entries);

        println!("Inserting entries...");
        self.insert_entries(lang, &entries);

        println!("Generating \"form-of\" entries...");
        self.generate_entries(lang, &entries);

        println!("Inserting version...");
        self.insert_version(lang);

        println!("Done");

        Ok(())
    }
}
