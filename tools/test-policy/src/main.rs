//! CLI adapter for source discovery, file reads, and policy diagnostics.

use restqs_test_policy::{Report, analyze};
use std::{
    error::Error,
    path::PathBuf,
    process::{Command, ExitCode},
};

/// Use explicit CLI paths or discover tracked and unignored Rust files with git; fail when
/// discovery cannot run.
fn source_paths() -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let arguments: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if !arguments.is_empty() {
        return Ok(arguments);
    }
    let output = Command::new("git")
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
            "--",
            "*.rs",
        ])
        .output()?;
    if !output.status.success() {
        return Err("git ls-files failed; run from the repository root".into());
    }
    paths_from_output(&output.stdout)
}

/// Decode NUL-separated UTF-8 paths, sort and deduplicate them, and reject an empty source
/// inventory.
fn paths_from_output(output: &[u8]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut paths: Vec<_> = std::str::from_utf8(output)?
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect();
    paths.sort();
    paths.dedup();
    if paths.is_empty() {
        return Err("no Rust sources found".into());
    }
    Ok(paths)
}

/// Read one Rust source file and delegate its syntax analysis without executing it.
fn read_report(path: &PathBuf) -> Result<Report, Box<dyn Error>> {
    Ok(analyze(&std::fs::read_to_string(path)?)?)
}

/// Emit source-positioned diagnostics to stderr for editors and CI logs.
fn print_report(path: &std::path::Path, report: &Report) {
    for diagnostic in &report.diagnostics {
        eprintln!(
            "{}:{}:{}: {}",
            path.display(),
            diagnostic.line,
            diagnostic.column,
            diagnostic.message
        );
    }
}

/// Coordinate discovery, analysis, diagnostics, and totals; return whether every source satisfies
/// the policy.
fn run() -> Result<bool, Box<dyn Error>> {
    let paths = source_paths()?;
    let mut tests = 0;
    let mut failures = 0;
    for path in &paths {
        let report = read_report(path).map_err(|error| format!("{}: {error}", path.display()))?;
        print_report(path, &report);
        tests += report.tests;
        failures += report.diagnostics.len();
    }
    println!(
        "Checked {tests} Rust tests in {} files; {failures} policy violations",
        paths.len()
    );
    Ok(failures == 0)
}

/// Map a successful policy check to exit zero and print infrastructure errors before returning
/// failure.
fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("test-policy: {error}");
            ExitCode::FAILURE
        }
    }
}
