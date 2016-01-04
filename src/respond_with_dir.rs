use iron::prelude::{ Response, IronResult, IronError };
use iron::mime::Mime;
use iron::status::Status;

use std::fs::{ ReadDir, DirEntry };
use std::io::{ Error as IoError, Result as IoResult, ErrorKind };

use static_dir::ResponseStrategy;

use rustc_serialize::json;
use errors;

use filetime::FileTime;

pub struct AsJson;

#[derive(RustcDecodable, RustcEncodable)]
pub struct DirEntryState {
    pub file_type: FileType,
    pub file_name: String,
    pub size: u64,
    pub creation_time: Option<u64>,
    pub last_modification_time: u64,
    pub last_access_time: u64,
}

#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug)]
pub enum FileType {
    File, Dir, Symlink
}

fn bad_str_err(desc: &str) -> IoError {
    let err = errors::BadString::new(desc);
    IoError::new(ErrorKind::Other, err)
}

fn file_name_as_string(entry: &DirEntry) -> IoResult<String> {
    entry
        .file_name()
        .into_string()
        .or(Err(bad_str_err("Could not read file name")))
}

impl DirEntryState {
    fn from_entry(entry: DirEntry) -> Result<DirEntryState, IoError> {
        let file_name =  try!(file_name_as_string(&entry));

        let file_type = match try!(entry.file_type()) {
            t if t.is_file()    => FileType::File,
            t if t.is_dir()     => FileType::Dir,
            t if t.is_symlink() => FileType::Symlink,
            _                   => unreachable!(),
        };

        let metadata = try!(entry.metadata());
        let last_modification_time = FileTime::from_last_modification_time(&metadata).seconds_relative_to_1970();
        let last_access_time = FileTime::from_last_access_time(&metadata).seconds_relative_to_1970();
        let creation_time = FileTime::from_creation_time(&metadata).map(|time| time.seconds_relative_to_1970());

        Ok(DirEntryState{
            file_name: file_name,
            file_type: file_type,
            size: metadata.len(),
            creation_time: creation_time,
            last_modification_time: last_modification_time,
            last_access_time: last_access_time,
        })
    }
}

impl ResponseStrategy for AsJson {
    fn make_response(&self, dir: ReadDir) -> IronResult<Response> {
        let entries: Vec<_> = dir
            .filter_map(|entry| entry.and_then(DirEntryState::from_entry).ok())
            .collect();

        match json::encode(&entries) {
            Ok(string) => {
                let content_type = "application/json; charset=utf-8".parse::<Mime>().unwrap();
                Ok(Response::with((Status::Ok, content_type, string)))
            },
            Err(err)   => Err(IronError::new(err, Status::InternalServerError)),
        }
    }
}
