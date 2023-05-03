use super::Event;
use super::RunStatus;

#[derive(Debug)]
pub(crate) struct JunitRunNotifier<W> {
    writer: W,
    events: Vec<Event>,
}

impl<W: std::io::Write> JunitRunNotifier<W> {
    pub(crate) fn new(writer: W) -> Self {
        Self {
            writer,
            events: Vec::new(),
        }
    }
}

impl<W: std::io::Write> super::Notifier for JunitRunNotifier<W> {
    fn notify(&mut self, event: Event) -> std::io::Result<()> {
        let finished = matches!(&event, Event::SuiteComplete { .. });
        self.events.push(event);
        if finished {
            let mut num_run = 0;
            let mut num_failed = 0;
            let mut num_ignored = 0;
            for event in &self.events {
                match event {
                    Event::DiscoverStart => {}
                    Event::DiscoverCase { run, .. } => {
                        if *run {
                            num_run += 1;
                        }
                    }
                    Event::DiscoverComplete { .. } => {}
                    Event::SuiteStart => {}
                    Event::CaseStart { .. } => {}
                    Event::CaseComplete { status, .. } => match status {
                        Some(RunStatus::Ignored) => {
                            num_ignored += 1;
                        }
                        Some(RunStatus::Failed) => {
                            num_failed += 1;
                        }
                        None => {}
                    },
                    Event::SuiteComplete { .. } => {}
                }
            }

            writeln!(self.writer, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
            writeln!(self.writer, "<testsuites>")?;

            writeln!(
                self.writer,
                "<testsuite name=\"test\" package=\"test\" id=\"0\" \
             tests=\"{num_run}\" \
             errors=\"0\" \
             failures=\"{num_failed}\" \
             skipped=\"{num_ignored}\" \
             >"
            )?;
            for event in std::mem::take(&mut self.events) {
                if let Event::CaseComplete {
                    name,
                    status,
                    message,
                    elapsed_s,
                    ..
                } = event
                {
                    let (class_name, test_name) = parse_class_name(&name);
                    let elapsed_s = elapsed_s.unwrap_or_default();
                    match status {
                        Some(RunStatus::Ignored) => {}
                        Some(RunStatus::Failed) => {
                            writeln!(
                                self.writer,
                                "<testcase classname=\"{class_name}\" \
                         name=\"{test_name}\" time=\"{elapsed_s}\">",
                            )?;
                            if let Some(message) = message {
                                writeln!(
                                    self.writer,
                                    "<failure message=\"{message}\" type=\"assert\"/>"
                                )?;
                            } else {
                                writeln!(self.writer, "<failure type=\"assert\"/>")?;
                            }
                            writeln!(self.writer, "</testcase>")?;
                        }
                        None => {
                            writeln!(
                                self.writer,
                                "<testcase classname=\"{class_name}\" \
                         name=\"{test_name}\" time=\"{elapsed_s}\"/>",
                            )?;
                        }
                    }
                }
            }
            writeln!(self.writer, "<system-out/>")?;
            writeln!(self.writer, "<system-err/>")?;
            writeln!(self.writer, "</testsuite>")?;
            writeln!(self.writer, "</testsuites>")?;
        }
        Ok(())
    }
}

fn parse_class_name(name: &str) -> (String, String) {
    // Module path => classname
    // Function name => name
    let module_segments: Vec<&str> = name.split("::").collect();
    let (class_name, test_name) = match module_segments[..] {
        [test] => (String::from("crate"), String::from(test)),
        [ref path @ .., test] => (path.join("::"), String::from(test)),
        [..] => unreachable!(),
    };
    (class_name, test_name)
}
