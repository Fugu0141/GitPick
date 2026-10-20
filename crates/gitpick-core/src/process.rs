//! Argument-vector process execution boundary for future Git operations.
//!
//! Callers must choose a trusted executable; user-selected paths belong in
//! separate arguments, not executable names or shell command strings.

use std::ffi::{OsStr, OsString};
use std::io;
use std::path::Path;
use std::process::{Command, Output};

/// Keeping stdout/stderr as bytes supports Git's NUL-delimited output.
pub trait ProcessRunner {
    fn run(&self, executable: &OsStr, args: &[OsString], current_dir: &Path) -> io::Result<Output>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemProcessRunner;

impl ProcessRunner for SystemProcessRunner {
    fn run(&self, executable: &OsStr, args: &[OsString], current_dir: &Path) -> io::Result<Output> {
        Command::new(executable)
            .args(args)
            .current_dir(current_dir)
            .output()
    }
}
