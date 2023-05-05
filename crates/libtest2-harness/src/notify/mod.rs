#[cfg(feature = "json")]
mod json;
#[cfg(feature = "junit")]
mod junit;
mod pretty;
mod summary;
mod terse;

#[cfg(feature = "json")]
pub(crate) use json::*;
#[cfg(feature = "junit")]
pub(crate) use junit::*;
pub(crate) use pretty::*;
pub(crate) use summary::*;
pub(crate) use terse::*;

pub(crate) trait Notifier {
    fn threaded(&mut self, _yes: bool) {}

    fn notify(&mut self, event: Event) -> std::io::Result<()>;
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[cfg_attr(feature = "json", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "json", serde(tag = "event"))]
pub(crate) enum Event {
    DiscoverStart,
    DiscoverCase {
        name: String,
        mode: RunMode,
        run: bool,
    },
    DiscoverComplete {
        #[allow(dead_code)]
        elapsed_s: Elapsed,
        seed: Option<u64>,
    },
    SuiteStart,
    CaseStart {
        name: String,
    },
    CaseComplete {
        name: String,
        #[allow(dead_code)]
        mode: RunMode,
        status: Option<RunStatus>,
        message: Option<String>,
        #[allow(dead_code)]
        elapsed_s: Option<Elapsed>,
    },
    SuiteComplete {
        elapsed_s: Elapsed,
    },
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[cfg_attr(feature = "json", serde(rename_all = "kebab-case"))]
pub enum RunMode {
    #[default]
    Test,
    Bench,
}

impl RunMode {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Test => "test",
            Self::Bench => "bench",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[cfg_attr(feature = "json", serde(rename_all = "kebab-case"))]
pub(crate) enum RunStatus {
    Ignored,
    Failed,
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[cfg_attr(feature = "json", serde(into = "String"))]
pub(crate) struct Elapsed(pub std::time::Duration);

impl std::fmt::Display for Elapsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}s", self.0.as_secs_f64())
    }
}

impl From<Elapsed> for String {
    fn from(elapsed: Elapsed) -> Self {
        elapsed.0.as_secs_f64().to_string()
    }
}

const FAILED: anstyle::Style =
    anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red)));
const OK: anstyle::Style =
    anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green)));
const IGNORED: anstyle::Style =
    anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow)));
