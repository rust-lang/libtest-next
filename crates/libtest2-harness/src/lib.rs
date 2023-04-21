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

pub mod cli {
    pub use lexarg::*;
    pub use lexarg_error::*;
}

#[non_exhaustive]
pub struct Harness {}

impl Harness {
    pub fn new() -> Self {
        Self {}
    }

    pub fn main(self) -> std::convert::Infallible {
        let raw = std::env::args_os().collect::<Vec<_>>();
        let mut parser = cli::Parser::new(&raw);
        let opts = match parse(&mut parser) {
            Ok(opts) => opts,
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        };

        println!("{:?}", opts);

        std::process::exit(0)
    }
}

fn parse(parser: &mut cli::Parser) -> cli::Result<libtest_lexarg::TestOpts> {
    let mut test_opts = libtest_lexarg::TestOptsParseState::new();

    let bin = parser.bin();
    while let Some(arg) = parser.next() {
        match arg {
            cli::Arg::Short('h') | cli::Arg::Long("help") => {
                let bin = bin
                    .unwrap_or_else(|| std::ffi::OsStr::new("test"))
                    .to_string_lossy();
                let options_help = libtest_lexarg::OPTIONS_HELP.trim();
                let after_help = libtest_lexarg::AFTER_HELP.trim();
                println!(
                    "Usage: {bin} [OPTIONS] [FILTER]...

{options_help}

{after_help}"
                );
                std::process::exit(0);
            }
            _ => {}
        }

        let arg = test_opts.parse_next(parser, arg)?;

        if let Some(arg) = arg {
            let msg = match arg {
                cli::Arg::Short(v) => {
                    format!("unrecognized `-{v}` flag")
                }
                cli::Arg::Long(v) => {
                    format!("unrecognized `--{v}` flag")
                }
                cli::Arg::Value(v) => {
                    format!("unrecognized `{}` value", v.to_string_lossy())
                }
                cli::Arg::Unexpected(v) => {
                    format!("unexpected `{}` value", v.to_string_lossy())
                }
            };
            return Err(cli::Error::msg(msg));
        }
    }

    test_opts.finish()
}
