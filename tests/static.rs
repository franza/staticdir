extern crate staticdir;
extern crate iron;
extern crate mount;
extern crate rustc_serialize;
extern crate staticfile;
extern crate iron_test;

use iron::prelude::*;
use iron::headers::{ Headers, ContentType };
use iron::mime::{ Mime, TopLevel, SubLevel };
use iron::status;
use iron_test::{ request, ProjectBuilder, response };
use mount::Mount;
use rustc_serialize::json;
use std::ops::Deref;
use std::fs::ReadDir;
use staticdir::*;
use staticfile::Static;

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

#[test]
fn response_should_be_200() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();

    let static_dir = StaticDir::new(p.root(), AsJson);
    let res = request::get("http://localhost:3000/", Headers::new(), &static_dir).unwrap();
    assert_eq!(res.status.unwrap(), status::Ok);
}

#[test]
fn response_should_be_json() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();

    let static_dir = StaticDir::new(p.root(), AsJson);
    let res = request::get("http://localhost:3000/", Headers::new(), &static_dir).unwrap();
    let &Mime(ref top, ref sub, _) = res.headers.get::<ContentType>().unwrap().deref();
    assert_eq!((top, sub), (&TopLevel::Application, &SubLevel::Json));
}

#[test]
fn response_should_contain_file_entry() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();

    let static_dir = StaticDir::new(p.root(), AsJson);
    let res = request::get("http://localhost:3000/", Headers::new(), &static_dir).unwrap();
    let body = response::extract_body_to_string(res);
    let entries: Vec<DirEntryState> = json::decode(&body).unwrap();
    let ref entry = entries[0];
    assert_eq!(entry.file_name, "file1.html");
    assert_eq!(entry.file_type, FileType::File);
    assert!(entry.size > 0);
    assert!(entry.last_modification_time > 0);
    assert!(entry.last_access_time > 0);
}

#[test]
fn should_work_with_mount() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();

    let static_dir = StaticDir::new(p.root(), AsJson);
    let mut mount = Mount::new();
    mount.mount("/mnt/", static_dir);
    let res = request::get("http://localhost:3000/mnt", Headers::new(), &mount).unwrap();
    assert_eq!(res.status.unwrap(), status::Ok);
}

#[test]
fn should_support_custom_strategies() {
    struct AsHtml;

    fn build_html(dir: ReadDir) -> String {
        let mut html = String::new();
        for entry in dir {
            let entry = entry.unwrap();
            html = format!("{}<li>{}</li>", html, entry.file_name().into_string().unwrap());
        }
        format!("<ul>{}</ul>", html)
    }

    impl ResponseStrategy for AsHtml {
        fn make_response(&self, dir: ReadDir) -> IronResult<Response> {
            let html = build_html(dir);
            let content_type = "text/html; charset=utf-8".parse::<Mime>().unwrap();
            Ok(Response::with((status::Ok, html, content_type)))
        }
    }

    let p = ProjectBuilder::new("example")
        .file("file1.html", "this is file1")
        .file("file2.html", "this is file2");
    p.build();

    let static_dir = StaticDir::new(p.root(), AsHtml);
    let res = request::get("http://localhost:3000/", Headers::new(), &static_dir).unwrap();
    let body = response::extract_body_to_string(res);
    assert_eq!(body, "<ul><li>file2.html</li><li>file1.html</li></ul>");
}

#[test]
fn should_work_with_trailing_slash() {
    let p = ProjectBuilder::new("example").file("dir/file1.html", "this is file1");
    p.build();

    let static_dir = StaticDir::new(p.root(), AsJson);
    let res = request::get("http://localhost:3000/dir/", Headers::new(), &static_dir).unwrap();
    assert_eq!(res.status.unwrap(), status::Ok);
}

#[test]
fn should_work_without_trailing_slash() {
    let p = ProjectBuilder::new("example").file("dir/file1.html", "this is file1");
    p.build();

    let static_dir = StaticDir::new(p.root(), AsJson);
    let res = request::get("http://localhost:3000/dir", Headers::new(), &static_dir).unwrap();
    assert_eq!(res.status.unwrap(), status::Ok);
}

#[test]
fn should_work_with_static_file() {
    let p = ProjectBuilder::new("example")
        .file("dir/file1.html", "this is file");
    p.build();

    let handle_statics = {
        let root = p.root();
        let mut chain = Chain::new(Static::new(root));
        chain.link_after(StaticDir::new(root, AsJson));
        chain
    };

    let res = request::get("http://localhost:3000/dir/file1.html", Headers::new(), &handle_statics).unwrap();
    assert_eq!(res.status.unwrap(), status::Ok);
    assert_eq!(response::extract_body_to_string(res), "this is file");

    let res = request::get("http://localhost:3000/dir/", Headers::new(), &handle_statics).unwrap();
    assert_eq!(res.status.unwrap(), status::Ok);
    let &Mime(ref top, ref sub, _) = res.headers.get::<ContentType>().unwrap().deref();
    assert_eq!((top, sub), (&TopLevel::Application, &SubLevel::Json));
}

#[test]
fn should_return_404_if_referred_to_non_existent_dir_without_trailing_slash() {
    let p = ProjectBuilder::new("example").file("file1", "this is file1");
    p.build();

    let static_dir = StaticDir::new(p.root(), AsJson);
    let err = request::get("http://localhost:3000/file1", Headers::new(), &static_dir).unwrap_err();
    assert_eq!(err.response.status.unwrap(), status::NotFound);
}

#[test]
fn should_return_404_if_referred_to_non_existent_dir_with_trailing_slash() {
    let p = ProjectBuilder::new("example").file("file1", "this is file1");
    p.build();

    let static_dir = StaticDir::new(p.root(), AsJson);
    let err = request::get("http://localhost:3000/file1/", Headers::new(), &static_dir).unwrap_err();
    assert_eq!(err.response.status.unwrap(), status::NotFound);
}

#[test]
fn should_work_with_funky_chars() {
    let p = ProjectBuilder::new("example").file("dir with spaces/some file.html", "this is file1");
    p.build();

    let static_dir = StaticDir::new(p.root(), AsJson);
    let res = request::get("http://localhost:3000/dir with spaces/", Headers::new(), &static_dir).unwrap();
    assert_eq!(res.status.unwrap(), status::Ok);

    let body = response::extract_body_to_string(res);
    let entries: Vec<DirEntryState> = json::decode(&body).unwrap();
    assert_eq!(entries[0].file_name, "some file.html");
}
