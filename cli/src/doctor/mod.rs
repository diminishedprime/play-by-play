//! `doctor`: checks that play-by-play is set up to work.

mod extension;
mod log;
mod path;
mod plugin;
mod tmux;
mod version;

use std::env;
use std::fmt::{self, Display, Formatter};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};
use crate::log::{BOLD, Color, GRAY, NORMAL, RESET};

use self::extension::Extension;
use self::log::LogFolder;
use self::path::OnPath;
use self::plugin::{Listed, Plugin};
use self::tmux::Tmux;
use self::version::Versions;

trait Check {
    const LABEL: &str;
    type Problem<'a>;

    fn assessments(facts: &Facts) -> impl Iterator<Item = Assessment<'_>>;

    fn suggested_fix<'a>(problem: Self::Problem<'a>) -> Fix<'a>;

    fn passed<'a>(text: impl Into<String>) -> Assessment<'a> {
        Assessment::new(Outcome::Passed, Self::LABEL, text, None)
    }

    fn skipped<'a>(text: impl Into<String>) -> Assessment<'a> {
        Assessment::new(Outcome::Skipped, Self::LABEL, text, None)
    }

    fn failed<'a>(text: impl Into<String>, problem: Self::Problem<'a>) -> Assessment<'a> {
        let fix = Self::suggested_fix(problem);
        Assessment::new(Outcome::Failed, Self::LABEL, text, Some(fix))
    }

    fn warned<'a>(text: impl Into<String>, problem: Self::Problem<'a>) -> Assessment<'a> {
        let fix = Self::suggested_fix(problem);
        Assessment::new(Outcome::Warning, Self::LABEL, text, Some(fix))
    }
}

enum Fix<'a> {
    Text(&'static str),
    AddToPath(&'a Path),
    MakeWritable(&'a Path),
}

impl Display for Fix<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Fix::Text(text) => f.write_str(text),
            Fix::AddToPath(dir) => write!(f, "add {} to your login shell's PATH", dir.display()),
            Fix::MakeWritable(dir) => write!(f, "run `chmod u+w {}`", dir.display()),
        }
    }
}

struct Facts {
    inside_tmux: bool,
    cargo_bin: Option<PathBuf>,
    log_folder: PathBuf,
    writable: bool,
    hyperlinks: Option<bool>,
    plugin: Option<Option<Listed>>,
    extension: Option<Option<String>>,
}

impl Facts {
    fn gather(dir: &Path) -> AppResult<Facts> {
        let inside_tmux = env::var_os("TMUX").is_some();
        let cargo_home = env::var_os("CARGO_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")));
        Ok(Facts {
            inside_tmux,
            cargo_bin: cargo_home.map(|cargo_home| cargo_home.join("bin")),
            log_folder: dir.to_path_buf(),
            writable: LogFolder::writable(dir).is_ok(),
            hyperlinks: (inside_tmux && OnPath::found("tmux"))
                .then(Tmux::has_hyperlinks)
                .transpose()?,
            plugin: OnPath::found("claude").then(Plugin::listed).transpose()?,
            extension: OnPath::found("code")
                .then(Extension::listed_version)
                .transpose()?,
        })
    }

    fn plugin_version(&self) -> Option<&str> {
        let plugin = self.plugin.as_ref()?.as_ref()?;
        Some(&plugin.version)
    }

    fn extension_version(&self) -> Option<&str> {
        self.extension.as_ref()?.as_deref()
    }
}

enum Outcome {
    Passed,
    Failed,
    Warning,
    Skipped,
}

impl Display for Outcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Passed => write!(f, "✅  {}", Color::Green),
            Outcome::Failed => write!(f, "❌  {}", Color::Red),
            Outcome::Warning => write!(f, "⚠️  {}", Color::Yellow),
            Outcome::Skipped => write!(f, "⏹️  {GRAY}"),
        }
    }
}

struct Assessment<'a> {
    outcome: Outcome,
    label: &'static str,
    text: String,
    fix: Option<Fix<'a>>,
}

impl<'a> Assessment<'a> {
    fn new(
        outcome: Outcome,
        label: &'static str,
        text: impl Into<String>,
        fix: Option<Fix<'a>>,
    ) -> Assessment<'a> {
        Assessment {
            outcome,
            label,
            text: text.into(),
            fix,
        }
    }

    fn failed(&self) -> bool {
        matches!(self.outcome, Outcome::Failed)
    }
}

impl Display for Assessment<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Assessment {
            outcome,
            label,
            text,
            fix,
        } = self;
        write!(f, "  {outcome}{BOLD}{label}{NORMAL} {text}")?;
        if let Some(fix) = fix {
            write!(f, "\n      Fix: {fix}")?;
        }
        writeln!(f, "{RESET}")
    }
}

/// Prints each check's outcome, leaving out the checks whose program isn't on
/// the `PATH`.
///
/// # Errors
///
/// When a check fails, or `claude`, `code` or `tmux` can't be run.
pub fn doctor(dir: &Path) -> AppResult {
    let facts = Facts::gather(dir)?;
    let mut out = BufWriter::new(io::stdout().lock());
    writeln!(out, "Checking play-by-play's setup")?;
    let healthy = OnPath::assessments(&facts)
        .chain(LogFolder::assessments(&facts))
        .chain(Tmux::assessments(&facts))
        .chain(Plugin::assessments(&facts))
        .chain(Extension::assessments(&facts))
        .chain(Versions::assessments(&facts))
        .try_fold(true, |healthy, line| {
            write!(out, "{line}")?;
            Ok::<_, AppError>(healthy && !line.failed())
        })?;
    out.flush()?;
    if healthy {
        Ok(())
    } else {
        Err(AppError::Unhealthy)
    }
}
