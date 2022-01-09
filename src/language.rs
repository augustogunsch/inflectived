#[derive(Debug)]
pub struct Language {
    pub code: String,
    pub name: String,
    pub types: Vec<String>
}

impl Language {
    pub fn new(code: &str, name: &str, types: Vec<String>) -> Self {
        Self {
            code: String::from(code),
            name: String::from(name),
            types
        }
    }
}
