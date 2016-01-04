# staticdir
Serving directory contents for [Iron](https://github.com/iron/iron) web-framework

## Purpose

Provides the list of files and directories in a mounted folder. To respond with files use [staticfile](https://github.com/iron/staticfile) along with this one. See examples.

## Example

Start web server at `http://localhost:3000` and mount current directory. List of directory contents will be available as JSON.

```rust
extern crate staticdir;
extern crate iron;

use iron::prelude::*;
use staticdir::{ StaticDir, AsJson };

fn main() {
    Iron::new(StaticDir::new(".", AsJson)).http("localhost:3000").unwrap();
}
```

This code will return you

```JSON
[
  {
    "file_type": "File",
    "file_name": ".gitignore",
    "size": 7,
    "creation_time": null,
    "last_modification_time": 1451939290,
    "last_access_time": 1451939309
  },
  {
    "file_type": "File",
    "file_name": "Cargo.toml",
    "size": 196,
    "creation_time": null,
    "last_modification_time": 1451939547,
    "last_access_time": 1451939547
  },
  {
    "file_type": "Dir",
    "file_name": "src",
    "size": 4096,
    "creation_time": null,
    "last_modification_time": 1451939462,
    "last_access_time": 1451939462
  }
]
```

## Customize behavior

If you require some additional fields in JSON or need an HTML page, there's `ResponseStrategy` trait you can implement.

```rust
extern crate staticdir;
extern crate iron;

use iron::prelude::*;
use iron::status::Status;
use staticdir::{ StaticDir, ResponseStrategy };
use std::fs::ReadDir;
use iron::mime::Mime;

struct AsHtml;

impl ResponseStrategy for AsHtml {
    fn make_response(&self, dir: ReadDir) -> IronResult<Response> {
        let mut html = String::new();

        for entry in dir {
            let entry = entry.unwrap();
            html = format!("{}<li>{}</li>", html, entry.file_name().into_string().unwrap());
        }

        html = format!("<ul>{}</ul>", html);
        let content_type = "text/html; charset=utf-8".parse::<Mime>().unwrap();

        Ok(Response::with((Status::Ok, html, content_type)))
    }
}

fn main() {
    Iron::new(StaticDir::new(".", AsHtml)).http("localhost:3000").unwrap();
}
```

This will return an HTML page with next contents

```
* Cargo.toml
* src
* .git
```

### Working with iron components

You can use other modules of [iron core bundle](https://github.com/iron/iron#core-extensions) like [staticfile](https://github.com/iron/staticfile) and [mount](https://github.com/iron/mount).
In next example you will receive both directory listing and static files.

```rust
extern crate staticdir;
extern crate iron;
extern crate mount;
extern crate staticfile;

use iron::prelude::*;
use mount::Mount;
use staticdir::{ StaticDir, AsJson };
use staticfile::Static;


fn main() {
    let root = "tests/mount";
    let mut handle_statics = Chain::new(Static::new(root));
    handle_statics.link_after(StaticDir::new(root, AsJson));

    let mut mount = Mount::new();
    mount.mount("/static/", handle_statics);
    let mut server = Iron::new(mount).http("localhost:3000").unwrap();
}
```
