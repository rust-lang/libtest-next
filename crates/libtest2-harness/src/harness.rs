use crate::*;

pub struct Harness {
    cases: Vec<Box<dyn Case>>,
}

impl Harness {
    pub fn new() -> Self {
        Self { cases: vec![] }
    }

    pub fn case(mut self, case: impl Case + 'static) -> Self {
        self.cases.push(Box::new(case));
        self
    }

    pub fn main(mut self) -> std::convert::Infallible {
        let raw = std::env::args_os().collect::<Vec<_>>();
        let mut parser = cli::Parser::new(&raw);
        let opts = match parse(&mut parser) {
            Ok(opts) => opts,
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        };

        match opts.color {
            libtest_lexarg::ColorConfig::AutoColor => anstream::ColorChoice::Auto,
            libtest_lexarg::ColorConfig::AlwaysColor => anstream::ColorChoice::Always,
            libtest_lexarg::ColorConfig::NeverColor => anstream::ColorChoice::Never,
        }
        .write_global();

        let total = self.cases.len();
        let matches_filter = |case: &dyn Case, filter: &str| {
            let test_name = case.name();

            match opts.filter_exact {
                true => test_name == filter,
                false => test_name.contains(filter),
            }
        };
        // Remove tests that don't match the test filter
        if !opts.filters.is_empty() {
            self.cases.retain(|case| {
                opts.filters
                    .iter()
                    .any(|filter| matches_filter(case.as_ref(), filter))
            });
        }
        // Skip tests that match any of the skip filters
        if !opts.skip.is_empty() {
            self.cases
                .retain(|case| !opts.skip.iter().any(|sf| matches_filter(case.as_ref(), sf)));
        }
        let num_filtered_out = total - self.cases.len();
        match run(&opts, &self.cases, num_filtered_out) {
            Ok(true) => std::process::exit(0),
            Ok(false) => std::process::exit(ERROR_EXIT_CODE),
            Err(e) => {
                eprintln!("error: io error when listing tests: {e:?}");
                std::process::exit(ERROR_EXIT_CODE)
            }
        }
    }
}

const ERROR_EXIT_CODE: i32 = 101;

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

fn run(
    opts: &libtest_lexarg::TestOpts,
    cases: &[Box<dyn Case>],
    num_filtered_out: usize,
) -> std::io::Result<bool> {
    let mut outcomes = Outcomes::new(&opts, cases, num_filtered_out)?;

    if opts.list {
        outcomes.list(cases)?;
        Ok(true)
    } else {
        outcomes.start_suite()?;

        let mut state = State::new();
        let run_ignored = match opts.run_ignored {
            libtest_lexarg::RunIgnored::Yes | libtest_lexarg::RunIgnored::Only => true,
            libtest_lexarg::RunIgnored::No => false,
        };
        state.run_ignored(run_ignored);

        for case in cases {
            outcomes.start_case(case.as_ref())?;
            let outcome = case.run(&state);
            outcomes.finish_case(case.as_ref(), outcome)?;
        }

        outcomes.finish_suite()?;

        Ok(!outcomes.has_failed())
    }
}
