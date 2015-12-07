extern crate staticdir;
extern crate iron;
extern crate hyper;
extern crate mount;
extern crate rustc_serialize;

use iron::prelude::*;

use mount::Mount;

use hyper::Client;
use hyper::status;
use hyper::header::{ ContentType };
use hyper::mime::{ Mime, TopLevel, SubLevel };

use std::io::Read;
use staticdir::{ StaticDir, AsJson, DirEntryState };

use std::ops::Deref;

use rustc_serialize::json;

#[test]
fn handler_provides_json() {
    let mut server = Iron::new(StaticDir::new("tests/mount", AsJson)).http("localhost:3000").unwrap();

    let client = Client::new();
    let mut res = client.get("http://localhost:3000").send().unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    server.close().unwrap();

    assert_eq!(res.status, status::StatusCode::Ok);
    let &Mime(ref top, ref sub, _) = res.headers.get::<ContentType>().unwrap().deref();
    assert_eq!((top, sub), (&TopLevel::Application, &SubLevel::Json));

    let entries: Vec<DirEntryState> = json::decode(&body).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].is_file, true);
    assert_eq!(entries[0].is_dir, false);
    assert_eq!(entries[0].is_symlink, false);
    assert_eq!(entries[0].path, "tests/mount/1.txt");
    assert_eq!(entries[0].file_name, "1.txt");
    assert_eq!(entries[1].is_file, false);
    assert_eq!(entries[1].is_dir, true);
    assert_eq!(entries[1].is_symlink, false);
    assert_eq!(entries[1].path, "tests/mount/nested");
    assert_eq!(entries[1].file_name, "nested");
}

#[test]
fn should_see_nested_files() {
    let mut server = Iron::new(StaticDir::new("tests/mount", AsJson)).http("localhost:3001").unwrap();

    let client = Client::new();
    let mut res = client.get("http://localhost:3001/nested").send().unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    server.close().unwrap();

    assert_eq!(res.status, status::StatusCode::Ok);
    let &Mime(ref top, ref sub, _) = res.headers.get::<ContentType>().unwrap().deref();
    assert_eq!((top, sub), (&TopLevel::Application, &SubLevel::Json));

    let entries: Vec<DirEntryState> = json::decode(&body).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].is_file, true);
    assert_eq!(entries[0].is_dir, false);
    assert_eq!(entries[0].is_symlink, false);
    assert_eq!(entries[0].path, "tests/mount/nested/2.txt");
    assert_eq!(entries[0].file_name, "2.txt");
}

#[test]
fn should_work_with_mount() {
    let mut mount = Mount::new();
    mount.mount("/mnt/", StaticDir::new("tests/mount", AsJson));
    let mut server = Iron::new(mount).http("localhost:3002").unwrap();

    let client = Client::new();
    let mut res = client.get("http://localhost:3002/mnt").send().unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    server.close().unwrap();

    assert_eq!(res.status, status::StatusCode::Ok);
    let &Mime(ref top, ref sub, _) = res.headers.get::<ContentType>().unwrap().deref();
    assert_eq!((top, sub), (&TopLevel::Application, &SubLevel::Json));
    
    let entries: Vec<DirEntryState> = json::decode(&body).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].is_file, true);
    assert_eq!(entries[0].is_dir, false);
    assert_eq!(entries[0].is_symlink, false);
    assert_eq!(entries[0].path, "tests/mount/1.txt");
    assert_eq!(entries[0].file_name, "1.txt");
    assert_eq!(entries[1].is_file, false);
    assert_eq!(entries[1].is_dir, true);
    assert_eq!(entries[1].is_symlink, false);
    assert_eq!(entries[1].path, "tests/mount/nested");
    assert_eq!(entries[1].file_name, "nested");
}
