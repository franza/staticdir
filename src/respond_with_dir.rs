use iron::prelude::{ Response, IronResult };
use iron::mime::Mime;
use iron::status::Status;

use std::fs::{ ReadDir, DirEntry };
use std::io::Result as IoResult;

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

impl From<DirEntry> for DirEntryState {
    fn from(entry: DirEntry) -> DirEntryState {
        DirEntryState{
            path: entry.path().to_str().expect("Failed to convert path to string").to_string(),
            file_name: entry.file_name().into_string().expect("Failed to convert filename to string"),
            is_file: entry.file_type().unwrap().is_file(),
            is_dir: entry.file_type().unwrap().is_dir(),
            is_symlink: entry.file_type().unwrap().is_symlink(),
        }
    }
}

fn flatten_read_dir(dir: ReadDir) -> IoResult<Vec<DirEntryState>> {
    let entries = dir.filter_map(|entry| entry.map(DirEntryState::from).ok()).collect();
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
