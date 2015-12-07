extern crate staticdir;
extern crate iron;
extern crate hyper;

use iron::prelude::*;

use hyper::Client;
use hyper::status;
use hyper::header::{ ContentType };
use hyper::mime::{ Mime, TopLevel, SubLevel };

use std::io::Read;
use staticdir::{ StaticDir, AsJson };

use std::ops::Deref;

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
    assert_eq!(body,  "[{\"is_file\":true,\"is_dir\":false,\"is_symlink\":false,\"path\":\"tests/mount/1.txt\",\"file_name\":\"1.txt\"},{\"is_file\":false,\"is_dir\":true,\"is_symlink\":false,\"path\":\"tests/mount/nested\",\"file_name\":\"nested\"}]");
}
