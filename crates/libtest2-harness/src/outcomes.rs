//! Definition of the `Outcomes`.
//!
//! This is just an abstraction for everything that is printed to the screen
//! (or logfile, if specified). These parameters influence printing:
//! - `color`
//! - `format` (and `quiet`)
//! - `logfile`

use libtest_lexarg::OutputFormat;

use crate::Case;
use crate::RunError;
use crate::RunErrorInner;
use crate::RunResult;

pub(crate) struct Outcomes {
    out: Box<dyn std::io::Write>,
    format: OutputFormat,
    name_width: usize,

    outcomes: std::collections::BTreeMap<String, RunResult>,
    total_elapsed: std::time::Instant,
    num_tests: usize,
    /// Number of tests and benchmarks that were filtered out (either by the
    /// filter-in pattern or by `--skip` arguments).
    num_filtered_out: usize,
    /// Number of passed tests.
    num_passed: usize,
    /// Number of failed tests and benchmarks.
    num_failed: usize,
    /// Number of ignored tests and benchmarks.
    num_ignored: usize,
}

impl Outcomes {
    /// Creates a new printer configured by the given arguments (`format`,
    /// `quiet`, `color` and `logfile` options).
    pub(crate) fn new(
        args: &libtest_lexarg::TestOpts,
        cases: &[Box<dyn Case>],
        num_filtered_out: usize,
    ) -> std::io::Result<Self> {
        // Determine target of all output
        let out: Box<dyn std::io::Write> = if let Some(logfile) = &args.logfile {
            let f = std::fs::File::create(logfile)?;
            if anstream::ColorChoice::global() == anstream::ColorChoice::Always {
                Box::new(f)
            } else {
                Box::new(anstream::StripStream::new(f))
            }
        } else {
            Box::new(anstream::stdout())
        };

        // Determine correct format
        let format = args.format;

        // Determine max test name length to do nice formatting later.
        //
        // Unicode is hard and there is no way we can properly align/pad the
        // test names and outcomes. Counting the number of code points is just
        // a cheap way that works in most cases. Usually, these names are
        // ASCII.
        let name_width = cases
            .iter()
            .map(|test| test.name().chars().count())
            .max()
            .unwrap_or(0);

        Ok(Self {
            out,
            format,
            name_width,
            outcomes: Default::default(),
            total_elapsed: std::time::Instant::now(),
            num_tests: cases.len(),
            num_filtered_out: num_filtered_out,
            num_passed: 0,
            num_failed: 0,
            num_ignored: 0,
        })
    }

    /// Prints the first line "running 3 tests".
    pub(crate) fn start_suite(&mut self) -> std::io::Result<()> {
        match self.format {
            OutputFormat::Pretty | OutputFormat::Terse => {
                let s = if self.num_tests == 1 { "" } else { "s" };

                writeln!(self.out)?;
                writeln!(self.out, "running {} test{s}", self.num_tests)?;
            }
            OutputFormat::Json | OutputFormat::Junit => todo!(),
        }

        Ok(())
    }

    /// Prints the text announcing the test (e.g. "test foo::bar ... "). Prints
    /// nothing in terse mode.
    pub(crate) fn start_case(&mut self, case: &dyn Case) -> std::io::Result<()> {
        match self.format {
            OutputFormat::Pretty => {
                write!(self.out, "test {: <1$} ... ", case.name(), self.name_width)?;
                self.out.flush()?;
            }
            OutputFormat::Terse => {
                // In terse mode, nothing is printed before the job. Only
                // `print_single_outcome` prints one character.
            }
            OutputFormat::Json | OutputFormat::Junit => todo!(),
        }

        Ok(())
    }

    /// Prints the outcome of a single tests. `ok` or `FAILED` in pretty mode
    /// and `.` or `F` in terse mode.
    pub(crate) fn finish_case(
        &mut self,
        case: &dyn Case,
        outcome: RunResult,
    ) -> std::io::Result<()> {
        match self.format {
            OutputFormat::Pretty => {
                let (s, style) = match &outcome {
                    Ok(()) => ("ok", OK),
                    Err(RunError(RunErrorInner::Failed(_))) => ("FAILED", FAILED),
                    Err(RunError(RunErrorInner::Ignored(_))) => ("ignored", IGNORED),
                };

                writeln!(self.out, "{}{s}{}", style.render(), style.render_reset())?;
            }
            OutputFormat::Terse => {
                let (c, style) = match outcome {
                    Ok(()) => ('.', OK),
                    Err(RunError(RunErrorInner::Failed(_))) => ('F', FAILED),
                    Err(RunError(RunErrorInner::Ignored(_))) => ('i', IGNORED),
                };

                write!(self.out, "{}{c}{}", style.render(), style.render_reset())?;
            }
            OutputFormat::Json | OutputFormat::Junit => todo!(),
        }

        match &outcome {
            Ok(()) => self.num_passed += 1,
            Err(RunError(RunErrorInner::Failed(_))) => self.num_failed += 1,
            Err(RunError(RunErrorInner::Ignored(_))) => self.num_ignored += 1,
        }
        self.outcomes.insert(case.name().to_owned(), outcome);

        Ok(())
    }

    /// Prints the summary line after all tests have been executed.
    pub(crate) fn finish_suite(&mut self) -> std::io::Result<()> {
        if self.has_failed() {
            writeln!(self.out)?;
            writeln!(self.out, "failures:")?;
            writeln!(self.out)?;

            // Print messages of all tests
            for (name, outcome) in &self.outcomes {
                if let Err(RunError(RunErrorInner::Failed(msg))) = outcome {
                    writeln!(self.out, "---- {} ----", name)?;
                    writeln!(self.out, "{}", msg)?;
                    writeln!(self.out)?;
                }
            }

            // Print summary list of failed tests
            writeln!(self.out)?;
            writeln!(self.out, "failures:")?;
            for (name, outcome) in &self.outcomes {
                if let Err(RunError(RunErrorInner::Failed(_))) = outcome {
                    writeln!(self.out, "    {}", name)?;
                }
            }
        }

        match self.format {
            OutputFormat::Pretty | OutputFormat::Terse => {
                let (summary, summary_style) = if self.has_failed() {
                    ("FAILED", FAILED)
                } else {
                    ("ok", OK)
                };
                let num_passed = self.num_passed;
                let num_failed = self.num_failed;
                let num_ignored = self.num_ignored;
                let num_filtered_out = self.num_filtered_out;
                let execution_time = self.total_elapsed.elapsed().as_secs_f64();

                writeln!(self.out)?;
                writeln!(
                    self.out,
                    "test result: {}{summary}{}. {num_passed} passed; {num_failed} failed; {num_ignored} ignored; \
                        {num_filtered_out} filtered out; finished in {execution_time:.2}s",
                    summary_style.render(),
                    summary_style.render_reset()
                )?;
                writeln!(self.out)?;
            }
            OutputFormat::Json | OutputFormat::Junit => todo!(),
        }
        Ok(())
    }

    /// Prints a list of all tests. Used if `--list` is set.
    pub(crate) fn list(&mut self, cases: &[Box<dyn Case>]) -> std::io::Result<()> {
        for case in cases {
            writeln!(self.out, "{}: test", case.name())?;
        }

        writeln!(self.out)?;
        writeln!(self.out, "{} tests", cases.len())?;
        writeln!(self.out)?;

        Ok(())
    }

    pub(crate) fn has_failed(&self) -> bool {
        0 < self.num_failed
    }
}

const FAILED: anstyle::Style =
    anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red)));
const OK: anstyle::Style =
    anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green)));
const IGNORED: anstyle::Style =
    anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow)));
