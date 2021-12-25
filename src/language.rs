#[derive(Debug)]
pub struct Language {
    pub code: String,
    pub types: Vec<String>
}

impl Language {
    pub fn new(code: &str, types: Vec<String>) -> Self {
        Self {
            code: String::from(code),
            types
        }
    }
}
