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
        let mut opts = parse(&mut parser).unwrap_or_else(|err| {
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
        if self.cases.len() == 1 {
            opts.test_threads = Some(std::num::NonZeroUsize::new(1).unwrap());
        }

        if !opts.list {
            match run(&opts, self.cases, notifier.as_mut()) {
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

    let mut opts = test_opts.finish()?;
    // If the platform is single-threaded we're just going to run
    // the test synchronously, regardless of the concurrency
    // level.
    let supports_threads = !cfg!(target_os = "emscripten") && !cfg!(target_family = "wasm");
    opts.test_threads = if cfg!(feature = "threads") && supports_threads {
        opts.test_threads
            .or_else(|| std::thread::available_parallelism().ok())
    } else {
        None
    };
    Ok(opts)
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
    let seed = shuffle::get_shuffle_seed(opts);
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
    cases: Vec<Box<dyn Case>>,
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

    let threads = opts.test_threads.map(|t| t.get()).unwrap_or(1);
    let is_multithreaded = 1 < threads;
    notifier.threaded(is_multithreaded);

    let mut state = State::new();
    let run_ignored = match opts.run_ignored {
        libtest_lexarg::RunIgnored::Yes | libtest_lexarg::RunIgnored::Only => true,
        libtest_lexarg::RunIgnored::No => false,
    };
    state.run_ignored(run_ignored);

    let mut success = true;
    if is_multithreaded {
        struct RunningTest {
            join_handle: std::thread::JoinHandle<()>,
        }

        impl RunningTest {
            fn join(self, event: &mut notify::Event) {
                if self.join_handle.join().is_err() {
                    if let notify::Event::CaseComplete {
                        status, message, ..
                    } = event
                    {
                        if status.is_none() {
                            *status = Some(notify::RunStatus::Failed);
                            *message = Some("panicked after reporting success".to_owned());
                        }
                    }
                }
            }
        }

        // Use a deterministic hasher
        type TestMap = std::collections::HashMap<
            String,
            RunningTest,
            std::hash::BuildHasherDefault<std::collections::hash_map::DefaultHasher>,
        >;

        let sync_success = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(success));
        let mut running_tests: TestMap = Default::default();
        let mut pending = 0;
        let state = std::sync::Arc::new(state);
        let (tx, rx) = std::sync::mpsc::channel::<notify::Event>();
        let mut remaining = std::collections::VecDeque::from(cases);
        while pending > 0 || !remaining.is_empty() {
            while pending < threads && !remaining.is_empty() {
                let case = remaining.pop_front().unwrap();
                let name = case.name().to_owned();

                let cfg = std::thread::Builder::new().name(name.to_owned());
                let tx = tx.clone();
                let case = std::sync::Arc::new(case);
                let case_fallback = case.clone();
                let state = state.clone();
                let state_fallback = state.clone();
                let sync_success = sync_success.clone();
                let sync_success_fallback = sync_success.clone();
                match cfg.spawn(move || {
                    let mut notifier = SenderNotifier { tx: tx.clone() };
                    let case_success = run_case(case.as_ref().as_ref(), &state, &mut notifier)
                        .expect("`SenderNotifier` is infallible");
                    if !case_success {
                        sync_success.store(case_success, std::sync::atomic::Ordering::Relaxed);
                    }
                }) {
                    Ok(join_handle) => {
                        running_tests.insert(name.clone(), RunningTest { join_handle });
                        pending += 1;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // `ErrorKind::WouldBlock` means hitting the thread limit on some
                        // platforms, so run the test synchronously here instead.
                        let case_success =
                            run_case(case_fallback.as_ref().as_ref(), &state_fallback, notifier)
                                .expect("`SenderNotifier` is infallible");
                        if !case_success {
                            sync_success_fallback
                                .store(case_success, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            let mut event = rx.recv().unwrap();
            if let notify::Event::CaseComplete { name, .. } = &event {
                let running_test = running_tests.remove(name).unwrap();
                running_test.join(&mut event);
                pending -= 1;
            }
            notifier.notify(event)?;
            success &= sync_success.load(std::sync::atomic::Ordering::SeqCst);
            if !success && opts.fail_fast {
                break;
            }
        }
    } else {
        for case in cases {
            success &= run_case(case.as_ref(), &state, notifier)?;
            if !success && opts.fail_fast {
                break;
            }
        }
    }

    notifier.notify(notify::Event::SuiteComplete {
        elapsed_s: notify::Elapsed(timer.elapsed()),
    })?;

    Ok(success)
}

fn run_case(
    case: &dyn Case,
    state: &State,
    notifier: &mut dyn notify::Notifier,
) -> std::io::Result<bool> {
    notifier.notify(notify::Event::CaseStart {
        name: case.name().to_owned(),
    })?;
    let timer = std::time::Instant::now();

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        __rust_begin_short_backtrace(|| case.run(state))
    }))
    .unwrap_or_else(|e| {
        // The `panic` information is just an `Any` object representing the
        // value the panic was invoked with. For most panics (which use
        // `panic!` like `println!`), this is either `&str` or `String`.
        let payload = e
            .downcast_ref::<String>()
            .map(|s| s.as_str())
            .or_else(|| e.downcast_ref::<&str>().copied());

        let msg = match payload {
            Some(payload) => format!("test panicked: {payload}"),
            None => "test panicked".to_string(),
        };
        Err(RunError::fail(msg))
    });

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

    Ok(status != Some(notify::RunStatus::Failed))
}

/// Fixed frame used to clean the backtrace with `RUST_BACKTRACE=1`.
#[inline(never)]
fn __rust_begin_short_backtrace<T, F: FnOnce() -> T>(f: F) -> T {
    let result = f();

    // prevent this frame from being tail-call optimised away
    std::hint::black_box(result)
}

#[derive(Clone, Debug)]
struct SenderNotifier {
    tx: std::sync::mpsc::Sender<notify::Event>,
}

impl notify::Notifier for SenderNotifier {
    fn notify(&mut self, event: notify::Event) -> std::io::Result<()> {
        // If the sender doesn't care, neither do we
        let _ = self.tx.send(event);
        Ok(())
    }
}
