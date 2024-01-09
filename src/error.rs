use std::error::Error;
use std::fmt;

/// A Raptobo Error.
#[derive(Debug)]
pub struct RaptoboError {
    /// The error description.
    details: String
}

impl RaptoboError {
    
    /// Create a new error with the given message as description.
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
