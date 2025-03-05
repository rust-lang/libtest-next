//! Argument error type for use with lexarg
//!
//! Inspired by [lexopt](https://crates.io/crates/lexopt), `lexarg` simplifies the formula down
//! further so it can be used for CLI plugin systems.
//!
//! ## Example
//! ```no_run
//! use lexarg_error::Error;
//! use lexarg_error::Result;
//!
//! struct Args {
//!     thing: String,
//!     number: u32,
//!     shout: bool,
//! }
//!
//! fn parse_args() -> Result<Args> {
//!     use lexarg::Arg::*;
//!
//!     let mut thing = None;
//!     let mut number = 1;
//!     let mut shout = false;
//!     let mut raw = std::env::args_os().collect::<Vec<_>>();
//!     let mut parser = lexarg::Parser::new(&raw);
//!     parser.bin();
//!     while let Some(arg) = parser.next_arg() {
//!         match arg {
//!             Short('n') | Long("number") => {
//!                 number = parser
//!                     .flag_value().ok_or_else(|| Error::msg("`--number` requires a value"))?
//!                     .to_str().ok_or_else(|| Error::msg("invalid number"))?
//!                     .parse().map_err(|e| Error::msg(e))?;
//!             }
//!             Long("shout") => {
//!                 shout = true;
//!             }
//!             Value(val) if thing.is_none() => {
//!                 thing = Some(val.to_str().ok_or_else(|| Error::msg("invalid number"))?);
//!             }
//!             Long("help") => {
//!                 println!("Usage: hello [-n|--number=NUM] [--shout] THING");
//!                 std::process::exit(0);
//!             }
//!             _ => {
//!                 return Err(Error::msg("unexpected argument"));
//!             }
//!         }
//!     }
//!
//!     Ok(Args {
//!         thing: thing.ok_or_else(|| Error::msg("missing argument THING"))?.to_owned(),
//!         number,
//!         shout,
//!     })
//! }
//!
//! fn main() -> Result<()> {
//!     let args = parse_args()?;
//!     let mut message = format!("Hello {}", args.thing);
//!     if args.shout {
//!         message = message.to_uppercase();
//!     }
//!     for _ in 0..args.number {
//!         println!("{}", message);
//!     }
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

/// `Result<T, Error>`
///
/// `lexarg_error::Result` may be used with one *or* two type parameters.
///
/// ```rust
/// use lexarg_error::Result;
///
/// # const IGNORE: &str = stringify! {
/// fn demo1() -> Result<T> {...}
///            // ^ equivalent to std::result::Result<T, lexarg_error::Error>
///
/// fn demo2() -> Result<T, OtherError> {...}
///            // ^ equivalent to std::result::Result<T, OtherError>
/// # };
/// ```
///
/// # Example
///
/// ```
/// # pub trait Deserialize {}
/// #
/// # mod serde_json {
/// #     use super::Deserialize;
/// #     use std::io;
/// #
/// #     pub fn from_str<T: Deserialize>(json: &str) -> io::Result<T> {
/// #         unimplemented!()
/// #     }
/// # }
/// #
/// # #[derive(Debug)]
/// # struct ClusterMap;
/// #
/// # impl Deserialize for ClusterMap {}
/// #
/// use lexarg_error::Result;
///
/// fn main() -> Result<()> {
///     # return Ok(());
///     let config = std::fs::read_to_string("cluster.json")?;
///     let map: ClusterMap = serde_json::from_str(&config)?;
///     println!("cluster info: {:#?}", map);
///     Ok(())
/// }
/// ```
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Argument error type for use with lexarg
#[derive(Debug)]
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
        Error {
            msg: message.to_string(),
        }
    }
}

impl<E> From<E> for Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[cold]
    fn from(error: E) -> Self {
        Error::msg(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.msg.fmt(formatter)
    }
}
