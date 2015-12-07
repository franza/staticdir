use iron::{ Handler, AfterMiddleware };
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

impl<T>  AfterMiddleware for StaticDir<T> where T: Send + Sync + Any + ResponseStrategy {
    fn after(&self, _req: &mut Request, res: Response) -> IronResult<Response> {
        Ok(res)
    }

    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        match err.response.status {
            //when chained with staticfile::Static NotFound may mean that it's a dir, not a file
            Some(Status::NotFound) => {
                let requested_path = unite_paths(&self.root, req);

                match metadata(&requested_path) {
                    Err(err) => Err(io_to_iron(err)),

                    Ok(ref meta) if meta.is_file() => unreachable!(),

                    Ok(ref meta) if meta.is_dir()  => {
                        let dir_entries = read_dir(&requested_path);

                        match dir_entries {
                            Err(err)    => Err(io_to_iron(err)),
                            Ok(entries) => self.behavior.make_response(entries),
                        }
                    },
                    Ok(_) => unreachable!()
                }
            },
            _ => Err(err),
        }
    }
}
