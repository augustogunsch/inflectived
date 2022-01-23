use std::cmp::{PartialEq, PartialOrd, Ordering};

use rusqlite::Row;
use serde::Serialize;

use crate::version::Version;

#[derive(Serialize, Debug, Clone)]
pub struct Language {
    pub code: String, // ISO 639-2
    pub name: String, // English name
    pub version: Option<Version>
}

impl Language {
    pub fn new(code: &str, name: &str) -> Self {
        Self {
            code: String::from(code),
            name: String::from(name),
            version: None
        }
    }

    pub fn from_row(row: &Row) -> Self {
        Self {
            code: row.get(0).unwrap(),
            name: row.get(1).unwrap(),
            version: Some(Version(row.get(2).unwrap(),
                                  row.get(3).unwrap(),
                                  row.get(4).unwrap()))
        }
    }

    pub fn list_langs() -> Vec<Self> {
        // Keep this list sorted by name
        let langs = vec![
            Self::new("eng", "English"),
            Self::new("fre", "French"),
            Self::new("ger", "German"),
            Self::new("ita", "Italian"),
            Self::new("pol", "Polish"),
            Self::new("por", "Portuguese"),
            Self::new("rus", "Russian"),
            Self::new("spa", "Spanish"),
        ];

        langs
    }
}

impl PartialEq for Language {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Language {}

impl Ord for Language {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Language {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}
