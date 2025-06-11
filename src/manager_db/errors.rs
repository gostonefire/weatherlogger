use std::fmt;
use std::fmt::Formatter;

pub struct DBError(pub String);

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DBError: {}", self.0)
    }
}
impl From<rusqlite::Error> for DBError {
    fn from(err: rusqlite::Error) -> Self { DBError(err.to_string()) }
}
impl From<serde_json::Error> for DBError {
    fn from(err: serde_json::Error) -> Self { DBError(err.to_string()) }
}
impl From<chrono::format::ParseError> for DBError {
    fn from(err: chrono::format::ParseError) -> Self { DBError(err.to_string()) }
}