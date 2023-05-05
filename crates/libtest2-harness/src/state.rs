pub use crate::*;

#[derive(Debug)]
pub struct State {
    mode: notify::RunMode,
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

    pub fn current_mode(&self) -> notify::RunMode {
        self.mode
    }
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            mode: Default::default(),
            run_ignored: false,
        }
    }

    pub(crate) fn set_mode(&mut self, mode: notify::RunMode) {
        self.mode = mode;
    }

    pub(crate) fn set_run_ignored(&mut self, yes: bool) {
        self.run_ignored = yes;
    }
}
