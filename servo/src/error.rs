use std::fmt;
use std::error::Error;
use std::boxed::Box;

#[derive(Debug)]
pub struct ServoError {
    details: String
}

impl ServoError {
    pub fn new(msg: &str) -> Self {
        Self { details: msg.to_string() }
    }

    pub fn boxnew(msg: &str) -> Box<Self> {
        Box::new(Self::new(msg))
    }
}

impl fmt::Display for ServoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ServoError {
    fn description(&self) -> &str {
        &self.details
    }
}