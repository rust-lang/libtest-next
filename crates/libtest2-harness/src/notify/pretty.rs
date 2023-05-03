use super::Event;
use super::RunStatus;
use super::FAILED;
use super::IGNORED;
use super::OK;

#[derive(Debug)]
pub(crate) struct PrettyRunNotifier<W> {
    writer: W,
    is_multithreaded: bool,
    summary: super::Summary,
    name_width: usize,
}

impl<W: std::io::Write> PrettyRunNotifier<W> {
    pub(crate) fn new(writer: W) -> Self {
        Self {
            writer,
            is_multithreaded: false,
            summary: Default::default(),
            name_width: 0,
        }
    }
}

impl<W: std::io::Write> super::Notifier for PrettyRunNotifier<W> {
    fn threaded(&mut self, yes: bool) {
        self.is_multithreaded = yes;
    }

    fn notify(&mut self, event: Event) -> std::io::Result<()> {
        self.summary.notify(event.clone())?;
        match event {
            Event::DiscoverStart => {}
            Event::DiscoverCase { name, run, .. } => {
                if run {
                    self.name_width = name.len().max(self.name_width);
                }
            }
            Event::DiscoverComplete { .. } => {}
            Event::SuiteStart => {
                self.summary.write_start(&mut self.writer)?;
            }
            Event::CaseStart { name, .. } => {
                if !self.is_multithreaded {
                    write!(self.writer, "test {: <1$} ... ", name, self.name_width)?;
                    self.writer.flush()?;
                }
            }
            Event::CaseComplete { name, status, .. } => {
                let (s, style) = match status {
                    Some(RunStatus::Ignored) => ("ignored", IGNORED),
                    Some(RunStatus::Failed) => ("FAILED", FAILED),
                    None => ("ok", OK),
                };

                if self.is_multithreaded {
                    write!(self.writer, "test {: <1$} ... ", name, self.name_width)?;
                }
                writeln!(self.writer, "{}{s}{}", style.render(), style.render_reset())?;
            }
            Event::SuiteComplete { .. } => {
                self.summary.write_complete(&mut self.writer)?;
            }
        }
        Ok(())
    }
}
