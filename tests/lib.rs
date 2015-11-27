extern crate staticdir;
extern crate iron;
extern crate hyper;

use iron::prelude::*;

use hyper::Client;
use hyper::status;

use std::io::Read;
use staticdir::StaticDir;

#[test]
fn receive_response() {
    let mut server = Iron::new(StaticDir::new("tests/mount")).http("localhost:3000").unwrap();

    let client = Client::new();
    let mut res = client.get("http://localhost:3000").send().unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    server.close().unwrap();

    assert_eq!(res.status, status::StatusCode::Ok);
    assert_eq!(body, "static-dir");
}
