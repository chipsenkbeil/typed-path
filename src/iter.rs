use paste::paste;
use std::{ffi::OsStr, fmt, iter::FusedIterator};

macro_rules! impl_iter {
    ($($platform:ident),+ $(,)?) => {paste!{$(
        #[doc = "An iterator over the [`" [<$platform:camel Component>] "`]s of a [`" [<$platform:camel Path>] "`], as [`[u8]`] slices."]
        #[doc = ""]
        #[doc = "This `struct` is created by the [`iter`] method on [`" [<$platform:camel Path>] "`]"]
        #[doc = "See its documentation for more."]
        #[doc = ""]
        #[doc = "[`iter`]: " [<$platform:camel Path>] "::iter"]
        pub struct [<$platform:camel PathIter>]<'a> {
            pub(crate) inner: $crate::[<$platform:camel Components>]<'a>,
        }

        impl<'a> [<$platform:camel PathIter>]<'a> {
            #[doc = "Extracts a slice corresponding to the portion of the path remaining for iteration."]
            #[doc = ""]
            #[doc = "# Examples"]
            #[doc = ""]
            #[doc = "```"]
            #[doc = "use typed_path::" [<$platform:camel Path>] ";"]
            #[doc = ""]
            #[doc = "let mut iter = " [<$platform:camel Path>] "::new(\"/tmp/foo/bar.txt\").iter();"]
            #[doc = "iter.next();"]
            #[doc = "iter.next();"]
            #[doc = ""]
            #[doc = "assert_eq!(" [<$platform:camel Path>] "::new(\"foo/bar.txt\"), iter.as_path());"]
            #[doc = "```"]
            #[inline]
            pub fn as_path(&self) -> &'a $crate::[<$platform:camel Path>] {
                self.inner.as_path()
            }
        }

        impl fmt::Debug for [<$platform:camel PathIter>]<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                struct DebugHelper<'a>(&'a $crate::[<$platform:camel Path>]);

                impl fmt::Debug for DebugHelper<'_> {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.debug_list().entries(self.0.iter()).finish()
                    }
                }

                f.debug_tuple(stringify!([<$platform:camel PathIter>]))
                    .field(&DebugHelper(self.as_path()))
                    .finish()
            }
        }

        impl AsRef<$crate::[<$platform:camel Path>]> for [<$platform:camel PathIter>]<'_> {
            #[inline]
            fn as_ref(&self) -> &$crate::[<$platform:camel Path>] {
                self.as_path()
            }
        }

        impl AsRef<str> for [<$platform:camel PathIter>]<'_> {
            #[inline]
            fn as_ref(&self) -> &str {
                self.as_path().as_str()
            }
        }

        impl<'a> Iterator for [<$platform:camel PathIter>]<'a> {
            type Item = &'a OsStr;

            #[inline]
            fn next(&mut self) -> Option<&'a OsStr> {
                self.inner.next().map($crate::[<$platform:camel Component>]::as_os_str)
            }
        }

        impl<'a> DoubleEndedIterator for [<$platform:camel PathIter>]<'a> {
            #[inline]
            fn next_back(&mut self) -> Option<&'a OsStr> {
                self.inner.next_back().map($crate::[<$platform:camel Component>]::as_os_str)
            }
        }

        impl FusedIterator for [<$platform:camel PathIter>]<'_> {}
    )+}};
}

macro_rules! impl_ancestors {
    ($platform:ident $sep:literal) => {paste!{
        #[doc = " An iterator over [`" [<$platform:camel Path>] "`] and its ancestors."]
        #[doc = ""]
        #[doc = " This `struct` is created by the [`ancestors`] method on [`" [<$platform:camel Path>] "`]."]
        #[doc = " See its documentation for more."]
        #[doc = ""]
        #[doc = " # Examples"]
        #[doc = ""]
        #[doc = " ```"]
        #[doc = " use typed_path::" [<$platform:camel Path>] ";"]
        #[doc = ""]
        #[doc = " let path = " [<$platform:camel Path>] "::new(\"" $sep "foo" $sep "bar\");"]
        #[doc = ""]
        #[doc = " for ancestor in path.ancestors() {"]
        #[doc = "     println!(\"{}\", ancestor);"]
        #[doc = " }"]
        #[doc = " ```"]
        #[doc = ""]
        #[doc = " [`ancestors`]: " [<$platform:camel Path>] "::ancestors"]
        #[derive(Copy, Clone, Debug)]
        pub struct [<$platform:camel Ancestors>]<'a> {
            pub(crate) next: Option<&'a $crate::[<$platform:camel Path>]>,
        }

        impl<'a> Iterator for [<$platform:camel Ancestors>]<'a> {
            type Item = &'a $crate::[<$platform:camel Path>];

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                let next = self.next;
                self.next = next.and_then($crate::[<$platform:camel Path>]::parent);
                next
            }
        }

        impl FusedIterator for [<$platform:camel Ancestors>]<'_> {}
    }}
}

impl_iter!(Windows, Unix);
impl_ancestors!(Windows "\\\\");
impl_ancestors!(Unix "/");
