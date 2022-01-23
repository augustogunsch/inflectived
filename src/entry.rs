use std::fs::File;
use std::io::{BufRead, BufReader};
use std::cmp;
use std::slice::Iter;
use serde_json::Value;
use serde::Deserialize;

#[derive (Clone, Debug)]
pub struct WiktionaryEntry {
    pub word: String,
    pub type_: String,
    pub unparsed_json: String
}

impl cmp::PartialEq for WiktionaryEntry {
    fn eq(&self, other: &Self) -> bool {
        self.word.eq(&other.word)
    }
}

impl cmp::Eq for WiktionaryEntry {}

impl cmp::PartialOrd for WiktionaryEntry {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for WiktionaryEntry {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.word.cmp(&other.word)
    }
}

impl WiktionaryEntry {
    pub fn parse(unparsed_json: &str) -> Self {
        // We could keep this in memory, but for bigger language databases
        // it's going to crash the program
        let json: Value = serde_json::from_str(unparsed_json).unwrap();

        let word = String::from(json["word"].as_str().unwrap());
        let type_ = String::from(json["pos"].as_str().unwrap());

        Self {
            word,
            type_,
            unparsed_json: String::from(unparsed_json)
        }
    }

    pub fn new(word: String, type_: String, unparsed_json: String) -> Self {
        Self {
            word,
            type_,
            unparsed_json
        }
    }

    pub fn parse_json(&self) -> Value {
        serde_json::from_str(&self.unparsed_json).unwrap()
    }
}

pub struct WiktionaryEntries(Vec<WiktionaryEntry>);

impl WiktionaryEntries {
    pub fn parse_data(data: File) -> Self {
        let reader = BufReader::new(data);

        let mut entries: Vec<WiktionaryEntry> = Vec::new();

        for line in reader.lines() {
            entries.push(WiktionaryEntry::parse(&line.unwrap()));
        }

        Self(entries)
    }

    pub fn iter(&self) -> Iter<WiktionaryEntry> {
        self.0.iter()
    }
}

#[derive(Debug, Deserialize)]
pub struct Form {
    pub form: String,
    pub tags: Option<Vec<String>>,
    pub source: Option<String>,
}

