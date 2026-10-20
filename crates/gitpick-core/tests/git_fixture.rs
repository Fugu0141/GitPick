//! Real-Git fixtures in disposable directories. These validate the harness,
//! not GitPick's repository inspection logic (Issue #4).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// Prevent inherited Git configuration from redirecting fixture operations to
/// the developer's real repository or index. This must run *after* any command
/// environment overrides and immediately before spawn.
fn isolate_git_environment(command: &mut Command, fixture_root: &Path) {
    for (name, _) in std::env::vars_os() {
        if name
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("GIT_")
        {
            command.env_remove(name);
        }
    }

    // A command may already have explicit overrides (e.g. a test scenario).
    // Command::env_remove removes these as well as inherited values.
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_CONFIG_COUNT",
        "GIT_CONFIG_KEY_0",
        "GIT_CONFIG_VALUE_0",
        "GIT_CONFIG_PARAMETERS",
        "GIT_CONFIG",
        "GIT_CONFIG_GLOBAL",
        "GIT_CONFIG_SYSTEM",
        "GIT_CEILING_DIRECTORIES",
        "GIT_TEMPLATE_DIR",
    ] {
        command.env_remove(key);
    }

    // Git may otherwise inherit a user's global/system aliases, hooks, or
    // filters. The designated global config file intentionally does not exist.
    command.env(
        "GIT_CONFIG_GLOBAL",
        fixture_root.join(".gitpick-no-global-config"),
    );
    command.env("GIT_CONFIG_NOSYSTEM", "1");
}

/// Only fresh, disposable fixture roots may be deleted by Drop.
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
        let mut command = Command::new("git");
        command.current_dir(&self.root).args(args);
        isolate_git_environment(&mut command, &self.root);
        command.output()
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
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Fixture F01: real Git preserves unusual names in -z output.
#[test]
fn git_fixture_resolves_repository_and_preserves_special_names() -> io::Result<()> {
    let repo = GitFixture::new()?;
    assert!(repo.path().join(".git").is_dir());

    let filename = "space #[]! Unicode 日本語.txt";
    fs::write(repo.path().join(filename), b"fixture")?;
    repo.git_success(&["add", "--", filename])?;

    let output = repo.git_success(&["ls-files", "-z"])?;
    assert!(
        output
            .stdout
            .split(|b| *b == 0)
            .any(|name| name == filename.as_bytes()),
        "Git did not preserve the selected file name"
    );
    Ok(())
}

/// Fixture F02: Git interprets ignore rules without deleting contents.
#[test]
fn git_fixture_checks_ignore_without_touching_file_content() -> io::Result<()> {
    let repo = GitFixture::new()?;
    fs::write(repo.path().join(".gitignore"), b"/generated/\n")?;
    fs::create_dir(repo.path().join("generated"))?;
    let path = repo.path().join("generated").join("output.bin");
    fs::write(&path, b"unchanged")?;

    repo.git_success(&["check-ignore", "-q", "--", "generated/output.bin"])?;
    assert_eq!(fs::read(path)?, b"unchanged");
    Ok(())
}

/// Fixture F03: repo discovery must fail outside a Git repository.
#[test]
fn git_fixture_can_detect_a_non_repository() -> io::Result<()> {
    let repo = GitFixture::new()?;
    let outside = repo.path().parent().expect("fixture has parent");
    let mut command = Command::new("git");
    command
        .current_dir(outside)
        .args(["rev-parse", "--show-toplevel"]);
    isolate_git_environment(&mut command, repo.path());
    // Prevent walking up above the temp directory into an unrelated repo.
    command.env("GIT_CEILING_DIRECTORIES", outside);
    assert!(!command.output()?.status.success());
    Ok(())
}

/// Fixture F04: inherited or explicit Git overrides cannot redirect an add.
#[test]
fn git_fixture_isolated_from_injected_git_environment() -> io::Result<()> {
    let intended = GitFixture::new()?;
    let victim = GitFixture::new()?;
    fs::write(intended.path().join("sentinel.txt"), b"intended")?;
    fs::write(victim.path().join("sentinel.txt"), b"victim")?;

    let mut command = Command::new("git");
    command
        .current_dir(intended.path())
        .args(["add", "--", "sentinel.txt"])
        .env("GIT_DIR", victim.path().join(".git"))
        .env("GIT_WORK_TREE", victim.path())
        .env("GIT_COMMON_DIR", victim.path().join(".git"))
        .env("GIT_INDEX_FILE", victim.path().join(".git/index"))
        .env("GIT_OBJECT_DIRECTORY", victim.path().join(".git/objects"))
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "core.bare")
        .env("GIT_CONFIG_VALUE_0", "true")
        .env("GIT_CONFIG_GLOBAL", victim.path().join("global.gitconfig"));
    isolate_git_environment(&mut command, intended.path());

    let output = command.output()?;
    assert!(
        output.status.success(),
        "isolated git add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        intended.git_success(&["ls-files", "-z"])?.stdout,
        b"sentinel.txt\0"
    );
    assert!(victim.git_success(&["ls-files", "-z"])?.stdout.is_empty());
    assert_eq!(fs::read(victim.path().join("sentinel.txt"))?, b"victim");
    Ok(())
}

/// Fixture F05: optional Git LFS smoke check; no LFS requirement for core.
#[test]
fn git_fixture_lfs_tracks_selected_file_when_available() -> io::Result<()> {
    let repo = GitFixture::new()?;
    let version = repo.git(&["lfs", "version"])?;
    if !version.status.success() {
        eprintln!("git-lfs is not installed; skipping optional fixture F05");
        return Ok(());
    }

    repo.git_success(&["lfs", "track", "--filename", "payload.bin"])?;
    let output = repo.git_success(&["check-attr", "filter", "--", "payload.bin"])?;
    assert_eq!(output.stdout, b"payload.bin: filter: lfs\n");
    Ok(())
}
