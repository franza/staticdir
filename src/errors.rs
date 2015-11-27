use std::error::Error;
use std::fmt;

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
