//! Serving contents of static directory.
//!
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
//!
//! This will provide JSON similar to this:
//!
//!```ignore
//! [
//!   {
//!     "file_type": "File", // "File", "Dir" or "Symlink"
//!     "file_name": ".gitignore",
//!     "size": 7,
//!     "creation_time": null, // may be null on some Unix systems
//!     "last_modification_time": 1451939290,
//!     "last_access_time": 1451939309
//!   },
//!   {
//!     "file_type": "File",
//!     "file_name": "Cargo.toml",
//!     "size": 196,
//!     "creation_time": null,
//!     "last_modification_time": 1451939547,
//!     "last_access_time": 1451939547
//!   },
//!   {
//!     "file_type": "Dir",
//!     "file_name": "src",
//!     "size": 4096,
//!     "creation_time": null,
//!     "last_modification_time": 1451939462,
//!     "last_access_time": 1451939462
//!   }
//! ]
//!```
//!
//! Because for different tasks different implementations of response may be required, this crate is designed to provide flexible behavior.
//! By default, only JSON response is supported, but different it can be customized with `ResponseStrategy` trait.
//! Here is how easily we can provide directory contents as HTML.
//!
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
//!
//! `StaticDir` implements `Handler` and `AfterMiddleware` so can be combined with other plugins of `iron` framework like `staticfile` or `mount`:
//!
//! ```no_run
//! extern crate staticdir;
//! extern crate iron;
//! extern crate mount;
//! extern crate staticfile;
//!
//! use iron::prelude::*;
//! use mount::Mount;
//! use staticdir::{ StaticDir, AsJson };
//! use staticfile::Static;
//!
//!
//! fn main() {
//!     let root = "tests/mount";
//!     let mut handle_statics = Chain::new(Static::new(root));
//!     handle_statics.link_after(StaticDir::new(root, AsJson));
//!
//!     let mut mount = Mount::new();
//!     mount.mount("/static/", handle_statics);
//!     let mut server = Iron::new(mount).http("localhost:3000").unwrap();
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
