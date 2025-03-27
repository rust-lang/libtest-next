//! Minimal, API stable CLI parser
//!
//! Inspired by [lexopt](https://crates.io/crates/lexopt), `lexarg` simplifies the formula down
//! further so it can be used for CLI plugin systems.
//!
//! ## Example
//!
//! ```no_run
#![doc = include_str!("../examples/hello.rs")]
//! ```

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(clippy::result_unit_err)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

/// Simplify parsing of arguments
pub mod prelude {
    pub use crate::Arg::*;
    pub use crate::OptionContextExt as _;
    pub use crate::ResultContextExt as _;
    pub use crate::ValueExt as _;
}

pub use lexarg_error::ErrorContext;
pub use lexarg_parser::Arg;
pub use lexarg_parser::Parser;
pub use lexarg_parser::RawArgs;

/// `Result` that defaults to [`Error`]
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Argument error type for use with lexarg
pub struct Error {
    msg: String,
}

impl Error {
    /// Create a new error object from a printable error message.
    #[cold]
    pub fn msg<M>(message: M) -> Self
    where
        M: std::fmt::Display,
    {
        Self {
            msg: message.to_string(),
        }
    }
}

impl From<ErrorContext<'_>> for Error {
    #[cold]
    fn from(error: ErrorContext<'_>) -> Self {
        Self::msg(error.to_string())
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.msg)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.msg.fmt(formatter)
    }
}

/// Extensions for parsing [`Arg::Value`]
pub trait ValueExt<'a> {
    /// Convert [`Arg::Value`]
    fn path(self) -> Result<&'a std::path::Path, ErrorContext<'a>>;
    /// Convert [`Arg::Value`] with a description of the intended format
    fn string(self, description: &str) -> Result<&'a str, ErrorContext<'a>>;
    /// Ensure [`Arg::Value`] is from a closed set of values
    fn one_of(self, possible: &[&str]) -> Result<&'a str, ErrorContext<'a>>;
    /// Parse [`Arg::Value`]
    fn parse<T: std::str::FromStr>(self) -> Result<T, ErrorContext<'a>>
    where
        T::Err: std::fmt::Display;
    /// Custom conversion for [`Arg::Value`]
    fn try_map<F, T, E>(self, op: F) -> Result<T, ErrorContext<'a>>
    where
        F: FnOnce(&'a std::ffi::OsStr) -> Result<T, E>,
        E: std::fmt::Display;
}

impl<'a> ValueExt<'a> for &'a std::ffi::OsStr {
    fn path(self) -> Result<&'a std::path::Path, ErrorContext<'a>> {
        Ok(std::path::Path::new(self))
    }
    fn string(self, description: &str) -> Result<&'a str, ErrorContext<'a>> {
        self.to_str().ok_or_else(|| {
            ErrorContext::msg(format_args!("invalid {description}")).unexpected(Arg::Value(self))
        })
    }
    fn one_of(self, possible: &[&str]) -> Result<&'a str, ErrorContext<'a>> {
        self.to_str()
            .filter(|v| possible.contains(v))
            .ok_or_else(|| {
                let mut possible = possible.iter();
                let first = possible.next().expect("at least one possible value");
                let mut error = format!("expected one of `{first}`");
                for possible in possible {
                    use std::fmt::Write as _;
                    let _ = write!(&mut error, ", `{possible}`");
                }
                ErrorContext::msg(error)
            })
    }
    fn parse<T: std::str::FromStr>(self) -> Result<T, ErrorContext<'a>>
    where
        T::Err: std::fmt::Display,
    {
        self.string(std::any::type_name::<T>())?
            .parse::<T>()
            .map_err(|err| ErrorContext::msg(err).unexpected(Arg::Value(self)))
    }
    fn try_map<F, T, E>(self, op: F) -> Result<T, ErrorContext<'a>>
    where
        F: FnOnce(&'a std::ffi::OsStr) -> Result<T, E>,
        E: std::fmt::Display,
    {
        op(self).map_err(|err| ErrorContext::msg(err).unexpected(Arg::Value(self)))
    }
}

impl<'a> ValueExt<'a> for Result<&'a std::ffi::OsStr, ErrorContext<'a>> {
    fn path(self) -> Result<&'a std::path::Path, ErrorContext<'a>> {
        self.and_then(|os| os.path())
    }
    fn string(self, description: &str) -> Result<&'a str, ErrorContext<'a>> {
        self.and_then(|os| os.string(description))
    }
    fn one_of(self, possible: &[&str]) -> Result<&'a str, ErrorContext<'a>> {
        self.and_then(|os| os.one_of(possible))
    }
    fn parse<T: std::str::FromStr>(self) -> Result<T, ErrorContext<'a>>
    where
        T::Err: std::fmt::Display,
    {
        self.and_then(|os| os.parse())
    }
    fn try_map<F, T, E>(self, op: F) -> Result<T, ErrorContext<'a>>
    where
        F: FnOnce(&'a std::ffi::OsStr) -> Result<T, E>,
        E: std::fmt::Display,
    {
        self.and_then(|os| os.try_map(op))
    }
}

/// Extensions for extending [`ErrorContext`]
pub trait ResultContextExt<'a> {
    /// [`Arg`] the error occurred within
    fn within(self, within: Arg<'a>) -> Self;
}

impl<'a, T> ResultContextExt<'a> for Result<T, ErrorContext<'a>> {
    fn within(self, within: Arg<'a>) -> Self {
        self.map_err(|err| err.within(within))
    }
}

/// Extensions for creating an [`ErrorContext`]
pub trait OptionContextExt<T> {
    /// [`Arg`] that was expected
    ///
    /// For [`Arg::Value`], the contents are assumed to be a placeholder
    fn ok_or_missing(self, expected: Arg<'static>) -> Result<T, ErrorContext<'static>>;
}

impl<T> OptionContextExt<T> for Option<T> {
    fn ok_or_missing(self, expected: Arg<'static>) -> Result<T, ErrorContext<'static>> {
        self.ok_or_else(|| match expected {
            Arg::Short(short) => ErrorContext::msg(format_args!("missing required `-{short}`")),
            Arg::Long(long) => ErrorContext::msg(format_args!("missing required `--{long}`")),
            Arg::Escape(escape) => ErrorContext::msg(format_args!("missing required `{escape}`")),
            Arg::Value(value) | Arg::Unexpected(value) => ErrorContext::msg(format_args!(
                "missing required `{}`",
                value.to_string_lossy()
            )),
        })
    }
}
