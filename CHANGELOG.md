# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.0] - 2024-02-24

* Add `push_checked` function, which ensures that any path added to an existing `PathBuf` or `TypedPathBuf` must abide by the following rules:
    1. It cannot be an absolute path. Only relative paths allowed.
    2. In the case of Windows, it cannot start with a prefix like `C:`.
    3. All normal components of the path must contain only valid characters.
    4. If parent directory (..) components are present, they must not result in a path traversal attack (impacting the current path).
* Add `join_checked` function, which ensures that any path joied with an existing path follows the rules of `push_checked`
* Add `with_encoding_checked` function to ensure that the resulting path from an encoding conversion is still valid
* Add `with_unix_encoding_checked` and `with_windows_encoding_checked` functions as shortcuts to `with_encoding_checked`
* Add `is_valid` to `Component` and `Utf8Component` traits alongside `Path` and `Utf8Path` to indicate if a component/path is valid for the given encoding

## [0.7.1] - 2024-02-15

* Support `wasm` family for compilation

## [0.7.0] - 2023-11-04

* Support `no_std` environments, when `default-features = false` is set for the crate

## [0.6.0] - 2023-10-12

* Refactor crate exports such that everything other than constants are now
  top-level exports
  * `typed_path::unix::UnixComponent` is now `typed_path::UnixComponent`
  * `typed_path::unix::Utf8UnixComponent` is now `typed_path::Utf8UnixComponent`
  * `typed_path::windows::WindowsComponent` is now `typed_path::WindowsComponent`
  * `typed_path::windows::Utf8WindowsComponent` is now `typed_path::Utf8WindowsComponent`
  * `typed_path::windows::WindowsPrefix` is now `typed_path::WindowsPrefix`
  * `typed_path::windows::Utf8WindowsPrefix` is now `typed_path::Utf8WindowsPrefix`
* Constants are now located within the `constants` module, broken out by `unix`
  and `windows` modules to house each set of constants
* `TypedPath` and `Utf8TypedPath` now match the method signature of `Path` for
  constructing self with `::new(...)`
* Majority of methods available for `Path` and `PathBuf` have been ported over
  to `TypedPath` and `TypedPathBuf`
* Implement `std::fmt::Display` for `Utf8UnixComponent`,
  `Utf8WindowsComponent`, and `Utf8TypedComponent`

## [0.5.0] - 2023-09-28

* Add `TypedPath`, `Utf8TypedPath`, `TypedPathBuf`, and `Utf8TypedPathBuf`
  enums to support code that can operate on both Windows and Unix paths

## [0.4.2] - 2023-09-19

* Add `From<&Utf8NativePath>` for `std::path::PathBuf`

## [0.4.1] - 2023-09-17

* Add `AsRef<std::path::Path>` for `Utf8NativePath` and `Utf8NativePathBuf`
* Add `From<Utf8NativePathBuf>` for `std::path::PathBuf`
* Add `rustfmt.toml` to dictate formatting using `cargo +nightly fmt --all`

## [0.4.0] - 2023-08-23

* Add `normalize` method to `Path` and `Utf8Path` to support resolving `.` and
  `..` across Windows and Unix encodings
* Add `is_parent` and `is_current` methods to `Component` and `Utf8Component`
  traits with implementations for both Unix and Windows component
  implementations
* Add `root`, `parent`, and `current` static methods to `Component` and
  `Utf8Component` traits to support creating the instances from generics
* Add `absolutize` to both `normalize` the path and - in the case of relative
  paths - prefix the path with the current working directory
* Add `with_encoding` to `Path` and `Utf8Path` support converting between the
  Unix and Windows encoding types
* Add `utils::current_dir` and `utils::utf8_current_dir` to retrieve the
  current working directory as either a `PathBuf` or `Utf8PathBuf`
* Add `with_unix_encoding` and `with_windows_encoding` to `Path` and `Utf8Path`
  support converting between the Unix and Windows encoding types
* Add `has_unix_encoding` and `has_windows_encoding` to `Path` and `Utf8Path`
  to detect explicit encodings

## [0.3.2] - 2023-03-27

* Fix implementation of `Display` for `Utf8Path` to use underlying str
  `Display` instead of `Debug`

## [0.3.1] - 2023-03-14

* Fix joining of empty path with relative path resulting in absolute path when
  using `UnixPath::join` or `Utf8UnixPath::join` (#6)

## [0.3.0] - 2023-02-14

* Add `Clone` implementation for `Box<Path<T>>` and `Box<Utf8Path<T>>`
* Fix `Clone` implementation for `PathBuf<T>` and `Utf8PathBuf<T>` requiring a
  clone implementation for the encoding, which is not necessary
  ([#5](https://github.com/chipsenkbeil/typed-path/issues/5))
* Update `Debug` implementation for `Path<T>`, `Utf8Path<T>`, `PathBuf<T>`, and
  `Utf8PathBuf<T>` to no longer require debug implementation for encoding,
  which is not necessary
* Add `label` method to encoding implementations, used for debugging purposes

## [0.2.1] - 2022-12-12

* Update README with more UTF8 examples and add proper testing of README
  examples via doctest

## [0.2.0] - 2022-12-12

* Added UTF-8 variants of `Path`, `PathBuf`, `Components`, `Component`, and
  other data structures to support `str` versus `[u8]`
* Remove requirements of `Clone`, `Debug`, `Display`, and `Sized` on
  `Encoding` and subsequent implementations `UnixEncoding` and
  `WindowsEncoding`

## [0.1.0] - 2022-08-24

Initial release of the library!

[Unreleased]: https://github.com/chipsenkbeil/typed-path/compare/v0.7.1...HEAD
[0.7.1]: https://github.com/chipsenkbeil/typed-path/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/chipsenkbeil/typed-path/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/chipsenkbeil/typed-path/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/chipsenkbeil/typed-path/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/chipsenkbeil/typed-path/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/chipsenkbeil/typed-path/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/chipsenkbeil/typed-path/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/chipsenkbeil/typed-path/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/chipsenkbeil/typed-path/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/chipsenkbeil/typed-path/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/chipsenkbeil/typed-path/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/chipsenkbeil/typed-path/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/chipsenkbeil/typed-path/releases/tag/v0.1.0
