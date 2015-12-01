use std::error::Error;
use std::fmt;
use std::io;

use iron::status::Status;
use iron::prelude::IronError;
use rustc_serialize::json;

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

pub fn json_to_iron(err: json::EncoderError) -> IronError {
    IronError::new(err, Status::InternalServerError)
}

#[derive(Debug)]
pub struct BadString<'a> {
    desc: &'a str,
}

impl<'a> BadString<'a> {
    pub fn new(desc: &str) -> BadString {
        BadString{ desc: desc }
    }
}

impl<'a> Error for BadString<'a> {
    fn description(&self) -> &str { &self.desc }
}

impl<'a> fmt::Display for BadString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}
