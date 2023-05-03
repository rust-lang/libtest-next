use super::Event;
use super::RunStatus;
use super::FAILED;
use super::OK;

#[derive(Default, Clone, Debug)]
pub(crate) struct Summary {
    pub(crate) seed: Option<u64>,
    pub(crate) failures: std::collections::BTreeMap<String, Option<String>>,
    pub(crate) elapsed_s: super::Elapsed,

    pub(crate) num_run: usize,
    /// Number of tests and benchmarks that were filtered out (either by the
    /// filter-in pattern or by `--skip` arguments).
    pub(crate) num_filtered_out: usize,

    /// Number of passed tests.
    pub(crate) num_passed: usize,
    /// Number of failed tests and benchmarks.
    pub(crate) num_failed: usize,
    /// Number of ignored tests and benchmarks.
    pub(crate) num_ignored: usize,
}

impl Summary {
    pub(crate) fn has_failed(&self) -> bool {
        0 < self.num_failed
    }

    pub(crate) fn write_start(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        let s = if self.num_run == 1 { "" } else { "s" };
        let seed = self
            .seed
            .map(|s| format!(" (shuffle seed: {s})"))
            .unwrap_or_default();

        writeln!(writer)?;
        writeln!(writer, "running {} test{s}{seed}", self.num_run)?;
        Ok(())
    }

    pub(crate) fn write_complete(&self, writer: &mut dyn ::std::io::Write) -> std::io::Result<()> {
        let (summary, summary_style) = if self.has_failed() {
            ("FAILED", FAILED)
        } else {
            ("ok", OK)
        };
        let num_passed = self.num_passed;
        let num_failed = self.num_failed;
        let num_ignored = self.num_ignored;
        let num_filtered_out = self.num_filtered_out;
        let elapsed_s = self.elapsed_s;

        if self.has_failed() {
            writeln!(writer)?;
            writeln!(writer, "failures:")?;
            writeln!(writer)?;

            // Print messages of all tests
            for (name, msg) in &self.failures {
                if let Some(msg) = msg {
                    writeln!(writer, "---- {} ----", name)?;
                    writeln!(writer, "{}", msg)?;
                    writeln!(writer)?;
                }
            }

            // Print summary list of failed tests
            writeln!(writer)?;
            writeln!(writer, "failures:")?;
            for (name, _) in &self.failures {
                writeln!(writer, "    {}", name)?;
            }
        }
        writeln!(writer)?;
        writeln!(
                    writer,
                    "test result: {}{summary}{}. {num_passed} passed; {num_failed} failed; {num_ignored} ignored; \
                        {num_filtered_out} filtered out; finished in {elapsed_s}",
                    summary_style.render(),
                    summary_style.render_reset()
                )?;
        writeln!(writer)?;

        Ok(())
    }
}

impl super::Notifier for Summary {
    fn notify(&mut self, event: Event) -> std::io::Result<()> {
        match event {
            Event::DiscoverStart => {}
            Event::DiscoverCase { run, .. } => {
                if run {
                    self.num_run += 1;
                } else {
                    self.num_filtered_out += 1;
                }
            }
            Event::DiscoverComplete { seed, .. } => {
                self.seed = seed;
            }
            Event::SuiteStart => {}
            Event::CaseStart { .. } => {}
            Event::CaseComplete {
                name,
                status,
                message,
                ..
            } => match status {
                Some(RunStatus::Ignored) => {
                    self.num_ignored += 1;
                }
                Some(RunStatus::Failed) => {
                    self.num_failed += 1;
                    self.failures.insert(name, message);
                }
                None => {
                    self.num_passed += 1;
                }
            },
            Event::SuiteComplete { elapsed_s, .. } => {
                self.elapsed_s = elapsed_s;
            }
        }
        Ok(())
    }
}
