# Typed Path

[![Crates.io][crates_img]][crates_lnk] [![Docs.rs][doc_img]][doc_lnk] [![CI][ci_img]][ci_lnk] [![RustC 1.58.1+][rustc_img]][rustc_lnk] 

[crates_img]: https://img.shields.io/crates/v/typed-path.svg
[crates_lnk]: https://crates.io/crates/typed-path
[doc_img]: https://docs.rs/typed-path/badge.svg
[doc_lnk]: https://docs.rs/typed-path
[ci_img]: https://github.com/chipsenkbeil/typed-path/actions/workflows/ci.yml/badge.svg
[ci_lnk]: https://github.com/chipsenkbeil/typed-path/actions/workflows/ci.yml
[rustc_img]: https://img.shields.io/badge/rustc_1.58.1+-lightgray.svg
[rustc_lnk]: https://blog.rust-lang.org/2022/01/20/Rust-1.58.1.html

Provides typed variants of
[`Path`](https://doc.rust-lang.org/std/path/struct.Path.html) and
[`PathBuf`](https://doc.rust-lang.org/std/path/struct.PathBuf.html) for Unix
and Windows.

## Install

```toml
[dependencies]
typed-path = "0.5"
```

## Why?

> Some applications need to manipulate Windows or UNIX paths on different
> platforms, for a variety of reasons: constructing portable file formats,
> parsing files from other platforms, handling archive formats, working with
> certain network protocols, and so on.
>
> -- Josh Triplett

[Check out this issue](https://github.com/rust-lang/rust/issues/66621) of a
discussion for this. The functionality actually exists within the standard
library, but is not exposed!

This means that parsing a path like `C:\path\to\file.txt` will be parsed
differently by `std::path::Path` depending on which platform you are on!

```rust
use std::path::Path;

fn main() {
    // On Windows, this prints out:
    //
    // * Prefix(PrefixComponent { raw: "C:", parsed: Disk(67) })
    // * RootDir
    // * Normal("path")
    // * Normal("to")
    // * Normal("file.txt")]
    //
    // But on Unix, this prints out:
    //
    // * Normal("C:\\path\\to\\file.txt")
    println!(
        "{:?}",
        Path::new(r"C:\path\to\file.txt")
            .components()
            .collect::<Vec<_>>()
    );
}
```

## Usage

### Byte paths

The library provides a generic `Path<T>` and `PathBuf<T>` that use `[u8]` and
`Vec<u8>` underneath instead of `OsStr` and `OsString`. An encoding generic
type is provided to dictate how the underlying bytes are parsed in order to
support consistent path functionality no matter what operating system you are
compiling against!

```rust
use typed_path::WindowsPath;

fn main() {
    // On all platforms, this prints out:
    //
    // * Prefix(PrefixComponent { raw: "C:", parsed: Disk(67) })
    // * RootDir
    // * Normal("path")
    // * Normal("to")
    // * Normal("file.txt")]
    //
    println!(
        "{:?}",
        WindowsPath::new(r"C:\path\to\file.txt")
            .components()
            .collect::<Vec<_>>()
    );
}
```

### UTF8-enforced paths

Alongside the byte paths, this library also supports UTF8-enforced paths
through `UTF8Path<T>` and `UTF8PathBuf<T>`, which internally use `str` and
`String`. An encoding generic type is provided to dictate how the underlying
characters are parsed in order to support consistent path functionality no
matter what operating system you are
compiling against!

```rust
use typed_path::Utf8WindowsPath;

fn main() {
    // On all platforms, this prints out:
    //
    // * Prefix(Utf8WindowsPrefixComponent { raw: "C:", parsed: Disk(67) })
    // * RootDir
    // * Normal("path")
    // * Normal("to")
    // * Normal("file.txt")]
    //
    println!(
        "{:?}",
        Utf8WindowsPath::new(r"C:\path\to\file.txt")
            .components()
            .collect::<Vec<_>>()
    );
}
```

### Converting between encodings

There may be times in which you need to convert between encodings such as when
you want to load a native path and convert it into another format. In that
case, you can use the `with_encoding` method to convert a `Path` or `Utf8Path`
into their respective `PathBuf` and `Utf8PathBuf` with an explicit encoding:

```rust
use typed_path::{Utf8Path, Utf8UnixEncoding, Utf8WindowsEncoding};

fn main() {
    // Convert from Unix to Windows
    let unix_path = Utf8Path::<Utf8UnixEncoding>::new("/tmp/foo.txt");
    let windows_path = unix_path.with_encoding::<Utf8WindowsEncoding>();
    assert_eq!(windows_path, Utf8Path::<Utf8WindowsEncoding>::new(r"\tmp\foo.txt"));
   
    // Converting from Windows to Unix will drop any prefix
    let windows_path = Utf8Path::<Utf8WindowsEncoding>::new(r"C:\tmp\foo.txt");
    let unix_path = windows_path.with_encoding::<Utf8UnixEncoding>();
    assert_eq!(unix_path, Utf8Path::<Utf8UnixEncoding>::new(r"/tmp/foo.txt"));
   
    // Converting to itself should retain everything
    let path = Utf8Path::<Utf8WindowsEncoding>::new(r"C:\tmp\foo.txt");
    assert_eq!(
        path.with_encoding::<Utf8WindowsEncoding>(),
        Utf8Path::<Utf8WindowsEncoding>::new(r"C:\tmp\foo.txt"),
    );
}
```

### Normalization

Alongside implementing the standard methods associated with
[`Path`](https://doc.rust-lang.org/std/path/struct.Path.html) and
[`PathBuf`](https://doc.rust-lang.org/std/path/struct.PathBuf.html) from the
standard library, this crate also implements several additional methods
including the ability to normalize a path by resolving `.` and `..` without the
need to have the path exist.


```rust
use typed_path::Utf8UnixPath;

assert_eq!(
    Utf8UnixPath::new("foo/bar//baz/./asdf/quux/..").normalize(),
    Utf8UnixPath::new("foo/bar/baz/asdf"),
);
```

In addition, you can leverage `absolutize` to convert a path to an absolute
form by prepending the current working directory if the path is relative and
then normalizing it:

```rust
use typed_path::{utils, Utf8UnixPath};

// With an absolute path, it is just normalized
let path = Utf8UnixPath::new("/a/b/../c/./d");
assert_eq!(path.absolutize().unwrap(), Utf8UnixPath::new("/a/c/d"));

// With a relative path, it is first joined with the current working directory
// and then normalized
let cwd = utils::utf8_current_dir().unwrap().with_unix_encoding();
let path = cwd.join(Utf8UnixPath::new("a/b/../c/./d"));
assert_eq!(path.absolutize().unwrap(), cwd.join(Utf8UnixPath::new("a/c/d")));
```

### Current directory

Helper functions are available in the `utils` module, and one of those provides
an identical experience to
[`std::env::current_dir`](https://doc.rust-lang.org/std/env/fn.current_dir.html):

```rust
// Retrieves the current directory as a NativePath:
//
// * For Unix family, this would be Path<UnixEncoding>
// * For Windows family, this would be Path<WindowsEncoding>
let _cwd = typed_path::utils::current_dir().unwrap();

// Retrieves the current directory as a Utf8NativePath:
//
// * For Unix family, this would be Utf8Path<Utf8UnixEncoding>
// * For Windows family, this would be Utf8Path<Utf8WindowsEncoding>
let _utf8_cwd = typed_path::utils::utf8_current_dir().unwrap();
```

## License

This project is licensed under either of

Apache License, Version 2.0, (LICENSE-APACHE or
[apache-license][apache-license]) MIT license (LICENSE-MIT or
[mit-license][mit-license]) at your option.

[apache-license]: http://www.apache.org/licenses/LICENSE-2.0
[mit-license]: http://opensource.org/licenses/MIT
