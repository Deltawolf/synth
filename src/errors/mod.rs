use std::fmt;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct NoResultError {
    details: String
}


impl NoResultError {
    pub fn new(msg: &str) -> NoResultError {
        NoResultError{details: msg.to_string()}
    }
}

impl fmt::Display for NoResultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for NoResultError {
    fn description(&self) -> &str {
        &self.details
    }
}
