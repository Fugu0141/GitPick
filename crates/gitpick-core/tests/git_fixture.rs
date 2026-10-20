//! Disposable real-Git fixtures; these test the test harness, not yet #4 logic.

use gitpick_core::process::{ProcessRunner, SystemProcessRunner};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct GitFixture {
    root: PathBuf,
}

impl GitFixture {
    fn new() -> io::Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "gitpick-fixture-{}-{timestamp}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root)?;
        let fixture = Self { root };
        fixture.git_success(&["init", "-q"])?;
        Ok(fixture)
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn git(&self, args: &[&str]) -> io::Result<Output> {
        let arguments = args.iter().map(OsString::from).collect::<Vec<_>>();
        SystemProcessRunner.run(OsStr::new("git"), &arguments, &self.root)
    }

    fn git_success(&self, args: &[&str]) -> io::Result<Output> {
        let output = self.git(args)?;
        if !output.status.success() {
            return Err(io::Error::other(format!(
                "git failed (status {:?}): {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(output)
    }
}

impl Drop for GitFixture {
    fn drop(&mut self) {
        // Only the fresh, uniquely-named disposable fixture is deleted.
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn git_fixture_resolves_repository_and_preserves_special_names() -> io::Result<()> {
    let repo = GitFixture::new()?;
    assert!(repo.path().join(".git").is_dir());

    let filename = "space #[]! Unicode 日本語.txt";
    fs::write(repo.path().join(filename), b"fixture")?;
    repo.git_success(&["add", "--", filename])?;

    let output = repo.git_success(&["ls-files", "-z"])?;
    let expected = filename.as_bytes();
    assert!(
        output.stdout.split(|b| *b == 0).any(|name| name == expected),
        "git did not preserve the selected file name"
    );
    Ok(())
}

#[test]
fn git_fixture_checks_ignore_without_touching_file_content() -> io::Result<()> {
    let repo = GitFixture::new()?;
    fs::write(repo.path().join(".gitignore"), b"/generated/\n")?;
    fs::create_dir(repo.path().join("generated"))?;
    let path = repo.path().join("generated").join("output.bin");
    fs::write(&path, b"unchanged")?;

    let output = repo.git_success(&["check-ignore", "-q", "--", "generated/output.bin"])?;
    assert!(output.status.success());
    assert_eq!(fs::read(path)?, b"unchanged");
    Ok(())
}

#[test]
fn git_fixture_can_detect_a_non_repository() -> io::Result<()> {
    let repo = GitFixture::new()?;
    let outside = repo.path().parent().expect("fixture has parent");
    let output = std::process::Command::new("git")
        .args(["-C"])
        .arg(outside)
        .args(["rev-parse", "--show-toplevel"])
        // Stop discovery at the disposable fixture's parent, even if the
        // developer happens to use a temp directory below another repository.
        .env("GIT_CEILING_DIRECTORIES", outside)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()?;
    assert!(!output.status.success());
    Ok(())
}
