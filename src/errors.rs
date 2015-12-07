use std::error::Error;
use std::fmt;
use std::io;

use iron::status::Status;
use iron::prelude::IronError;

#[derive(Debug)]
pub struct NotADir;

impl Error for NotADir {
    fn description(&self) -> &str { "Requested entry is file" }
}

impl fmt::Display for NotADir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

pub fn io_to_iron(err: io::Error) -> IronError {
    let status = match err.kind() {
        io::ErrorKind::NotFound         => Status::NotFound,
        io::ErrorKind::PermissionDenied => Status::Forbidden,
        _                               => Status::InternalServerError,
    };
    IronError::new(err, status)
}

#[derive(Debug)]
pub struct BadString {
    desc: String,
}

impl BadString {
    pub fn new(desc: &str) -> BadString {
        BadString{ desc: desc.into() }
    }
}

impl Error for BadString {
    fn description(&self) -> &str { &self.desc }
}

impl fmt::Display for BadString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}
