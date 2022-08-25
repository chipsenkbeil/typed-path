# Typed Path

[![Crates.io][crates_img]][crates_lnk] [![Docs.rs][doc_img]][doc_lnk] [![CI][ci_img]][ci_lnk] [![RustC 1.58.1+][rustc_img]][rustc_lnk] 

[crates_img]: https://img.shields.io/crates/v/typed-path.svg
[crates_lnk]: https://crates.io/crates/typed-path
[doc_img]: https://docs.rs/typed-path/badge.svg
[doc_lnk]: https://docs.rs/typed-path
[ci_img]: https://github.com/chipsenkbeil/typed-path/actions/workflows/ci.yml/badge.svg
[ci_lnk]: https://github.com/chipsenkbeil/typed-path/actions/workflows/ci.yml
[rustc_img]: https://img.shields.io/badge/typed-path-rustc_1.58+-lightgray.svg
[rustc_lnk]: https://blog.rust-lang.org/2022/01/20/Rust-1.58.1.html

Provides typed variants of
[`Path`](https://doc.rust-lang.org/std/path/struct.Path.html) and
[`PathBuf`](https://doc.rust-lang.org/std/path/struct.PathBuf.html) for Unix
and Windows.

## Install

```toml
[dependencies]
typed-path = "0.1"
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

## License

This project is licensed under either of

Apache License, Version 2.0, (LICENSE-APACHE or
[apache-license][apache-license]) MIT license (LICENSE-MIT or
[mit-license][mit-license]) at your option.

[apache-license]: http://www.apache.org/licenses/LICENSE-2.0
[mit-license]: http://opensource.org/licenses/MIT
