use super::CaseMode;
use super::Event;
use super::RunStatus;
use super::FAILED;
use super::IGNORED;
use super::OK;

#[derive(Debug)]
pub(crate) struct TerseListNotifier<W> {
    writer: W,
    tests: usize,
}

impl<W: std::io::Write> TerseListNotifier<W> {
    pub(crate) fn new(writer: W) -> Self {
        Self { writer, tests: 0 }
    }
}

impl<W: std::io::Write> super::Notifier for TerseListNotifier<W> {
    fn notify(&mut self, event: Event) -> std::io::Result<()> {
        match event {
            Event::DiscoverStart => {}
            Event::DiscoverCase { name, mode, run } => {
                if run {
                    let mode = match mode {
                        CaseMode::Test => "test",
                        CaseMode::Bench => "bench",
                    };
                    writeln!(self.writer, "{name}: {mode}")?;
                    self.tests += 1;
                }
            }
            Event::DiscoverComplete { .. } => {
                writeln!(self.writer)?;
                writeln!(self.writer, "{} tests", self.tests)?;
                writeln!(self.writer)?;
            }
            Event::SuiteStart => {}
            Event::CaseStart { .. } => {}
            Event::CaseComplete { .. } => {}
            Event::SuiteComplete { .. } => {}
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct TerseRunNotifier<W> {
    writer: W,
    summary: super::Summary,
}

impl<W: std::io::Write> TerseRunNotifier<W> {
    pub(crate) fn new(writer: W) -> Self {
        Self {
            writer,
            summary: Default::default(),
        }
    }
}

impl<W: std::io::Write> super::Notifier for TerseRunNotifier<W> {
    fn notify(&mut self, event: Event) -> std::io::Result<()> {
        self.summary.notify(event.clone())?;
        match event {
            Event::DiscoverStart => {}
            Event::DiscoverCase { .. } => {}
            Event::DiscoverComplete { .. } => {}
            Event::SuiteStart => {
                self.summary.write_start(&mut self.writer)?;
            }
            Event::CaseStart { .. } => {}
            Event::CaseComplete { status, .. } => {
                let (c, style) = match status {
                    Some(RunStatus::Ignored) => ('i', IGNORED),
                    Some(RunStatus::Failed) => ('F', FAILED),
                    None => ('.', OK),
                };
                write!(self.writer, "{}{c}{}", style.render(), style.render_reset())?;
                self.writer.flush()?;
            }
            Event::SuiteComplete { .. } => {
                self.summary.write_complete(&mut self.writer)?;
            }
        }
        Ok(())
    }
}
