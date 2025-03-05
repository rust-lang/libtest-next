//! An experimental replacement for the core of libtest
//!
//! # Usage
//!
//! To use this, you most likely want to add a manual `[[test]]` section to
//! `Cargo.toml` and set `harness = false`. For example:
//!
//! ```toml
//! [[test]]
//! name = "mytest"
//! path = "tests/mytest.rs"
//! harness = false
//! ```
//!
//! And in `tests/mytest.rs` you would call [`Harness::main`] in the `main` function:
//!
//! ```no_run
//! libtest2_harness::Harness::with_env()
//!     .main();
//! ```
//!

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![warn(clippy::print_stderr)]
// #![warn(clippy::print_stdout)]
#![allow(clippy::todo)]

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

mod case;
mod harness;
mod notify;
mod shuffle;
mod state;

pub mod cli;

pub use case::*;
pub use harness::*;
pub use notify::RunMode;
pub use state::*;
