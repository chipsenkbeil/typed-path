use crate::{UnixPath, WindowsPath, UNIX_SEPARATOR, WINDOWS_SEPARATOR};
use paste::paste;
use std::{
    cmp,
    collections::VecDeque,
    fmt,
    iter::{DoubleEndedIterator, FusedIterator},
};

mod unix;
mod windows;

pub use unix::*;
pub use windows::*;

macro_rules! impl_components {
    ($($platform:ident),+ $(,)?) => {paste!{$(
        #[derive(Clone)]
        pub struct [<$platform:camel Components>]<'a> {
            /// Represents raw [`str`] that comprises the remaining components
            raw: &'a str,

            /// Represents the parsed components that this iterator can traverse
            components: VecDeque<[<$platform:camel Component>]<'a>>,
        }

        impl<'a> [<$platform:camel Components>]<'a> {
            /// Extracts a slice corresponding to the portion of the path remaining for iteration
            pub fn as_path(&self) -> &'a [<$platform:camel Path>] {
                [<$platform:camel Path>]::new(self.raw)
            }
        }

        impl fmt::Debug for [<$platform:camel Components>]<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                struct DebugHelper<'a>(&'a [<$platform:camel Path>]);

                impl fmt::Debug for DebugHelper<'_> {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.debug_list().entries(self.0.components()).finish()
                    }
                }

                f.debug_tuple(stringify!([<$platform:camel Components>]))
                    .field(&DebugHelper(self.as_path()))
                    .finish()
            }
        }

        impl<'a> Iterator for [<$platform:camel Components>]<'a> {
            type Item = [<$platform:camel Component>]<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                let component = self.components.pop_front();

                // We need to adjust our raw str to advance by the len of the component and all
                // separators leading to the next component
                if let Some(c) = component.as_ref() {
                    // Advance by the len of the component
                    self.raw = &self.raw[c.len()..];

                    // Now advance while we still have separators in front of our next component
                    self.raw = match self.raw.char_indices().find(|(_, c)| *c != [<$platform:snake:upper _SEPARATOR>]) {
                        Some((i, _)) => &self.raw[i..],
                        None => "",
                    };
                }

                component
            }
        }

        impl<'a> DoubleEndedIterator for [<$platform:camel Components>]<'a> {
            fn next_back(&mut self) -> Option<Self::Item> {
                let component = self.components.pop_back();

                // We need to adjust our raw str to trim from the end by the len of the component and all
                // separators leading to the previous component
                if let Some(c) = component.as_ref() {
                    // Trim from end by the len of the component
                    self.raw = &self.raw[..=(self.raw.len() - c.len())];

                    // Now trim from end while we still have separators in after of our last component
                    self.raw = match self
                        .raw
                        .char_indices()
                        .rev()
                        .find(|(_, c)| *c != [<$platform:snake:upper _SEPARATOR>])
                    {
                        Some((i, _)) => &self.raw[..=i],
                        None => "",
                    };
                }

                component
            }
        }

        impl FusedIterator for [<$platform:camel Components>]<'_> {}

        impl<'a> cmp::PartialEq for [<$platform:camel Components>]<'a> {
            #[inline]
            fn eq(&self, other: &[<$platform:camel Components>]<'a>) -> bool {
                self.components == other.components
            }
        }

        impl cmp::Eq for [<$platform:camel Components>]<'_> {}

        impl<'a> cmp::PartialOrd for [<$platform:camel Components>]<'a> {
            #[inline]
            fn partial_cmp(&self, other: &[<$platform:camel Components>]<'a>) -> Option<cmp::Ordering> {
                self.components.partial_cmp(&other.components)
            }
        }

        impl cmp::Ord for [<$platform:camel Components>]<'_> {
            #[inline]
            fn cmp(&self, other: &Self) -> cmp::Ordering {
                self.components.cmp(&other.components)
            }
        }
    )+}};
}

impl_components!(Windows, Unix);
