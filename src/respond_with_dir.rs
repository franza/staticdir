use iron::prelude::{ Response, IronResult };
use iron::mime::Mime;
use iron::status::Status;

use std::fs::{ ReadDir, DirEntry };
use std::io::Result as IoResult;

use static_dir::RespondWithDir;
use errors::io_to_iron;

pub struct AsJson;

pub struct DirEntryState {
    is_file: bool,
    is_dir: bool,
    is_symlink: bool,
    path: String,
    file_name: String,
}

impl DirEntryState {
    pub fn to_json(self) -> String {
        format!("{{\"is_file\":{},\"is_dir\":{},\"is_symlink\":{},\"path\":{:?},\"file_name\":{:?}}}", self.is_file, self.is_dir, self.is_symlink, self.path, self.file_name)
    }
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

fn dir_to_json(dir: ReadDir) -> IoResult<String> {
    let entries = dir
        .filter_map(|entry| entry.map(DirEntryState::from).ok())
        .map(|entry| entry.to_json()).collect::<Vec<_>>().join(",");
    Ok(format!("[{}]", entries))
}

impl RespondWithDir for AsJson {
    fn to_res(&self, dir: ReadDir) -> IronResult<Response> {
        let content_type = "application/json".parse::<Mime>().unwrap();
        match dir_to_json(dir) {
            Ok(json) => Ok(Response::with((Status::Ok, content_type, json))),
            Err(err) => Err(io_to_iron(err)),
        }

    }
}
