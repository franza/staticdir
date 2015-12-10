extern crate staticdir;
extern crate iron;
extern crate hyper;
extern crate mount;
extern crate rustc_serialize;
extern crate staticfile;

use iron::prelude::*;

use mount::Mount;

use hyper::Client;
use hyper::status;
use hyper::header::{ ContentType };
use hyper::mime::{ Mime, TopLevel, SubLevel };

use std::io::Read;
use staticdir::{ StaticDir, AsJson };
use staticdir::respond_with_dir::{ DirEntryState, FileType };

use std::ops::Deref;

use rustc_serialize::json;

use staticfile::Static;

fn assert_top_dir(entries: Vec<DirEntryState>) {
    assert_eq!(entries.len(), 4);

    assert_eq!(entries[0].file_name, "по-русски");
    assert_eq!(entries[0].path, "tests/mount/по-русски");
    assert_eq!(entries[0].file_type, FileType::Dir);

    assert_eq!(entries[1].file_name, "1.txt");
    assert_eq!(entries[1].path, "tests/mount/1.txt");
    assert_eq!(entries[1].file_type, FileType::File);

    assert_eq!(entries[2].file_name, "nested");
    assert_eq!(entries[2].path, "tests/mount/nested");
    assert_eq!(entries[2].file_type, FileType::Dir);

    assert_eq!(entries[3].file_name, "has space");
    assert_eq!(entries[3].path, "tests/mount/has space");
    assert_eq!(entries[3].file_type, FileType::Dir);
}

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
    assert_top_dir(entries);
}

#[test]
fn should_work_if_dir_has_funky_chars() {
    let mut server = Iron::new(StaticDir::new("tests/mount", AsJson)).http("localhost:3001").unwrap();

    let client = Client::new();
    let mut res = client.get("http://localhost:3001/has space").send().unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    server.close().unwrap();

    assert_eq!(res.status, status::StatusCode::Ok);
    let &Mime(ref top, ref sub, _) = res.headers.get::<ContentType>().unwrap().deref();
    assert_eq!((top, sub), (&TopLevel::Application, &SubLevel::Json));

    let entries: Vec<DirEntryState> = json::decode(&body).unwrap();
    assert_eq!(entries.len(), 0);
}

#[test]
fn should_see_nested_files() {
    let mut server = Iron::new(StaticDir::new("tests/mount", AsJson)).http("localhost:3002").unwrap();

    let client = Client::new();
    let mut res = client.get("http://localhost:3002/nested").send().unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    server.close().unwrap();

    assert_eq!(res.status, status::StatusCode::Ok);
    let &Mime(ref top, ref sub, _) = res.headers.get::<ContentType>().unwrap().deref();
    assert_eq!((top, sub), (&TopLevel::Application, &SubLevel::Json));

    let entries: Vec<DirEntryState> = json::decode(&body).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_type, FileType::File);
    assert_eq!(entries[0].path, "tests/mount/nested/2.txt");
    assert_eq!(entries[0].file_name, "2.txt");
}

#[test]
fn should_work_with_mount() {
    let mut mount = Mount::new();
    mount.mount("/mnt/", StaticDir::new("tests/mount", AsJson));
    let mut server = Iron::new(mount).http("localhost:3003").unwrap();

    let client = Client::new();
    let mut res = client.get("http://localhost:3003/mnt").send().unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    server.close().unwrap();

    assert_eq!(res.status, status::StatusCode::Ok);
    let &Mime(ref top, ref sub, _) = res.headers.get::<ContentType>().unwrap().deref();
    assert_eq!((top, sub), (&TopLevel::Application, &SubLevel::Json));

    let entries: Vec<DirEntryState> = json::decode(&body).unwrap();
    assert_top_dir(entries);
}

#[test]
fn should_work_with_static_file_and_trailing_slash() {
    let handle_statics = {
        let root = "tests/mount";
        let mut chain = Chain::new(Static::new(root));
        chain.link_after(StaticDir::new(root, AsJson));
        chain
    };

    let mut mount = Mount::new();
    mount.mount("/mnt/", handle_statics);
    let mut server = Iron::new(mount).http("localhost:3004").unwrap();

    let client = Client::new();
    let mut dir_res = client.get("http://localhost:3004/mnt/").send().unwrap();
    let mut dir_entries = String::new();
    dir_res.read_to_string(&mut dir_entries).unwrap();

    let mut file_res = client.get("http://localhost:3004/mnt/1.txt").send().unwrap();
    let mut file_body = String::new();
    file_res.read_to_string(&mut file_body).unwrap();
    server.close().unwrap();

    assert_eq!(file_res.status, status::StatusCode::Ok);
    assert_eq!(file_body, "file 1.txt\n");

    assert_eq!(dir_res.status, status::StatusCode::Ok);

    let &Mime(ref top, ref sub, _) = dir_res.headers.get::<ContentType>().unwrap().deref();
    assert_eq!((top, sub), (&TopLevel::Application, &SubLevel::Json));

    let entries: Vec<DirEntryState> = json::decode(&dir_entries).unwrap();
    assert_top_dir(entries);
}

#[test]
fn should_work_with_static_file_and_no_trailing_slash() {
    let handle_statics = {
        let root = "tests/mount";
        let mut chain = Chain::new(Static::new(root));
        chain.link_after(StaticDir::new(root, AsJson));
        chain
    };

    let mut mount = Mount::new();
    mount.mount("/mnt/", handle_statics);
    let mut server = Iron::new(mount).http("localhost:3005").unwrap();

    let client = Client::new();
    let mut dir_res = client.get("http://localhost:3005/mnt").send().unwrap();
    let mut dir_entries = String::new();
    dir_res.read_to_string(&mut dir_entries).unwrap();

    let mut file_res = client.get("http://localhost:3005/mnt/1.txt").send().unwrap();
    let mut file_body = String::new();
    file_res.read_to_string(&mut file_body).unwrap();
    server.close().unwrap();

    assert_eq!(file_res.status, status::StatusCode::Ok);
    assert_eq!(file_body, "file 1.txt\n");

    assert_eq!(dir_res.status, status::StatusCode::Ok);

    let &Mime(ref top, ref sub, _) = dir_res.headers.get::<ContentType>().unwrap().deref();
    assert_eq!((top, sub), (&TopLevel::Application, &SubLevel::Json));

    let entries: Vec<DirEntryState> = json::decode(&dir_entries).unwrap();
    assert_top_dir(entries);
}
