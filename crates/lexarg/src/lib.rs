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
        self.msg.fmt(formatter)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.msg.fmt(formatter)
    }
}
