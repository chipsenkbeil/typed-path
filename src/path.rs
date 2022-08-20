use crate::{Prefix, UnixComponent, WindowsComponent};
use paste::paste;
use std::{
    borrow::{Cow, ToOwned},
    cmp, fmt,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct StripPrefixError;

impl fmt::Display for StripPrefixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "prefix not found")
    }
}

impl std::error::Error for StripPrefixError {}

fn rsplit_file_at_dot(file: &str) -> (Option<&str>, Option<&str>) {
    if file == ".." {
        return (Some(file), None);
    }

    let mut iter = file.rsplitn(2, |c: char| c == '.');
    let after = iter.next();
    let before = iter.next();
    if before == Some("") {
        (Some(file), None)
    } else {
        (before, after)
    }
}

macro_rules! impl_path {
    ($platform:ident $sep:literal $prefix:literal) => {paste!{
        #[doc = "UTF-8 version of [`std::path::Path`] for " [<$platform:camel>]]
        #[repr(transparent)]
        pub struct [<$platform:camel Path>] {
            /// Path as an unparsed `str` slice
            inner: str,
        }

        impl [<$platform:camel Path>] {
            #[doc = "Directly wraps a string slice as a [`" [<$platform:camel Path>] "`] slice."]
            #[doc = ""]
            #[doc = "This is a cost-free conversion."]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = ""]
            #[doc = "```"]
            #[doc = "use typed_path::" [<$platform:camel Path>] ";"]
            #[doc = ""]
            #[doc = [<$platform:camel Path>] "::new(\"foo.txt\");"]
            #[doc = "```"]
            #[doc = ""]
            #[doc = "You can create [`" [<$platform:camel Path>] "`]s from [`String`]s, or even other [`" [<$platform:camel Path>] "`]s:"]
            #[doc = ""]
            #[doc = "```"]
            #[doc = "use typed_path::" [<$platform:camel Path>] ";"]
            #[doc = ""]
            #[doc = "let string = String::from(\"foo.txt\");"]
            #[doc = "let from_string = " [<$platform:camel Path>] "::new(&string);"]
            #[doc = "let from_path = " [<$platform:camel Path>] "::new(&from_string);"]
            #[doc = "assert_eq!(from_string, from_path);"]
            #[doc = "```"]
            #[inline]
            pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &[<$platform:camel Path>] {
                unsafe {
                    &*(s.as_ref() as *const str as *const [<$platform:camel Path>])
                }
            }

            #[doc = "Yields the underlying [`str`] slice"]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = ""]
            #[doc = "```"]
            #[doc = "use typed_path::" [<$platform:camel Path>] ";"]
            #[doc = ""]
            #[doc = "let s = " [<$platform:camel Path>] "::new(\"foo.txt\").as_str();"]
            #[doc = "assert_eq!(s, \"foo.txt\");"]
            #[doc = "```"]
            pub fn as_str(&self) -> &str {
                &self.inner
            }

            #[doc = "Converts a [`" [<$platform:camel Path>] "`] to an owned [`" [<$platform:camel PathBuf>] "`]."]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = ""]
            #[doc = "```"]
            #[doc = "use typed_path::" [<$platform:camel Path>] ";"]
            #[doc = ""]
            #[doc = "let path_buf = " [<$platform:camel Path>] "::new(\"foo.txt\").to_path_buf();"]
            #[doc = "assert_eq!(path_buf, typed_path::" [<$platform:camel PathBuf>] "::from(\"foo.txt\"));"]
            #[doc = "```"]
            pub fn to_path_buf(&self) -> $crate::[<$platform:camel PathBuf>] {
                $crate::[<$platform:camel PathBuf>] {
                    inner: self.inner.to_owned(),
                }
            }

            #[doc = "Returns `true` if the [`" [<$platform:camel Path>] "`] is relative, i.e., not absolute."]
            #[doc = ""]
            #[doc = "See [`is_absolute`]'s documentation for more details."]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = ""]
            #[doc = "```"]
            #[doc = "use typed_path::" [<$platform:camel Path>] ";"]
            #[doc = ""]
            #[doc = "assert!(" [<$platform:camel Path>] "::new(\"foo.txt\").is_relative());"]
            #[doc = "```"]
            #[doc = ""]
            #[doc = "[`is_absolute`]: " [<$platform:camel Path>] "::is_absolute"]
            #[inline]
            pub fn is_relative(&self) -> bool {
                !self.is_absolute()
            }

            #[doc = " Returns the [`" [<$platform:camel Path>] "`] without its final component, if there is one."]
            #[doc = ""]
            #[doc = " Returns [`None`] if the path terminates in a root or prefix."]
            #[doc = ""]
            #[doc = " # Examples"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::" [<$platform:camel Path>] ";"]
            #[doc = ""]
            #[doc = " let path = " [<$platform:camel Path>] "::new(\"" $sep "foo" $sep "bar\");"]
            #[doc = " let parent = path.parent().unwrap();"]
            #[doc = " assert_eq!(parent, " [<$platform:camel Path>] "::new(\"" $sep "foo\"));"]
            #[doc = ""]
            #[doc = " let grand_parent = parent.parent().unwrap();"]
            #[doc = " assert_eq!(grand_parent, " [<$platform:camel Path>] "::new(\"" $sep "\"));"]
            #[doc = " assert_eq!(grand_parent.parent(), None);"]
            #[doc = " ```"]
            pub fn parent(&self) -> Option<&[<$platform:camel Path>]> {
                let mut comps = self.components();
                let comp = comps.next_back();
                comp.and_then(|p| match p {
                    $crate::[<$platform:camel Component>]::Normal(_)
                    | $crate::[<$platform:camel Component>]::CurDir
                    | $crate::[<$platform:camel Component>]::ParentDir => {
                        Some(comps.as_path())
                    }
                    _ => None,
                })
            }

            #[doc = " Produces an iterator over [`" [<$platform:camel Path>] "`] and its ancestors."]
            #[doc = ""]
            #[doc = " The iterator will yield the [`" [<$platform:camel Path>] "`] that is returned if the [`parent`] method is used zero"]
            #[doc = " or more times. That means, the iterator will yield `&self`, `&self.parent().unwrap()`,"]
            #[doc = " `&self.parent().unwrap().parent().unwrap()` and so on. If the [`parent`] method returns"]
            #[doc = " [`None`], the iterator will do likewise. The iterator will always yield at least one value,"]
            #[doc = " namely `&self`."]
            #[doc = ""]
            #[doc = " # Examples"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::" [<$platform:camel Path>] ";"]
            #[doc = ""]
            #[doc = " let mut ancestors = " [<$platform:camel Path>] "::new(\"" $sep "foo" $sep "bar\").ancestors();"]
            #[doc = " assert_eq!(ancestors.next(), Some(" [<$platform:camel Path>] "::new(\"" $sep "foo" $sep "bar\")));"]
            #[doc = " assert_eq!(ancestors.next(), Some(" [<$platform:camel Path>] "::new(\"" $sep "foo\")));"]
            #[doc = " assert_eq!(ancestors.next(), Some(" [<$platform:camel Path>] "::new(\"" $sep "\")));"]
            #[doc = " assert_eq!(ancestors.next(), None);"]
            #[doc = ""]
            #[doc = " let mut ancestors = " [<$platform:camel Path>] "::new(\".." $sep "foo" $sep "bar\").ancestors();"]
            #[doc = " assert_eq!(ancestors.next(), Some(" [<$platform:camel Path>] "::new(\".." $sep "foo" $sep "bar\")));"]
            #[doc = " assert_eq!(ancestors.next(), Some(" [<$platform:camel Path>] "::new(\".." $sep "foo\")));"]
            #[doc = " assert_eq!(ancestors.next(), Some(" [<$platform:camel Path>] "::new(\"..\")));"]
            #[doc = " assert_eq!(ancestors.next(), Some(" [<$platform:camel Path>] "::new(\"\")));"]
            #[doc = " assert_eq!(ancestors.next(), None);"]
            #[doc = " ```"]
            #[doc = ""]
            #[doc = " [`parent`]: " [<$platform:camel Path>] "::parent"]
            #[inline]
            pub fn ancestors(&self) -> $crate::[<$platform:camel Ancestors>]<'_> {
                $crate::[<$platform:camel Ancestors>] { next: Some(&self) }
            }

            #[doc = " Returns the final component of the [`" [<$platform:camel Path>] "`], if there is one."]
            #[doc = ""]
            #[doc = " If the path is a normal file, this is the file name. If it's the path of a directory, this"]
            #[doc = " is the directory name."]
            #[doc = ""]
            #[doc = " Returns [`None`] if the path terminates in `..`."]
            #[doc = ""]
            #[doc = " # Examples"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::" [<$platform:camel Path>] ";"]
            #[doc = ""]
            #[doc = " assert_eq!(Some(\"bin\"), " [<$platform:camel Path>] "::new(\"" $sep "usr" $sep "bin" $sep "\").file_name());"]
            #[doc = " assert_eq!(Some(\"foo.txt\"), " [<$platform:camel Path>] "::new(\"tmp" $sep "foo.txt\").file_name());"]
            #[doc = " assert_eq!(Some(\"foo.txt\"), " [<$platform:camel Path>] "::new(\"foo.txt" $sep ".\").file_name());"]
            #[doc = " assert_eq!(Some(\"foo.txt\"), " [<$platform:camel Path>] "::new(\"foo.txt" $sep "." $sep $sep "\").file_name());"]
            #[doc = " assert_eq!(None, " [<$platform:camel Path>] "::new(\"foo.txt" $sep "..\").file_name());"]
            #[doc = " assert_eq!(None, " [<$platform:camel Path>] "::new(\"" $sep "\").file_name());"]
            #[doc = " ```"]
            pub fn file_name(&self) -> Option<&str> {
                self.components().next_back().and_then(|p| match p {
                    $crate::[<$platform:camel Component>]::Normal(p) => Some(p),
                    _ => None,
                })
            }

            #[doc = " Returns a path that, when joined onto `base`, yields `self`."]
            #[doc = ""]
            #[doc = " # Errors"]
            #[doc = ""]
            #[doc = " If `base` is not a prefix of `self` (i.e., [`starts_with`]"]
            #[doc = " returns `false`), returns [`Err`]."]
            #[doc = ""]
            #[doc = " [`starts_with`]:" [<$platform:camel  >] "Path::starts_with"]
            #[doc = ""]
            #[doc = " # Examples"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::{" [<$platform:camel Path>] ", " [<$platform:camel PathBuf>] "};"]
            #[doc = ""]
            #[doc = " let path = " [<$platform:camel Path>] "::new(\"" $sep "test" $sep "haha" $sep "foo.txt\");"]
            #[doc = ""]
            #[doc = " assert_eq!(path.strip_prefix(\"" $sep "\"), Ok(" [<$platform:camel Path>] "::new(\"test" $sep "haha" $sep "foo.txt\")));"]
            #[doc = " assert_eq!(path.strip_prefix(\"" $sep "test\"), Ok(" [<$platform:camel Path>] "::new(\"haha" $sep "foo.txt\")));"]
            #[doc = " assert_eq!(path.strip_prefix(\"" $sep "test" $sep "\"), Ok(" [<$platform:camel Path>] "::new(\"haha" $sep "foo.txt\")));"]
            #[doc = " assert_eq!(path.strip_prefix(\"" $sep "test" $sep "haha" $sep "foo.txt\"), Ok(" [<$platform:camel Path>] "::new(\"\")));"]
            #[doc = " assert_eq!(path.strip_prefix(\"" $sep "test" $sep "haha" $sep "foo.txt" $sep "\"), Ok(" [<$platform:camel Path>] "::new(\"\")));"]
            #[doc = ""]
            #[doc = " assert!(path.strip_prefix(\"test\").is_err());"]
            #[doc = " assert!(path.strip_prefix(\"" $sep "haha\").is_err());"]
            #[doc = ""]
            #[doc = " let prefix = " [<$platform:camel PathBuf>] "::from(\"" $sep "test" $sep "\");"]
            #[doc = " assert_eq!(path.strip_prefix(prefix), Ok(" [<$platform:camel Path>] "::new(\"haha" $sep "foo.txt\")));"]
            #[doc = " ```"]
            pub fn strip_prefix<P>(&self, base: P) -> Result<&[<$platform:camel Path>], StripPrefixError>
            where
                P: AsRef<[<$platform:camel Path>]>,
            {
                self._strip_prefix(base.as_ref())
            }

            fn _strip_prefix(&self, base: &[<$platform:camel Path>]) -> Result<&[<$platform:camel Path>], StripPrefixError> {
                [<$platform:snake:lower _iter_after>](self.components(), base.components())
                    .map(|c| c.as_path())
                    .ok_or(StripPrefixError)
            }

            #[doc = " Determines whether `base` is a prefix of `self`."]
            #[doc = ""]
            #[doc = " Only considers whole path components to match."]
            #[doc = ""]
            #[doc = " # Examples"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::" [<$platform:camel Path>] ";"]
            #[doc = ""]
            #[doc = " let path = " [<$platform:camel Path>] "::new(\"" $sep "etc" $sep "passwd\");"]
            #[doc = ""]
            #[doc = " assert!(path.starts_with(\"" $sep "etc\"));"]
            #[doc = " assert!(path.starts_with(\"" $sep "etc" $sep "\"));"]
            #[doc = " assert!(path.starts_with(\"" $sep "etc" $sep "passwd\"));"]
            #[doc = " assert!(path.starts_with(\"" $sep "etc" $sep "passwd" $sep "\")); // extra slash is okay"]
            #[doc = " assert!(path.starts_with(\"" $sep "etc" $sep "passwd" $sep $sep $sep "\")); // multiple extra slashes are okay"]
            #[doc = ""]
            #[doc = " assert!(!path.starts_with(\"" $sep "e\"));"]
            #[doc = " assert!(!path.starts_with(\"" $sep "etc" $sep "passwd.txt\"));"]
            #[doc = ""]
            #[doc = " assert!(!" [<$platform:camel Path>] "::new(\"" $sep "etc" $sep "foo.rs\").starts_with(\"" $sep "etc" $sep "foo\"));"]
            #[doc = " ```"]
            pub fn starts_with<P: AsRef<[<$platform:camel Path>]>>(&self, base: P) -> bool {
                self._starts_with(base.as_ref())
            }

            fn _starts_with(&self, base: &[<$platform:camel Path>]) -> bool {
                [<$platform:snake:lower _iter_after>](
                    self.components(),
                    base.components(),
                ).is_some()
            }

            #[doc = " Determines whether `child` is a suffix of `self`."]
            #[doc = ""]
            #[doc = " Only considers whole path components to match."]
            #[doc = ""]
            #[doc = " # Examples"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::" [<$platform:camel Path>] ";"]
            #[doc = ""]
            #[doc = " let path = Path::new(\"" $sep "etc" $sep "resolv.conf\");"]
            #[doc = ""]
            #[doc = " assert!(path.ends_with(\"resolv.conf\"));"]
            #[doc = " assert!(path.ends_with(\"etc" $sep "resolv.conf\"));"]
            #[doc = " assert!(path.ends_with(\"" $sep "etc" $sep "resolv.conf\"));"]
            #[doc = ""]
            #[doc = " assert!(!path.ends_with(\"" $sep "resolv.conf\"));"]
            #[doc = " assert!(!path.ends_with(\"conf\")); // use .extension() instead"]
            #[doc = " ```"]
            pub fn ends_with<P: AsRef<[<$platform:camel Path>]>>(&self, child: P) -> bool {
                self._ends_with(child.as_ref())
            }

            fn _ends_with(&self, child: &[<$platform:camel Path>]) -> bool {
                [<$platform:snake:lower _iter_after>](
                    self.components().rev(),
                    child.components().rev(),
                ).is_some()
            }

            #[doc = " Extracts the stem (non-extension) portion of [`self.file_name`]."]
            #[doc = ""]
            #[doc = " [`self.file_name`]: " [<$platform:camel Path>] "::file_name"]
            #[doc = ""]
            #[doc = " The stem is:"]
            #[doc = ""]
            #[doc = " * [`None`], if there is no file name;"]
            #[doc = " * The entire file name if there is no embedded `.`;"]
            #[doc = " * The entire file name if the file name begins with `.` and has no other `.`s within;"]
            #[doc = " * Otherwise, the portion of the file name before the final `.`"]
            #[doc = ""]
            #[doc = " # Examples"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::" [<$platform:camel Path>] ";"]
            #[doc = ""]
            #[doc = " assert_eq!(\"foo\", Path::new(\"foo.rs\").file_stem().unwrap());"]
            #[doc = " assert_eq!(\"foo.tar\", Path::new(\"foo.tar.gz\").file_stem().unwrap());"]
            #[doc = " ```"]
            pub fn file_stem(&self) -> Option<&str> {
                self.file_name().map(rsplit_file_at_dot).and_then(|(before, after)| before.or(after))
            }

            #[doc = " Extracts the extension of [`self.file_name`], if possible."]
            #[doc = ""]
            #[doc = " The extension is:"]
            #[doc = ""]
            #[doc = " * [`None`], if there is no file name;"]
            #[doc = " * [`None`], if there is no embedded `.`;"]
            #[doc = " * [`None`], if the file name begins with `.` and has no other `.`s within;"]
            #[doc = " * Otherwise, the portion of the file name after the final `.`"]
            #[doc = ""]
            #[doc = " [`self.file_name`]: " [<$platform:camel Path>] "::file_name"]
            #[doc = ""]
            #[doc = " # Examples"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::" [<$platform:camel Path>] ";"]
            #[doc = ""]
            #[doc = " assert_eq!(\"rs\", " [<$platform:camel Path>] "::new(\"foo.rs\").extension().unwrap());"]
            #[doc = " assert_eq!(\"gz\", " [<$platform:camel Path>] "::new(\"foo.tar.gz\").extension().unwrap());"]
            #[doc = " ```"]
            pub fn extension(&self) -> Option<&str> {
                self.file_name().map(rsplit_file_at_dot).and_then(|(before, after)| before.and(after))
            }

            #[doc = " Creates an owned [`" [<$platform:camel PathBuf>] "`] with `path` adjoined to `self`."]
            #[doc = ""]
            #[doc = " See [`" [<$platform:camel PathBuf>] "::push`] for more details on what it means to adjoin a path."]
            #[doc = ""]
            #[doc = " # Examples"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::{" [<$platform:camel Path>] ", " [<$platform:camel PathBuf>] "};"]
            #[doc = ""]
            #[doc = " assert_eq!(" [<$platform:camel Path>] "::new(\"" $sep "etc\").join(\"passwd\"), " [<$platform:camel PathBuf>] "::from(\"" $sep "etc" $sep "passwd\"));"]
            #[doc = " ```"]
            pub fn join<P: AsRef<[<$platform:camel Path>]>>(&self, path: P) -> $crate::[<$platform:camel PathBuf>] {
                self._join(path.as_ref())
            }

            fn _join(&self, path: &[<$platform:camel Path>]) -> $crate::[<$platform:camel PathBuf>] {
                let mut buf = self.to_path_buf();
                buf.push(path);
                buf
            }

            #[doc = " Creates an owned [`" [<$platform:camel PathBuf>] "`] like `self` but with the given file name."]
            #[doc = ""]
            #[doc = " See [`" [<$platform:camel PathBuf>] "::set_file_name`] for more details."]
            #[doc = ""]
            #[doc = " # Examples"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::{" [<$platform:camel Path>] ", " [<$platform:camel PathBuf>] "};"]
            #[doc = ""]
            #[doc = " let path = " [<$platform:camel Path>] "::new(\"" $sep "tmp" $sep "foo.txt\");"]
            #[doc = " assert_eq!(path.with_file_name(\"bar.txt\"), " [<$platform:camel PathBuf>] "::from(\"" $sep "tmp" $sep "bar.txt\"));"]
            #[doc = ""]
            #[doc = " let path = " [<$platform:camel Path>] "::new(\"" $sep "tmp\");"]
            #[doc = " assert_eq!(path.with_file_name(\"var\"), " [<$platform:camel PathBuf>] "::from(\"" $sep "var\"));"]
            #[doc = " ```"]
            pub fn with_file_name<S: AsRef<str>>(&self, file_name: S) -> $crate::[<$platform:camel PathBuf>] {
                self._with_file_name(file_name.as_ref())
            }

            fn _with_file_name(&self, file_name: &str) -> $crate::[<$platform:camel PathBuf>] {
                let mut buf = self.to_path_buf();
                buf.set_file_name(file_name);
                buf
            }

            #[doc = " Creates an owned [`" [<$platform:camel PathBuf>] "`] like `self` but with the given extension."]
            #[doc = ""]
            #[doc = " See [`" [<$platform:camel PathBuf>] "::set_extension`] for more details."]
            #[doc = ""]
            #[doc = " # Examples"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::{" [<$platform:camel Path>] ", " [<$platform:camel PathBuf>] "};"]
            #[doc = ""]
            #[doc = " let path = " [<$platform:camel Path>] "::new(\"foo.rs\");"]
            #[doc = " assert_eq!(path.with_extension(\"txt\"), " [<$platform:camel PathBuf>] "::from(\"foo.txt\"));"]
            #[doc = ""]
            #[doc = " let path = " [<$platform:camel Path>] "::new(\"foo.tar.gz\");"]
            #[doc = " assert_eq!(path.with_extension(\"\"), " [<$platform:camel PathBuf>] "::from(\"foo.tar\"));"]
            #[doc = " assert_eq!(path.with_extension(\"xz\"), " [<$platform:camel PathBuf>] "::from(\"foo.tar.xz\"));"]
            #[doc = " assert_eq!(path.with_extension(\"\").with_extension(\"txt\"), " [<$platform:camel PathBuf>] "::from(\"foo.txt\"));"]
            #[doc = " ```"]
            pub fn with_extension<S: AsRef<str>>(&self, extension: S) -> $crate::[<$platform:camel PathBuf>] {
                self._with_extension(extension.as_ref())
            }

            fn _with_extension(&self, extension: &str) -> $crate::[<$platform:camel PathBuf>] {
                let mut buf = self.to_path_buf();
                buf.set_extension(extension);
                buf
            }

            #[doc = " Produces an iterator over the [`" [<$platform:camel Component>] "`]s of the path."]
            #[doc = ""]
            #[doc = " When parsing the path, there is a small amount of normalization:"]
            #[doc = ""]
            #[doc = " * Repeated separators are ignored, so `a" $sep "b` and `a" $sep $sep "b` both have"]
            #[doc = "   `a` and `b` as components."]
            #[doc = ""]
            #[doc = " * Occurrences of `.` are normalized away, except if they are at the"]
            #[doc = "   beginning of the path. For example, `a" $sep "." $sep "b`, `a" $sep "b" $sep "`, `a" $sep "b" $sep ".` and"]
            #[doc = "   `a" $sep "b` all have `a` and `b` as components, but `." $sep "a" $sep "b` starts with"]
            #[doc = "   an additional [`CurDir`] component."]
            #[doc = ""]
            #[doc = " * A trailing slash is normalized away, `" $sep "a" $sep "b` and `" $sep "a" $sep "b" $sep "` are equivalent."]
            #[doc = ""]
            #[doc = " Note that no other normalization takes place; in particular, `a" $sep "c`"]
            #[doc = " and `a" $sep "b" $sep ".." $sep "c` are distinct, to account for the possibility that `b`"]
            #[doc = " is a symbolic link (so its parent isn't `a`)."]
            #[doc = ""]
            #[doc = " # Examples"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::{" [<$platform:camel Path>] ", " [<$platform:camel Component>] "};"]
            #[doc = ""]
            #[doc = " let mut components = " [<$platform:camel Path>] "::new(\"" $sep "tmp" $sep "foo.txt\").components();"]
            #[doc = ""]
            #[doc = " assert_eq!(components.next(), Some(" [<$platform:camel Component>] "::RootDir));"]
            #[doc = " assert_eq!(components.next(), Some(" [<$platform:camel Component>] "::Normal(\"tmp\")));"]
            #[doc = " assert_eq!(components.next(), Some(" [<$platform:camel Component>] "::Normal(\"foo.txt\")));"]
            #[doc = " assert_eq!(components.next(), None)"]
            #[doc = " ```"]
            #[doc = ""]
            #[doc = " [`CurDir`]: " [<$platform:camel Component>] "::CurDir"]
            pub fn components(&self) -> $crate::[<$platform:camel Components>] {
                $crate::parser::[<$platform:snake:lower>]::parse_components(&self.inner)
            }

            #[doc = "Produces an iterator over the path's components viewed as [`str`] slices."]
            #[doc = ""]
            #[doc = " For more information about the particulars of how the path is separated"]
            #[doc = " into components, see [`components`]."]
            #[doc = ""]
            #[doc = " [`components`]: " [<$platform:camel Path>] "::components"]
            #[doc = ""]
            #[doc = " # Examples"]
            #[doc = ""]
            #[doc = " ```"]
            #[doc = " use typed_path::" [<$platform:camel Path>] ";"]
            #[doc = ""]
            #[doc = " let mut it = " [<$platform:camel Path>] "::new(\"" $sep "tmp" $sep "foo.txt\").iter();"]
            #[doc = " assert_eq!(it.next(), Some(&typed_path::" [<$platform:snake:upper _SEPARATOR_STR>] "));"]
            #[doc = " assert_eq!(it.next(), Some(\"tmp\"));"]
            #[doc = " assert_eq!(it.next(), Some(\"foo.txt\"));"]
            #[doc = " assert_eq!(it.next(), None)"]
            #[doc = " ```"]
            #[inline]
            pub fn iter(&self) -> $crate::[<$platform:camel PathIter>]<'_> {
                $crate::[<$platform:camel PathIter>] { inner: self.components() }
            }

            #[doc = "Converts a [`Box<" [<$platform:camel Path>] ">`](Box) into a"]
            #[doc = "[`" [<$platform:camel PathBuf>] "`] without copying or allocating."]
            pub fn into_path_buf(self: Box<[<$platform:camel Path>]>) -> $crate::[<$platform:camel PathBuf>] {
                let rw = Box::into_raw(self) as *mut str;
                let inner = unsafe { Box::from_raw(rw) };
                $crate::[<$platform:camel PathBuf>] { inner: String::from(inner) }
            }
        }

        impl AsRef<str> for [<$platform:camel Path>] {
            #[inline]
            fn as_ref(&self) -> &str {
                &self.inner
            }
        }

        impl fmt::Debug for [<$platform:camel Path>] {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&self.inner, formatter)
            }
        }

        impl fmt::Display for [<$platform:camel Path>] {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Display::fmt(&self.inner, f)
            }
        }

        impl cmp::PartialEq for [<$platform:camel Path>] {
            #[inline]
            fn eq(&self, other: &[<$platform:camel Path>]) -> bool {
                self.components() == other.components()
            }
        }

        impl cmp::Eq for [<$platform:camel Path>] {}

        impl cmp::PartialOrd for [<$platform:camel Path>] {
            #[inline]
            fn partial_cmp(&self, other: &[<$platform:camel Path>]) -> Option<cmp::Ordering> {
                self.components().partial_cmp(other.components())
            }
        }

        impl cmp::Ord for [<$platform:camel Path>] {
            #[inline]
            fn cmp(&self, other: &[<$platform:camel Path>]) -> cmp::Ordering {
                self.components().cmp(other.components())
            }
        }

        impl AsRef<[<$platform:camel Path>]> for [<$platform:camel Path>] {
            #[inline]
            fn as_ref(&self) -> &[<$platform:camel Path>] {
                self
            }
        }

        impl AsRef<[<$platform:camel Path>]> for Cow<'_, str> {
            #[inline]
            fn as_ref(&self) -> &[<$platform:camel Path>] {
                [<$platform:camel Path>]::new(self)
            }
        }

        impl AsRef<[<$platform:camel Path>]> for str {
            #[inline]
            fn as_ref(&self) -> &[<$platform:camel Path>] {
                [<$platform:camel Path>]::new(self)
            }
        }

        impl AsRef<[<$platform:camel Path>]> for String {
            #[inline]
            fn as_ref(&self) -> &[<$platform:camel Path>] {
                [<$platform:camel Path>]::new(self)
            }
        }

        impl<'a> IntoIterator for &'a [<$platform:camel Path>] {
            type Item = &'a str;
            type IntoIter = $crate::[<$platform:camel PathIter>]<'a>;

            #[inline]
            fn into_iter(self) -> Self::IntoIter {
                $crate::[<$platform:camel PathIter>] {
                    inner: self.components(),
                }
            }
        }

        impl ToOwned for [<$platform:camel Path>] {
            type Owned = $crate::[<$platform:camel PathBuf>];

            #[inline]
            fn to_owned(&self) -> $crate::[<$platform:camel PathBuf>] {
                self.to_path_buf()
            }
        }

        // Iterate through `iter` while it matches `prefix`; return `None` if `prefix`
        // is not a prefix of `iter`, otherwise return `Some(iter_after_prefix)` giving
        // `iter` after having exhausted `prefix`.
        fn [<$platform:snake:lower _iter_after>]<'a, 'b, I, J>(
            mut iter: I,
            mut prefix: J,
        ) -> Option<I>
        where
            I: Iterator<Item = [<$platform:camel Component>]<'a>> + Clone,
            J: Iterator<Item = [<$platform:camel Component>]<'b>>,
        {
            loop {
                let mut iter_next = iter.clone();
                match (iter_next.next(), prefix.next()) {
                    (Some(ref x), Some(ref y)) if x == y => (),
                    (Some(_), Some(_)) => return None,
                    (Some(_), None) => return Some(iter),
                    (None, None) => return Some(iter),
                    (None, Some(_)) => return None,
                }
                iter = iter_next;
            }
        }
    }};
}

impl_path!(Windows "\\\\" "C:");
impl_path!(Unix "/" "");

impl WindowsPath {
    /// Returns `true` if the [`WindowsPath`] is absolute, i.e., if it is independent of the
    /// current directory.
    ///
    /// On Windows, a path is absolute if it has a prefix and starts with the root: `c:\windows` is
    /// absolute, while `c:temp` and `\\temp` are not.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::WindowsPath;
    ///
    /// assert!(WindowsPath::new(r"C:\foo.txt").is_absolute());
    /// assert!(!WindowsPath::new(r"C:foo.txt").is_absolute());
    /// assert!(!WindowsPath::new(r"\foo.txt").is_absolute());
    /// assert!(!WindowsPath::new("foo.txt").is_absolute());
    /// ```
    ///
    /// [`has_root`]: UnixPath::has_root
    #[inline]
    pub fn is_absolute(&self) -> bool {
        self.has_root()
            && matches!(
                self.components().next(),
                Some(crate::WindowsComponent::Prefix(_))
            )
    }

    /// Returns `true` if [`WindowsPath`] has a root.
    ///
    /// On Windows, a path has a root if it:
    ///
    /// * has no prefix and begins with a separator, e.g., `\\windows`
    /// * has a prefix followed by a separator, e.g., `c:\\windows` but not `c:windows`
    /// * has any non-disk prefix, e.g., `\\\\server\\share`
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::WindowsPath;
    ///
    /// // No prefix, but begins with a separator
    /// assert!(WindowsPath::new(r"\windows").has_root());
    ///
    /// // Disk prefix followed by a separator
    /// assert!(WindowsPath::new(r"c:\windows").has_root());
    /// assert!(WindowsPath::new(r"\\?\c:\windows").has_root());
    ///
    /// // Non-disk prefixes
    /// assert!(WindowsPath::new(r"\\?\pictures").has_root());
    /// assert!(WindowsPath::new(r"\\?\UNC\server").has_root());
    /// assert!(WindowsPath::new(r"\\?\UNC\server\share").has_root());
    /// assert!(WindowsPath::new(r"\\.\BrainInterface").has_root());
    ///
    /// // Non-root examples
    /// assert!(!WindowsPath::new(r"c:windows").has_root());
    /// assert!(!WindowsPath::new(r"\\?\c:windows").has_root());
    /// assert!(!WindowsPath::new(r"etc\passwd").has_root());
    /// ```
    pub fn has_root(&self) -> bool {
        let mut components = self.components();
        match components.next() {
            Some(WindowsComponent::RootDir) => true,
            Some(WindowsComponent::Prefix(x)) => match x.kind() {
                Prefix::Disk(_) | Prefix::VerbatimDisk(_) => {
                    matches!(components.next(), Some(WindowsComponent::RootDir))
                }
                _ => true,
            },
            _ => false,
        }
    }
}

impl UnixPath {
    /// Returns `true` if the [`UnixPath`] is absolute, i.e., if it is independent of the current
    /// directory.
    ///
    /// On Unix, a path is absolute if it starts with the root, so `is_absolute` and [`has_root`]
    /// are equivalent.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::UnixPath;
    ///
    /// assert!(UnixPath::new("/foo.txt").is_absolute());
    /// assert!(!UnixPath::new("foo.txt").is_absolute());
    /// ```
    ///
    /// [`has_root`]: UnixPath::has_root
    #[inline]
    pub fn is_absolute(&self) -> bool {
        self.has_root()
    }

    /// Returns `true` if [`UnixPath`] has a root.
    ///
    /// On Unix, a path has a root if it begins with `/`.
    ///
    /// # Examples
    ///
    /// ```
    /// use typed_path::UnixPath;
    ///
    /// assert!(UnixPath::new("/etc/passwd").has_root());
    /// assert!(!UnixPath::new("etc/passwd").has_root());
    /// ```
    #[inline]
    pub fn has_root(&self) -> bool {
        matches!(self.components().next(), Some(UnixComponent::RootDir))
    }
}
