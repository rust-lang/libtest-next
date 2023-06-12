//! An experimental replacement for libtest
//!
//! # Usage
//!
//! To use this, you most likely want to add a manual `[[test]]` section to
//! `Cargo.toml` and set `harness = false`. For example:
//!
//! ```toml
//! [[test]]
//! name = "mytest"
//! path = "tests/mytest.rs"
//! harness = false
//! ```
//!
//! And in `tests/mytest.rs` you would call [`Harness::main`] in the `main` function:
//!
//! ```no_run
//! libtest2::Harness::with_env()
//!     .main();
//! ```
//!

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub use libtest2_harness::Harness;
pub use libtest2_harness::RunError;
pub use libtest2_harness::RunResult;
pub use libtest2_harness::State;
pub use libtest2_harness::TestKind;

use libtest2_harness::Case;
use libtest2_harness::Source;

pub struct Trial {
    name: String,
    #[allow(clippy::type_complexity)]
    runner: Box<dyn Fn(&State) -> Result<(), RunError> + Send + Sync>,
}

impl Trial {
    pub fn test(
        name: impl Into<String>,
        runner: impl Fn(&State) -> Result<(), RunError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            runner: Box::new(runner),
        }
    }
}

impl Case for Trial {
    fn name(&self) -> &str {
        &self.name
    }
    fn kind(&self) -> TestKind {
        Default::default()
    }
    fn source(&self) -> Option<&Source> {
        None
    }
    fn exclusive(&self, _: &State) -> bool {
        false
    }

    fn run(&self, state: &State) -> Result<(), RunError> {
        (self.runner)(state)
    }
}
