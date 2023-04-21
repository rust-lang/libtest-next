pub use crate::*;

pub trait Case {
    // The name of a test
    //
    // By convention this follows the rules for rust paths; i.e., it should be a series of
    // identifiers separated by double colons. This way if some test runner wants to arrange the
    // tests hierarchically it may.
    fn name(&self) -> &str;
    fn kind(&self) -> TestKind;
    fn source(&self) -> Option<&Source>;

    fn run(&self, state: &State) -> Result<(), RunError>;
}

pub struct SimpleCase {
    name: String,
    runner: Box<dyn Fn(&State) -> Result<(), RunError> + Send + Sync>,
    kind: TestKind,
    source: Option<Source>,
}

impl SimpleCase {
    pub fn test(
        name: impl Into<String>,
        runner: impl Fn(&State) -> Result<(), RunError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            runner: Box::new(runner),
            kind: Default::default(),
            source: None,
        }
    }

    pub fn kind(mut self, kind: TestKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn source(mut self, source: Source) -> Self {
        self.source = Some(source);
        self
    }
}

impl Case for SimpleCase {
    fn name(&self) -> &str {
        &self.name
    }
    fn kind(&self) -> TestKind {
        self.kind
    }
    fn source(&self) -> Option<&Source> {
        self.source.as_ref()
    }

    fn run(&self, state: &State) -> Result<(), RunError> {
        (self.runner)(state)
    }
}

/// Type of the test according to the [rust book](https://doc.rust-lang.org/cargo/guide/tests.html)
/// conventions.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TestKind {
    /// Unit-tests are expected to be in the `src` folder of the crate.
    UnitTest,
    /// Integration-style tests are expected to be in the `tests` folder of the crate.
    IntegrationTest,
    /// Doctests are created by the `librustdoc` manually, so it's a different type of test.
    DocTest,
    /// Tests for the sources that don't follow the project layout convention
    /// (e.g. tests in raw `main.rs` compiled by calling `rustc --test` directly).
    Unknown,
}

impl Default for TestKind {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Source {
    Rust {
        source_file: std::path::PathBuf,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    },
    Path(std::path::PathBuf),
}

pub type RunResult = Result<(), RunError>;

#[derive(Debug)]
pub struct RunError(pub(crate) RunErrorInner);

impl RunError {
    pub fn cause(cause: impl Into<Fail>) -> Self {
        Self(RunErrorInner::Failed(cause.into()))
    }

    pub fn msg(cause: impl std::fmt::Display) -> Self {
        Self::cause(FailMessage(cause.to_string()))
    }

    pub(crate) fn ignore() -> Self {
        Self(RunErrorInner::Ignored(Ignore { reason: None }))
    }

    pub(crate) fn ignore_for(reason: String) -> Self {
        Self(RunErrorInner::Ignored(Ignore {
            reason: Some(reason),
        }))
    }
}

impl<E> From<E> for RunError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        Self::cause(error)
    }
}

#[derive(Debug)]
pub(crate) enum RunErrorInner {
    Failed(Fail),
    Ignored(Ignore),
}

#[derive(Debug)]
pub struct Fail {
    inner: Box<dyn std::error::Error + Send + Sync + 'static>,
}

impl<E> From<E> for Fail
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[cold]
    fn from(error: E) -> Self {
        Fail {
            inner: Box::new(error),
        }
    }
}

impl std::fmt::Display for Fail {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(formatter)
    }
}

#[derive(Debug)]
pub struct FailMessage(String);

impl std::fmt::Display for FailMessage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for FailMessage {}

#[derive(Debug)]
pub struct Ignore {
    reason: Option<String>,
}

impl Ignore {
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}
