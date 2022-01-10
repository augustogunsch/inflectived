#[derive(Debug)]
pub struct Language {
    pub code: String,
    pub name: String
}

impl Language {
    pub fn new(code: &str, name: &str) -> Self {
        Self {
            code: String::from(code),
            name: String::from(name)
        }
    }
}
