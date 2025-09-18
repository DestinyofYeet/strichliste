use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub struct DBError(String);

impl DBError {
    pub fn new(error: impl ToString) -> Self {
        DBError(error.to_string())
    }
}

impl Display for DBError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.0)
    }
}
impl From<sqlx::Error> for DBError {
    fn from(value: sqlx::Error) -> Self {
        DBError::new(value)
    }
}
