pub use crate::*;

#[derive(Debug)]
pub struct State {
    run_ignored: bool,
}

impl State {
    pub fn ignore(&self) -> Result<(), RunError> {
        if self.run_ignored {
            Ok(())
        } else {
            Err(RunError::ignore())
        }
    }

    pub fn ignore_for(&self, reason: impl std::fmt::Display) -> Result<(), RunError> {
        if self.run_ignored {
            Ok(())
        } else {
            Err(RunError::ignore_for(reason.to_string()))
        }
    }
}

impl State {
    pub(crate) fn new() -> Self {
        Self { run_ignored: false }
    }

    pub(crate) fn run_ignored(&mut self, yes: bool) {
        self.run_ignored = yes;
    }
}
