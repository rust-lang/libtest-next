use libtest_lexarg::OutputFormat;

use crate::*;

pub struct Harness {
    raw: Vec<std::ffi::OsString>,
    cases: Vec<Box<dyn Case>>,
}

impl Harness {
    pub fn with_args(args: impl IntoIterator<Item = impl Into<std::ffi::OsString>>) -> Self {
        let raw = args.into_iter().map(|s| s.into()).collect::<Vec<_>>();
        Self { raw, cases: vec![] }
    }

    pub fn with_env() -> Self {
        let raw = std::env::args_os().collect::<Vec<_>>();
        Self { raw, cases: vec![] }
    }

    pub fn case(mut self, case: impl Case + 'static) -> Self {
        self.cases.push(Box::new(case));
        self
    }

    pub fn cases(mut self, cases: impl IntoIterator<Item = impl Case + 'static>) -> Self {
        for case in cases {
            self.cases.push(Box::new(case));
        }
        self
    }

    pub fn main(mut self) -> ! {
        let mut parser = cli::Parser::new(&self.raw);
        let opts = parse(&mut parser).unwrap_or_else(|err| {
            eprintln!("{}", err);
            std::process::exit(1)
        });

        match opts.color {
            libtest_lexarg::ColorConfig::AutoColor => anstream::ColorChoice::Auto,
            libtest_lexarg::ColorConfig::AlwaysColor => anstream::ColorChoice::Always,
            libtest_lexarg::ColorConfig::NeverColor => anstream::ColorChoice::Never,
        }
        .write_global();

        let mut notifier = notifier(&opts).unwrap_or_else(|err| {
            eprintln!("{}", err);
            std::process::exit(1)
        });
        discover(&opts, &mut self.cases, notifier.as_mut()).unwrap_or_else(|err| {
            eprintln!("{}", err);
            std::process::exit(1)
        });

        if !opts.list {
            match run(&opts, &self.cases, notifier.as_mut()) {
                Ok(true) => {}
                Ok(false) => std::process::exit(ERROR_EXIT_CODE),
                Err(e) => {
                    eprintln!("error: io error when listing tests: {e:?}");
                    std::process::exit(ERROR_EXIT_CODE)
                }
            }
        }

        std::process::exit(0)
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
                cli::Arg::Escape => "handled `--`".to_owned(),
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

fn notifier(opts: &libtest_lexarg::TestOpts) -> std::io::Result<Box<dyn notify::Notifier>> {
    let stdout = anstream::stdout();
    let notifier: Box<dyn notify::Notifier> = match opts.format {
        #[cfg(feature = "json")]
        OutputFormat::Json => Box::new(notify::JsonNotifier::new(stdout)),
        #[cfg(not(feature = "json"))]
        OutputFormat::Json => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "`--format=json` is not supported",
            ));
        }
        _ if opts.list => Box::new(notify::TerseListNotifier::new(stdout)),
        OutputFormat::Pretty => Box::new(notify::PrettyRunNotifier::new(stdout)),
        OutputFormat::Terse => Box::new(notify::TerseRunNotifier::new(stdout)),
        #[cfg(feature = "junit")]
        OutputFormat::Junit => Box::new(notify::JunitRunNotifier::new(stdout)),
        #[cfg(not(feature = "junit"))]
        OutputFormat::Junit => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "`--format=junit` is not supported",
            ));
        }
    };
    Ok(notifier)
}

fn discover(
    opts: &libtest_lexarg::TestOpts,
    cases: &mut Vec<Box<dyn Case>>,
    notifier: &mut dyn notify::Notifier,
) -> std::io::Result<()> {
    notifier.notify(notify::Event::DiscoverStart)?;
    let timer = std::time::Instant::now();

    // Do this first so it applies to both discover and running
    cases.sort_unstable_by_key(|case| case.name().to_owned());
    let seed = shuffle::get_shuffle_seed(&opts);
    if let Some(seed) = seed {
        shuffle::shuffle_tests(seed, cases);
    }

    let matches_filter = |case: &dyn Case, filter: &str| {
        let test_name = case.name();

        match opts.filter_exact {
            true => test_name == filter,
            false => test_name.contains(filter),
        }
    };
    let mut retain_cases = Vec::with_capacity(cases.len());
    for case in cases.iter() {
        let filtered_in = opts.filters.is_empty()
            || opts
                .filters
                .iter()
                .any(|filter| matches_filter(case.as_ref(), filter));
        let filtered_out =
            !opts.skip.is_empty() && opts.skip.iter().any(|sf| matches_filter(case.as_ref(), sf));
        let retain_case = filtered_in && !filtered_out;
        retain_cases.push(retain_case);
        notifier.notify(notify::Event::DiscoverCase {
            name: case.name().to_owned(),
            mode: notify::CaseMode::Test,
            run: retain_case,
        })?;
    }
    let mut retain_cases = retain_cases.into_iter();
    cases.retain(|_| retain_cases.next().unwrap());

    notifier.notify(notify::Event::DiscoverComplete {
        elapsed_s: notify::Elapsed(timer.elapsed()),
        seed,
    })?;

    Ok(())
}

fn run(
    opts: &libtest_lexarg::TestOpts,
    cases: &[Box<dyn Case>],
    notifier: &mut dyn notify::Notifier,
) -> std::io::Result<bool> {
    notifier.notify(notify::Event::SuiteStart)?;
    let timer = std::time::Instant::now();

    if opts.force_run_in_process {
        todo!("`--force-run-in-process` is not yet supported");
    }
    if opts.exclude_should_panic {
        todo!("`--exclude-should-panic` is not yet supported");
    }
    if opts.nocapture {
        todo!("`--nocapture` is not yet supported");
    }
    if opts.time_options.is_some() {
        todo!("`--report-time` / `--ensure-time` are not yet supported");
    }
    if opts.options.display_output {
        todo!("`--show-output` is not yet supported");
    }
    if opts.options.panic_abort {
        todo!("panic-abort is not yet supported");
    }
    if opts.logfile.is_some() {
        todo!("`--logfile` is not yet supported");
    }

    let mut state = State::new();
    let run_ignored = match opts.run_ignored {
        libtest_lexarg::RunIgnored::Yes | libtest_lexarg::RunIgnored::Only => true,
        libtest_lexarg::RunIgnored::No => false,
    };
    state.run_ignored(run_ignored);

    let mut success = true;
    for case in cases {
        notifier.notify(notify::Event::CaseStart {
            name: case.name().to_owned(),
        })?;
        let timer = std::time::Instant::now();

        let outcome = case.run(&state);

        let err = outcome.as_ref().err();
        let status = err.map(|e| e.status());
        let message = err.and_then(|e| e.cause().map(|c| c.to_string()));
        notifier.notify(notify::Event::CaseComplete {
            name: case.name().to_owned(),
            mode: notify::CaseMode::Test,
            status,
            message,
            elapsed_s: Some(notify::Elapsed(timer.elapsed())),
        })?;

        success &= status != Some(notify::RunStatus::Failed);
        if !success && opts.fail_fast {
            break;
        }
    }

    notifier.notify(notify::Event::SuiteComplete {
        elapsed_s: notify::Elapsed(timer.elapsed()),
    })?;

    Ok(success)
}
