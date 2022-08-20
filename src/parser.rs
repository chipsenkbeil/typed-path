#[macro_use]
mod utils;

pub mod unix;
pub mod windows;

pub type ParseResult<'a, T> = Result<(ParseInput<'a>, T), ParseError>;
pub type ParseInput<'a> = &'a [u8];
pub type ParseError = &'static str;
