use iron::{ Handler };
use iron::prelude::*;
use iron::status::Status;

use std::path::{ PathBuf, Path };
use std::fs::{ metadata, read_dir, ReadDir };
use std::io;

use errors::NotADir;

pub trait DirToResponse {
    fn to_res(&self, dir: ReadDir) -> Response;
}

//TODO: add cache, see http://ironframework.io/doc/src/staticfile/static_handler.rs.html#30-34
pub struct StaticDir<T> {
    pub root: PathBuf,
    converter: Box<T>,
}

impl<T> StaticDir<T> {
    pub fn new<P>(root: P, converter: T) -> StaticDir<T> where P: AsRef<Path> {
        StaticDir{ root: root.as_ref().to_path_buf(), converter: Box::new(converter) }
    }
}

fn unite_paths<P: AsRef<Path>>(root_path: P, request: &Request) -> PathBuf {
    let mut path = root_path.as_ref().to_path_buf();
    path.extend(&request.url.path);
    path
}

fn io_err_to_iron_err(err: io::Error) -> IronError {
    let status = match err.kind() {
        io::ErrorKind::NotFound         => Status::NotFound,
        io::ErrorKind::PermissionDenied => Status::Forbidden,
        _                               => Status::InternalServerError,
    };
    IronError::new(err, status)
}

use std::any::Any;

impl<T> Handler for StaticDir<T> where T: Send + Sync + Any + DirToResponse {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let requested_path = unite_paths(&self.root, req);
        match metadata(&requested_path) {
            Err(err) => Err(io_err_to_iron_err(err)),
            Ok(ref meta) if meta.is_dir() =>
                match read_dir(&requested_path) {
                    Err(err) => Err(io_err_to_iron_err(err)),
                    Ok(dir)  => {
                        Ok(self.converter.to_res(dir))
                    }
                },
            Ok(_) => Err(IronError::new(NotADir, Status::BadRequest)),
        }
    }
}
