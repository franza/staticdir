//! Serving contents of static directory.
//! # Examples
//! ```no_run
//! extern crate staticdir;
//! extern crate iron;
//!
//! use iron::prelude::*;
//! use staticdir::{ StaticDir, AsJson };
//!
//! fn main() {
//!     Iron::new(StaticDir::new(".", AsJson)).http("localhost:3000").unwrap();
//! }
//! ```
//! Because for different tasks different implementations of response may be required, this crate is designed to provide flexible behavior.
//! By default, only JSON response is supported, but different it can be customized with `ResponseStrategy` trait.
//! Here is how easily we can provide directory contents as HTML.
//! # Examples
//! ```no_run
//! extern crate staticdir;
//! extern crate iron;
//!
//! use iron::prelude::*;
//! use iron::status::Status;
//! use staticdir::{ StaticDir, ResponseStrategy };
//! use std::fs::ReadDir;
//! use iron::mime::Mime;
//!
//! struct AsHtml;
//!
//! fn build_html(dir: ReadDir) -> String {
//!     let mut html = String::new();
//!     for entry in dir {
//!         let entry = entry.unwrap();
//!         html = format!("{}<li>{}</li>", html, entry.file_name().into_string().unwrap());
//!     }
//!     format!("<ul>{}</ul>", html)
//! }
//!
//! impl ResponseStrategy for AsHtml {
//!     fn make_response(&self, dir: ReadDir) -> IronResult<Response> {
//!         let html = build_html(dir);
//!         let content_type = "text/html; charset=utf-8".parse::<Mime>().unwrap();
//!         Ok(Response::with((Status::Ok, html, content_type)))
//!     }
//! }
//!
//! fn main() {
//!     Iron::new(StaticDir::new(".", AsHtml)).http("localhost:3000").unwrap();
//! }
//! ```

extern crate iron;
extern crate rustc_serialize;
extern crate url;
extern crate filetime;

pub use self::static_dir::StaticDir;
pub use self::static_dir::ResponseStrategy;
pub use self::respond_with_dir::AsJson;

mod static_dir;
pub mod errors;
mod respond_with_dir;
