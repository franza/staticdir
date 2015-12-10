use iron::{ Handler, AfterMiddleware };
use iron::prelude::*;
use iron::status::Status;

use std::path::{ PathBuf, Path };
use std::fs::{ metadata, read_dir, ReadDir };
use std::any::Any;

use errors::{ NotADir, io_to_iron };

use url::percent_encoding::percent_decode;

pub trait ResponseStrategy {
    fn make_response(&self, dir: ReadDir) -> IronResult<Response>;
}

//TODO: add cache, see http://ironframework.io/doc/src/staticfile/static_handler.rs.html#30-34
pub struct StaticDir<T> {
    pub root: PathBuf,
    response_strategy: Box<T>,
}

impl<T> StaticDir<T> {
    pub fn new<P>(root: P, response_strategy: T) -> StaticDir<T> where P: AsRef<Path> {
        StaticDir{ root: root.as_ref().to_path_buf(), response_strategy: Box::new(response_strategy) }
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
            .and_then(|dir| self.response_strategy.make_response(dir))
    }
}

#[inline]
fn extend_req_path<P>(request: &Request, root_path: P) -> PathBuf where P: AsRef<Path> {
    let mut path = root_path.as_ref().to_path_buf();
    let decoded_req_path = request.url.path.iter().map(|part| String::from_utf8(percent_decode(part.as_bytes())).unwrap());
    path.extend(decoded_req_path);
    path
}

impl<T> Handler for StaticDir<T> where T: Send + Sync + Any + ResponseStrategy {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let requested_path = extend_req_path(req, &self.root);
        self.respond_from_path(&requested_path)
    }
}

impl<T>  AfterMiddleware for StaticDir<T> where T: Send + Sync + Any + ResponseStrategy {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        match res.status {
            //when chained with staticfile::Static MovedPermanently may mean that it's a dir, not a file.
            //Also in this case there's no trailing slash, but handling only first case
            Some(Status::MovedPermanently) => {
                let requested_path = extend_req_path(req, &self.root);
                self.respond_from_path(&requested_path)
            },
            _ => Ok(res),
        }
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        match err.response.status {
            //when chained with staticfile::Static NotFound may mean that it's a dir, not a file
            Some(Status::NotFound) => {
                let requested_path = extend_req_path(req, &self.root);
                self.respond_from_path(&requested_path)
            },
            _ => Err(err),
        }
    }
}
