# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
* Add `to_encoding` to `Path` and `Utf8Path` support converting between the
  Unix and Windows encoding types

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

[Unreleased]: https://github.com/chipsenkbeil/typed-path/compare/v0.4.0...HEAD
[0.3.2]: https://github.com/chipsenkbeil/typed-path/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/chipsenkbeil/typed-path/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/chipsenkbeil/typed-path/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/chipsenkbeil/typed-path/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/chipsenkbeil/typed-path/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/chipsenkbeil/typed-path/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/chipsenkbeil/typed-path/releases/tag/v0.1.0
