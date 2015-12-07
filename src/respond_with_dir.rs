use iron::prelude::{ Response, IronResult, IronError };
use iron::mime::Mime;
use iron::status::Status;

use std::fs::{ ReadDir, DirEntry };
use std::io::{ Error as IoError, Result as IoResult, ErrorKind };

use static_dir::RespondWithDir;

use rustc_serialize::json;
use errors;

pub struct AsJson;

#[derive(RustcDecodable, RustcEncodable)]
pub struct DirEntryState {
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub path: String,
    pub file_name: String,
}

fn bad_str_err(desc: &str) -> IoError {
    let err = errors::BadString::new(desc);
    IoError::new(ErrorKind::Other, err)
}

fn file_name_as_string(entry: &DirEntry) -> IoResult<String> {
    entry
        .file_name()
        .into_string()
        .or_else(|_| Err(bad_str_err("Could not read file name")))
}

fn file_path_as_string(entry: &DirEntry) -> IoResult<String> {
    entry
        .path()
        .to_str()
        .ok_or_else(|| bad_str_err("Could not read path"))
        .map(|s| s.into())
}

impl DirEntryState {
    fn from_entry(entry: DirEntry) -> Result<DirEntryState, IoError> {
        let path =       try!(file_path_as_string(&entry));
        let file_name =  try!(file_name_as_string(&entry));
        let is_file =    try!(entry.file_type()).is_file();
        let is_dir =     try!(entry.file_type()).is_dir();
        let is_symlink = try!(entry.file_type()).is_symlink();

        Ok(DirEntryState{
            path: path,
            file_name: file_name,
            is_file: is_file,
            is_dir: is_dir,
            is_symlink: is_symlink,
        })
    }
}

impl RespondWithDir for AsJson {
    fn to_res(&self, dir: ReadDir) -> IronResult<Response> {
        let entries: Vec<_> = dir
            .filter_map(|entry| entry.and_then(DirEntryState::from_entry).ok())
            .collect();

        match json::encode(&entries) {
            Ok(string) => {
                let content_type = "application/json".parse::<Mime>().unwrap();
                Ok(Response::with((Status::Ok, content_type, string)))
            },
            Err(err)   => Err(IronError::new(err, Status::InternalServerError)),
        }
    }
}
