#![forbid(unsafe_code)]

use diskhog::{human_size, scan, ScanOptions};
use std::env;
use std::path::PathBuf;
use std::process;

const DEFAULT_LIMIT: usize = 20;
const MAX_WARNINGS_TO_PRINT: usize = 10;

#[derive(Debug, PartialEq, Eq)]
struct Cli {
    path: PathBuf,
    limit: usize,
    include_files: bool,
    include_dirs: bool,
    max_depth: Option<usize>,
}

fn main() {
    let cli = match parse_args(env::args().skip(1)) {
        Ok(cli) => cli,
        Err(ParseOutcome::Help) => {
            print_help();
            return;
        }
        Err(ParseOutcome::Error(message)) => {
            eprintln!("diskhog: {message}");
            eprintln!("Try 'diskhog --help' for usage.");
            process::exit(2);
        }
    };

    let report = match scan(
        &cli.path,
        ScanOptions {
            include_files: cli.include_files,
            include_dirs: cli.include_dirs,
            max_depth: cli.max_depth,
        },
    ) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("diskhog: cannot scan '{}': {error}", cli.path.display());
            process::exit(1);
        }
    };

    println!("{:>10}  {:<4}  PATH", "SIZE", "TYPE");
    println!("{:>10}  {:<4}  ----", "----", "----");

    let shown = report.entries.len().min(cli.limit);
    for entry in report.entries.iter().take(cli.limit) {
        println!(
            "{:>10}  {:<4}  {}",
            human_size(entry.size),
            entry.kind.label(),
            entry.path.display()
        );
    }

    if shown == 0 {
        println!("No matching files or directories found.");
    } else if report.entries.len() > shown {
        println!(
            "\nShowing {shown} of {} matching entries.",
            report.entries.len()
        );
    }

    if !report.issues.is_empty() {
        eprintln!(
            "\nWarning: {} path(s) could not be fully read:",
            report.issues.len()
        );
        for issue in report.issues.iter().take(MAX_WARNINGS_TO_PRINT) {
            eprintln!("  {}: {}", issue.path.display(), issue.message);
        }
        if report.issues.len() > MAX_WARNINGS_TO_PRINT {
            eprintln!(
                "  ... and {} more",
                report.issues.len() - MAX_WARNINGS_TO_PRINT
            );
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParseOutcome {
    Help,
    Error(String),
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Cli, ParseOutcome> {
    let mut path = None;
    let mut limit = DEFAULT_LIMIT;
    let mut max_depth = None;
    let mut files_flag = false;
    let mut dirs_flag = false;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Err(ParseOutcome::Help),
            "--files" => files_flag = true,
            "--dirs" => dirs_flag = true,
            "--limit" => {
                let value = args.next().ok_or_else(|| {
                    ParseOutcome::Error("--limit requires a positive integer".into())
                })?;
                limit = parse_positive_usize("--limit", &value)?;
            }
            "--depth" => {
                let value = args.next().ok_or_else(|| {
                    ParseOutcome::Error("--depth requires a non-negative integer".into())
                })?;
                max_depth = Some(parse_non_negative_usize("--depth", &value)?);
            }
            value if value.starts_with('-') => {
                return Err(ParseOutcome::Error(format!("unknown option '{value}'")));
            }
            value => {
                if path.replace(PathBuf::from(value)).is_some() {
                    return Err(ParseOutcome::Error(
                        "only one path may be scanned at a time".into(),
                    ));
                }
            }
        }
    }

    let (include_files, include_dirs) = match (files_flag, dirs_flag) {
        (false, false) | (true, true) => (true, true),
        (true, false) => (true, false),
        (false, true) => (false, true),
    };

    Ok(Cli {
        path: path.unwrap_or_else(|| PathBuf::from(".")),
        limit,
        include_files,
        include_dirs,
        max_depth,
    })
}

fn parse_positive_usize(option: &str, value: &str) -> Result<usize, ParseOutcome> {
    let parsed = parse_non_negative_usize(option, value)?;
    if parsed == 0 {
        return Err(ParseOutcome::Error(format!(
            "{option} must be greater than zero"
        )));
    }
    Ok(parsed)
}

fn parse_non_negative_usize(option: &str, value: &str) -> Result<usize, ParseOutcome> {
    value.parse::<usize>().map_err(|_| {
        ParseOutcome::Error(format!(
            "{option} expects a non-negative integer, got '{value}'"
        ))
    })
}

fn print_help() {
    println!(
        "DiskHog {version}\n\
Fast, read-only disk usage analysis.\n\n\
USAGE:\n    diskhog [OPTIONS] [PATH]\n\n\
ARGS:\n    [PATH]    Directory or file to scan [default: .]\n\n\
OPTIONS:\n    --limit N    Show at most N entries [default: 20]\n    --files      Show files only (combine with --dirs to show both)\n    --dirs       Show directories only (combine with --files to show both)\n    --depth N    Only display entries up to depth N; directory totals stay recursive\n    -h, --help   Print help\n\n\
DiskHog never deletes files and never follows symbolic links.",
        version = env!("CARGO_PKG_VERSION")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Cli, ParseOutcome> {
        parse_args(args.iter().map(|value| (*value).to_string()))
    }

    #[test]
    fn defaults_match_v0_1_0_contract() {
        let cli = parse(&["."]).unwrap();
        assert_eq!(cli.limit, 20);
        assert!(cli.include_files);
        assert!(cli.include_dirs);
        assert_eq!(cli.max_depth, None);
    }

    #[test]
    fn files_filter_is_exclusive_when_used_alone() {
        let cli = parse(&["--files", "."]).unwrap();
        assert!(cli.include_files);
        assert!(!cli.include_dirs);
    }

    #[test]
    fn dirs_filter_is_exclusive_when_used_alone() {
        let cli = parse(&["--dirs", "."]).unwrap();
        assert!(!cli.include_files);
        assert!(cli.include_dirs);
    }

    #[test]
    fn both_filters_restore_combined_view() {
        let cli = parse(&["--files", "--dirs", "."]).unwrap();
        assert!(cli.include_files);
        assert!(cli.include_dirs);
    }

    #[test]
    fn limit_and_depth_are_parsed() {
        let cli = parse(&["--limit", "50", "--depth", "3", "."]).unwrap();
        assert_eq!(cli.limit, 50);
        assert_eq!(cli.max_depth, Some(3));
    }

    #[test]
    fn zero_limit_is_rejected() {
        assert!(matches!(
            parse(&["--limit", "0", "."]),
            Err(ParseOutcome::Error(_))
        ));
    }
}
