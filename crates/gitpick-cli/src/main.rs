use gitpick_core::{GitPickError, Request, RuleAction};
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "GitPick (development scaffold)\n\
Usage:\n\
  gitpick scan <path> [path ...]\n\
  gitpick plan <ignore|unignore|lfs|unlfs> <path>\n\
  gitpick apply <plan-file>\n\
  gitpick --help\n\
\n\
scan / plan / apply are intentionally unavailable until their Issues are implemented.\n\
All commands currently leave files, the Git index, and history untouched.";

#[derive(Debug, PartialEq, Eq)]
enum CliCommand {
    Help,
    Version,
    Request(Request),
}

fn parse_args(args: Vec<OsString>) -> Result<CliCommand, String> {
    let Some(command) = args.first().and_then(|arg| arg.to_str()) else {
        return Err("missing command (or command contains invalid Unicode)".to_string());
    };

    match command {
        "--help" | "-h" | "help" if args.len() == 1 => Ok(CliCommand::Help),
        "--version" | "-V" if args.len() == 1 => Ok(CliCommand::Version),
        "scan" => {
            if args.len() < 2 {
                return Err("scan requires at least one path".to_string());
            }
            Ok(CliCommand::Request(Request::Scan {
                paths: args[1..].iter().map(PathBuf::from).collect(),
            }))
        }
        "plan" => {
            if args.len() != 3 {
                return Err("plan requires an action and exactly one path".to_string());
            }
            let action = match args[1].to_str() {
                Some("ignore") => RuleAction::Ignore,
                Some("unignore") => RuleAction::Unignore,
                Some("lfs") => RuleAction::Lfs,
                Some("unlfs") => RuleAction::Unlfs,
                _ => return Err("unknown plan action".to_string()),
            };
            Ok(CliCommand::Request(Request::Plan {
                action,
                path: PathBuf::from(&args[2]),
            }))
        }
        "apply" => {
            if args.len() != 2 {
                return Err("apply requires exactly one plan-file path".to_string());
            }
            Ok(CliCommand::Request(Request::Apply {
                plan_path: PathBuf::from(&args[1]),
            }))
        }
        _ => Err("unknown command".to_string()),
    }
}

fn main() -> ExitCode {
    let args = env::args_os().skip(1).collect();
    match parse_args(args) {
        Ok(CliCommand::Help) => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Ok(CliCommand::Version) => {
            println!("gitpick {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(CliCommand::Request(request)) => match gitpick_core::execute(request) {
            Ok(()) => ExitCode::SUCCESS,
            Err(GitPickError::NotImplemented(feature)) => {
                eprintln!("error: {feature} is not implemented; no changes were made");
                ExitCode::from(3)
            }
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::from(1)
            }
        },
        Err(error) => {
            eprintln!("error: {error}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scan_paths_with_spaces() {
        let args = vec![
            OsString::from("scan"),
            OsString::from("folder with spaces/file.txt"),
        ];
        assert!(matches!(
            parse_args(args),
            Ok(CliCommand::Request(Request::Scan { paths })) if paths.len() == 1
        ));
    }

    #[test]
    fn requires_explicit_apply_plan_path() {
        assert!(parse_args(vec![OsString::from("apply")]).is_err());
    }

    #[test]
    fn rejects_unknown_plan_action() {
        let args = vec![
            OsString::from("plan"),
            OsString::from("delete"),
            OsString::from("file.txt"),
        ];
        assert!(parse_args(args).is_err());
    }
}
