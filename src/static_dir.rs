use iron::{ Handler };
use iron::prelude::*;
use iron::status::Status;

use std::path::{ PathBuf, Path };
use std::fs::{ metadata, read_dir };
use std::io;

use errors::NotADir;

//TODO: add cache, see http://ironframework.io/doc/src/staticfile/static_handler.rs.html#30-34
pub struct StaticDir {
    pub root: PathBuf
}

impl StaticDir {
    pub fn new<P: AsRef<Path>>(root: P) -> StaticDir {
        StaticDir{ root: root.as_ref().to_path_buf() }
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

impl Handler for StaticDir {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let requested_path = unite_paths(&self.root, req);
        match metadata(&requested_path) {
            Err(err) => Err(io_err_to_iron_err(err)),
            Ok(ref meta) if meta.is_dir() => Ok(Response::with((Status::Ok, "static-dir"))),
            Ok(_) => Err(IronError::new(NotADir, Status::BadRequest)),
        }
    }
}
