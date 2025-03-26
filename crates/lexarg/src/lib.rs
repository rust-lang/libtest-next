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

pub use lexarg_parser::Arg;
pub use lexarg_parser::Parser;
pub use lexarg_parser::RawArgs;
