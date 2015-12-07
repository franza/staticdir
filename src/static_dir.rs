use iron::{ Handler };
use iron::prelude::*;
use iron::status::Status;

use std::path::{ PathBuf, Path };
use std::fs::{ metadata, read_dir, ReadDir };

use errors::{ NotADir, io_to_iron };

pub trait ResponseStrategy {
    fn make_response(&self, dir: ReadDir) -> IronResult<Response>;
}

//TODO: add cache, see http://ironframework.io/doc/src/staticfile/static_handler.rs.html#30-34
pub struct StaticDir<T> {
    pub root: PathBuf,
    behavior: Box<T>,
}

impl<T> StaticDir<T> {
    pub fn new<P>(root: P, behavior: T) -> StaticDir<T> where P: AsRef<Path> {
        StaticDir{ root: root.as_ref().to_path_buf(), behavior: Box::new(behavior) }
    }
}

#[inline]
fn unite_paths<P: AsRef<Path>>(root_path: P, request: &Request) -> PathBuf {
    let mut path = root_path.as_ref().to_path_buf();
    path.extend(&request.url.path);
    path
}

use std::any::Any;

impl<T> Handler for StaticDir<T> where T: Send + Sync + Any + ResponseStrategy {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let requested_path = unite_paths(&self.root, req);
        metadata(&requested_path)
            .map_err(io_to_iron)
            .and_then(|meta| {
                match meta.is_dir() {
                    true  => read_dir(&requested_path).map_err(io_to_iron),
                    false => Err(IronError::new(NotADir, Status::BadRequest)),
                }
            })
            .and_then(|dir| self.behavior.make_response(dir))
    }
}
