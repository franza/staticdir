use iron::{ Handler, AfterMiddleware };
use iron::prelude::*;
use iron::status::Status;

use std::path::{ PathBuf, Path };
use std::fs::{ metadata, read_dir, ReadDir };
use std::any::Any;

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

impl<T> StaticDir<T> where T: Send + Sync + Any + ResponseStrategy {
    fn respond_from_path<P>(&self, path: P) -> IronResult<Response> where P: AsRef<Path> {
        metadata(&path)
            .map_err(io_to_iron)
            .and_then(|meta| {
                match meta.is_dir() {
                    true  => read_dir(&path).map_err(io_to_iron),
                    false => Err(IronError::new(NotADir, Status::BadRequest)),
                }
            })
            .and_then(|dir| self.behavior.make_response(dir))
    }
}

#[inline]
fn unite_paths<P>(root_path: P, request: &Request) -> PathBuf where P: AsRef<Path> {
    let mut path = root_path.as_ref().to_path_buf();
    path.extend(&request.url.path);
    path
}

impl<T> Handler for StaticDir<T> where T: Send + Sync + Any + ResponseStrategy {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let requested_path = unite_paths(&self.root, req);
        self.respond_from_path(&requested_path)
    }
}

impl<T>  AfterMiddleware for StaticDir<T> where T: Send + Sync + Any + ResponseStrategy {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        match res.status {
            //when chained with staticfile::Static MovedPermanently may mean that it's a dir, not a file.
            //Also in this case there's no trailing slash, but handling only first case
            Some(Status::MovedPermanently) => {
                let requested_path = unite_paths(&self.root, req);
                self.respond_from_path(&requested_path)
            },
            _ => Ok(res),
        }
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        match err.response.status {
            //when chained with staticfile::Static NotFound may mean that it's a dir, not a file
            Some(Status::NotFound) => {
                let requested_path = unite_paths(&self.root, req);
                self.respond_from_path(&requested_path)
            },
            _ => Err(err),
        }
    }
}
