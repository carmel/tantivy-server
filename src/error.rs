use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ServerError {
    msg: String,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for ServerError {
    fn description(&self) -> &str {
        &self.msg
    }
}

// #[derive(Debug)]
// pub struct ServerError(String);
