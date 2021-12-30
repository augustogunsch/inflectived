use std::cmp;
use std::iter::IntoIterator;
use json::JsonValue::{Object, Short, Array};
use json::JsonValue;

#[derive (Clone)]
#[derive (Debug)]
pub struct WiktionaryEntry {
    pub word: String,
    pub type_: String,
    pub parsed_json: JsonValue
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
        let json = json::parse(unparsed_json).unwrap();

        let (word, type_) = match &json {
            Object(o) => ( 
                match o.get("word") {
                    Some(w) => match w {
                        Short(s) => s.to_string(),
                        JsonValue::String(s) => s.clone(),
                        _ => panic!("Not a string: {}", w.pretty(8))
                    },
                    None => panic!("No field 'word': {}", o.pretty(8))
                },
                match o.get("pos") {
                    Some(w) => match w {
                        Short(s) => s.to_string(),
                        JsonValue::String(s) => s.clone(),
                        _ => panic!("Not a string: {}", w.pretty(8))
                    },
                    None => panic!("No field 'pos': {}", o.pretty(8))
                }
            ),
            _ => panic!("Not an object: {}", json.pretty(8))
        };

        Self {
            word,
            type_,
            parsed_json: json
        }
    }
}

pub struct WiktionaryEntries(Vec<WiktionaryEntry>);

impl WiktionaryEntries {
    pub fn parse_data(data: String) -> Self {
        let mut entries: Vec<WiktionaryEntry> = Vec::new();

        for line in data.lines() {
            entries.push(WiktionaryEntry::parse(line));
        }

        Self(entries)
    }
}

impl IntoIterator for WiktionaryEntries {
    type Item = WiktionaryEntry;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
