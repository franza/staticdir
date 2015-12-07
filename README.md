# staticdir
Serving directory contents for [Iron](https://github.com/iron/iron) web-framework

## Purpose

Provides only the list of files and directories in a mounted folder. To respond with files use [staticfile](https://github.com/iron/staticfile) along with this one. See examples.

## Example

Working with [staticfile](https://github.com/iron/staticfile) and [mount](https://github.com/iron/mount)

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
Visiting `http://localhost:3000/static/` (no trailing slashes supported too) provides a neat JSON containing a list of dir contents like:

```json
[
  {
    "is_file":true,
    "is_dir":false,
    "is_symlink":false,
    "path":"tests/mount/1.txt",
    "file_name":"1.txt"
  },
  {
    "is_file":false,
    "is_dir":true,
    "is_symlink":false,
    "path":"tests/mount/nested",
    "file_name":"nested"
    }
]
```
It can also be used without other libs, just with good ol' Iron:

```rust
fn handler_provides_json() {
    let mut server = Iron::new(StaticDir::new("tests/mount", AsJson)).http("localhost:3000").unwrap();
}
```

The library was designed to use with [staticfile](https://github.com/iron/staticfile) so only `iron::middleware::AfterMiddleware` and `iron::middleware::Handler` were implemented.

## Customization

This line

```rust
StaticDir::new(root, AsJson)
```

says that the contents will be delivered as JSON. Currently, `AsJson` is the only  supported response strategy but you can implement `ResponseStrategy` trait to provide something different in an ordinary Iron response:

```rust
struct Banana;

impl ResponseStrategy for Banana {
    fn make_response(&self, _dir: ReadDir) -> IronResult<Response> {
        Ok(Response::with((Status::Ok, "good ol' banana")))
    }
}

fn main() {
    let root = "tests/mount";
    let mut handle_statics = Chain::new(Static::new(root));
    handle_statics.link_after(StaticDir::new(root, Banana));

    let mut mount = Mount::new();
    mount.mount("/static/", handle_statics);
    let mut server = Iron::new(mount).http("localhost:3000").unwrap();
}
```
