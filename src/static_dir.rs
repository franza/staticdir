use iron::{ Handler, AfterMiddleware };
use iron::prelude::*;
use iron::status::Status;

use std::path::{ PathBuf, Path };
use std::fs::{ metadata, read_dir, ReadDir };
use std::any::Any;

use errors::{ NotADir, io_to_iron };

use url::percent_encoding::percent_decode;

/// This trait is used by `StaticDir` to make a response from the collection of directory entries.
pub trait ResponseStrategy {
    /// Creates a response from the collection of directory entries.
    fn make_response(&self, dir: ReadDir) -> IronResult<Response>;
}

//TODO: add cache, see http://ironframework.io/doc/src/staticfile/static_handler.rs.html#30-34
/// An implementation of `Handler` which serves list of directory contents.
pub struct StaticDir<T> {
    ///The path from which this handler serves the list of directory contents.
    pub root: PathBuf,
    response_strategy: Box<T>,
}

impl<T> StaticDir<T> {
    /// Creates new instance of the `StaticDir` handler.
    pub fn new<P>(root: P, response_strategy: T) -> StaticDir<T> where P: Into<PathBuf> {
        StaticDir{ root: root.into(), response_strategy: Box::new(response_strategy) }
    }
}

impl<T> StaticDir<T> where T: Send + Sync + Any + ResponseStrategy {
    fn provide_dir<P>(&self, path: P) -> IronResult<Response> where P: AsRef<Path> {
        metadata(&path)
            .map_err(io_to_iron)
            .and_then(|meta| {
                match meta.is_dir() {
                    true  => read_dir(&path).map_err(io_to_iron),
                    false => Err(IronError::new(NotADir, Status::BadRequest)),
                }
            })
            .and_then(|dir| self.response_strategy.make_response(dir))
    }
}

#[inline]
fn extend_req_path<P>(request: &Request, root_path: P) -> PathBuf where P: Into<PathBuf> {
    let mut path = root_path.into();
    let decoded_req_path = request.url.path.iter().map(|part| String::from_utf8(percent_decode(part.as_bytes())).unwrap());
    path.extend(decoded_req_path);
    path
}

impl<T> Handler for StaticDir<T> where T: Send + Sync + Any + ResponseStrategy {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let requested_path = extend_req_path(req, &self.root);
        self.provide_dir(&requested_path)
    }
}


impl<T> AfterMiddleware for StaticDir<T> where T: Send + Sync + Any + ResponseStrategy {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        match res.status {
            //when chained with staticfile::Static MovedPermanently may mean that it's a dir, not a file.
            //Also in this case there's no trailing slash, but handling only first case
            Some(Status::MovedPermanently) => {
                let requested_path = extend_req_path(req, &self.root);
                self.provide_dir(&requested_path)
            },
            _ => Ok(res),
        }
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        match err.response.status {
            //when chained with staticfile::Static NotFound may mean that it's a dir, not a file
            Some(Status::NotFound) => {
                let requested_path = extend_req_path(req, &self.root);
                self.provide_dir(&requested_path)
            },
            _ => Err(err),
        }
    }
}
