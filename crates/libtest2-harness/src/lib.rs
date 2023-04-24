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
//! And in `tests/mytest.rs` you would call [`run`] in the `main` function:
//!
//! ```no_run
//! libtest2_harness::Harness::new()
//!     .main();
//! ```
//!

mod case;
mod harness;
mod outcomes;
mod state;

pub mod cli;

pub use case::*;
pub use harness::*;
pub(crate) use outcomes::*;
pub use state::*;
