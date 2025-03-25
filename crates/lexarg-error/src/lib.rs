//! Error type for use with lexarg
//!
//! Inspired by [lexopt](https://crates.io/crates/lexopt), `lexarg` simplifies the formula down
//! further so it can be used for CLI plugin systems.
//!
//! ## Example
//! ```no_run
//! use lexarg_error::Error;
//! use lexarg_error::ErrorContext;
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
//!     let bin_name = parser
//!         .next_raw()
//!         .expect("nothing parsed yet so no attached lingering")
//!         .expect("always at least one");
//!     while let Some(arg) = parser.next_arg() {
//!         match arg {
//!             Short("n") | Long("number") => {
//!                 let value = parser
//!                     .next_flag_value()
//!                     .ok_or_else(|| ErrorContext::msg("missing required value").within(arg))?;
//!                 number = value
//!                     .to_str().ok_or_else(|| ErrorContext::msg("invalid number").unexpected(lexarg::Arg::Value(value)).within(arg))?
//!                     .parse().map_err(|e| ErrorContext::msg(e).unexpected(lexarg::Arg::Value(value)).within(arg))?;
//!             }
//!             Long("shout") => {
//!                 shout = true;
//!             }
//!             Value(val) if thing.is_none() => {
//!                 thing = Some(val
//!                     .to_str()
//!                     .ok_or_else(|| ErrorContext::msg("invalid number").unexpected(arg))?
//!                 );
//!             }
//!             Long("help") => {
//!                 println!("Usage: hello [-n|--number=NUM] [--shout] THING");
//!                 std::process::exit(0);
//!             }
//!             _ => {
//!                 return Err(ErrorContext::msg("unexpected argument").unexpected(arg).within(lexarg::Arg::Value(bin_name)).into());
//!             }
//!         }
//!     }
//!
//!     Ok(Args {
//!         thing: thing.ok_or_else(|| ErrorContext::msg("missing argument THING").within(lexarg::Arg::Value(bin_name)))?.to_owned(),
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

/// `Result` that defaults to [`Error`]
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

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.msg.fmt(formatter)
    }
}

/// Collect context for creating an [`Error`]
#[derive(Debug)]
pub struct ErrorContext<'a> {
    msg: String,
    within: Option<lexarg::Arg<'a>>,
    unexpected: Option<lexarg::Arg<'a>>,
}

impl<'a> ErrorContext<'a> {
    /// Create a new error object from a printable error message.
    #[cold]
    pub fn msg<M>(message: M) -> Self
    where
        M: std::fmt::Display,
    {
        Self {
            msg: message.to_string(),
            within: None,
            unexpected: None,
        }
    }

    /// [`Arg`][lexarg::Arg] the error occurred within
    #[cold]
    pub fn within(mut self, within: lexarg::Arg<'a>) -> Self {
        self.within = Some(within);
        self
    }

    /// The failing [`Arg`][lexarg::Arg]
    #[cold]
    pub fn unexpected(mut self, unexpected: lexarg::Arg<'a>) -> Self {
        self.unexpected = Some(unexpected);
        self
    }
}

impl<E> From<E> for ErrorContext<'_>
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[cold]
    fn from(error: E) -> Self {
        Self::msg(error)
    }
}

impl std::fmt::Display for ErrorContext<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.msg.fmt(formatter)?;
        if let Some(unexpected) = &self.unexpected {
            write!(formatter, ", found `")?;
            match unexpected {
                lexarg::Arg::Short(short) => write!(formatter, "-{short}")?,
                lexarg::Arg::Long(long) => write!(formatter, "--{long}")?,
                lexarg::Arg::Escape(value) => write!(formatter, "{value}")?,
                lexarg::Arg::Value(value) | lexarg::Arg::Unexpected(value) => {
                    write!(formatter, "{}", value.to_string_lossy())?;
                }
            }
            write!(formatter, "`")?;
        }
        if let Some(within) = &self.within {
            write!(formatter, " when parsing `")?;
            match within {
                lexarg::Arg::Short(short) => write!(formatter, "-{short}")?,
                lexarg::Arg::Long(long) => write!(formatter, "--{long}")?,
                lexarg::Arg::Escape(value) => write!(formatter, "{value}")?,
                lexarg::Arg::Value(value) | lexarg::Arg::Unexpected(value) => {
                    write!(formatter, "{}", value.to_string_lossy())?;
                }
            }
            write!(formatter, "`")?;
        }
        Ok(())
    }
}
