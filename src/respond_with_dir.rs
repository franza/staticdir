use iron::prelude::{ Response, IronResult, IronError };
use iron::mime::Mime;
use iron::status::Status;

use std::fs::{ ReadDir, DirEntry };
use std::io::{ Error as IoError, Result as IoResult, ErrorKind };

use static_dir::ResponseStrategy;

use rustc_serialize::json;
use errors;

use filetime::FileTime;

/// Provides list of directory contents in JSON format like
///
///```ignore
/// [
///   {
///     "file_type": "File", // "File", "Dir" or "Symlink"
///     "file_name": ".gitignore",
///     "size": 7,
///     "creation_time": null, // may be null on some Unix systems
///     "last_modification_time": 1451939290,
///     "last_access_time": 1451939309
///   },
///   {
///     "file_type": "File",
///     "file_name": "Cargo.toml",
///     "size": 196,
///     "creation_time": null,
///     "last_modification_time": 1451939547,
///     "last_access_time": 1451939547
///   },
///   {
///     "file_type": "Dir",
///     "file_name": "src",
///     "size": 4096,
///     "creation_time": null,
///     "last_modification_time": 1451939462,
///     "last_access_time": 1451939462
///   }
/// ]
///```
///
/// Current and parent directories (`.` and `..`) are not included.
pub struct AsJson;

#[derive(RustcDecodable, RustcEncodable)]
struct DirEntryState {
    file_type: FileType,
    file_name: String,
    size: u64,
    creation_time: Option<u64>,
    last_modification_time: u64,
    last_access_time: u64,
}

#[derive(RustcDecodable, RustcEncodable, PartialEq, Eq, Debug)]
enum FileType {
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

/// If failed to read metadata of a directory entry, such entry will not cause panic and will not be returned in resulting JSON
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
