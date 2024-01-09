use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct RaptoboError {
    details: String
}

impl RaptoboError {
    pub fn new(msg: &str) -> RaptoboError {
        RaptoboError{details: msg.to_string()}
    }
}

impl fmt::Display for RaptoboError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for RaptoboError {
    fn description(&self) -> &str {
        &self.details
    }
}
