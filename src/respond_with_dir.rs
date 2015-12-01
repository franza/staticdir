use iron::prelude::{ Response, IronResult };
use iron::mime::Mime;
use iron::status::Status;

use std::fs::{ ReadDir, DirEntry };
use std::io::{ Result as IoResult, Error as IoError };
use std::io;

use static_dir::RespondWithDir;

use rustc_serialize::json;
use errors;

pub struct AsJson;

#[derive(RustcDecodable, RustcEncodable)]
pub struct DirEntryState {
    is_file: bool,
    is_dir: bool,
    is_symlink: bool,
    path: String,
    file_name: String,
}

// fn bad_str_err(desc: &str) -> IoError {
//     let err = errors::BadString::new(desc);
//     io::Error::new(io::ErrorKind::Other, err)
// }

impl DirEntryState {
    fn from_entry(entry: DirEntry) -> Result<DirEntryState, IoError> {
        let bad_path = io::Error::new(io::ErrorKind::Other, errors::BadString::new("Could not read path"));
        let bad_file_name = io::Error::new(io::ErrorKind::Other, errors::BadString::new("Could not read file name"));
        // let bad_path = bad_str_err("Cound not read path");
        // let bad_file_name = bad_str_err("Could not read file name");
        let path = try!(entry.path().to_str().ok_or(bad_path)).to_string();
        let file_name = try!(entry.file_name().into_string().or(Err(bad_file_name)));
        let is_file = try!(entry.file_type()).is_file();
        let is_dir = try!(entry.file_type()).is_dir();
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

fn flatten_read_dir(dir: ReadDir) -> IoResult<Vec<DirEntryState>> {
    //filter_map will remove all erroneous entries from the collection
    let entries = dir.filter_map(|entry| entry.and_then(DirEntryState::from_entry).ok()).collect();
    Ok(entries)
}

impl RespondWithDir for AsJson {
    fn to_res(&self, dir: ReadDir) -> IronResult<Response> {
        let content_type = "application/json".parse::<Mime>().unwrap();
        flatten_read_dir(dir)
            .map_err(errors::io_to_iron)
            .and_then(|entries| json::encode(&entries).map_err(errors::json_to_iron))
            .and_then(|json| Ok(Response::with((Status::Ok, content_type, json))))
    }
}
