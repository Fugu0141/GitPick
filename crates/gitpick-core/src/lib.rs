//! Platform-independent entry points for GitPick.
//!
//! Issue #2 only establishes a non-mutating scaffold. Repository inspection,
//! planning, and applying are intentionally not implemented here yet.

pub mod process;

use std::error::Error as StdError;
use std::fmt;
use std::path::PathBuf;

/// The requested rule operation. A rule update never implies an index update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleAction {
    Ignore,
    Unignore,
    Lfs,
    Unlfs,
}

/// A request shared by CLI and future UI integrations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Scan { paths: Vec<PathBuf> },
    Plan { action: RuleAction, path: PathBuf },
    Apply { plan_path: PathBuf },
}

#[derive(Debug)]
pub enum GitPickError {
    /// Work for this command has not yet been implemented. Never report success.
    NotImplemented(&'static str),
    Io(std::io::Error),
}

impl fmt::Display for GitPickError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented(feature) => {
                write!(f, "{feature} is not implemented yet; no changes were made")
            }
            Self::Io(error) => write!(f, "I/O operation failed: {error}"),
        }
    }
}

impl StdError for GitPickError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::NotImplemented(_) => None,
        }
    }
}

/// Routes requests through the core rather than handling Git in the CLI.
///
/// This deliberately fails closed until the appropriate feature Issue is
/// implemented. In particular, `Apply` does not modify files or the index.
pub fn execute(request: Request) -> Result<(), GitPickError> {
    let feature = match request {
        Request::Scan { .. } => "scan (Issue #4)",
        Request::Plan { .. } => "plan (Issues #5-#7)",
        Request::Apply { .. } => "apply (Issue #7)",
    };
    Err(GitPickError::NotImplemented(feature))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_is_explicitly_unimplemented() {
        let response = execute(Request::Apply {
            plan_path: PathBuf::from("not-a-real-plan.json"),
        });
        assert!(matches!(response, Err(GitPickError::NotImplemented(_))));
    }

    #[test]
    fn scan_is_explicitly_unimplemented() {
        let response = execute(Request::Scan {
            paths: vec![PathBuf::from("example")],
        });
        assert!(matches!(response, Err(GitPickError::NotImplemented(_))));
    }
}
