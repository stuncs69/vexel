#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn create_workspace(prefix: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    dir.push(format!("vexel_test_{}_{}_{}", prefix, std::process::id(), nanos));
    fs::create_dir_all(&dir).expect("failed to create temp workspace");
    dir
}

pub fn write_workspace_file(workspace: &Path, relative_path: &str, contents: &str) -> PathBuf {
    let path = workspace.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent directories");
    }
    fs::write(&path, contents).expect("failed to write workspace file");
    path
}

pub fn run_vexel(workspace: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vexel"))
        .current_dir(workspace)
        .args(args)
        .output()
        .expect("failed to execute vexel binary")
}

pub fn run_script(workspace: &Path, script_relative_path: &str) -> Output {
    let script_path = workspace.join(script_relative_path);
    let script_arg = script_path.to_string_lossy().to_string();
    run_vexel(workspace, &[&script_arg])
}

pub fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

pub fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n")
}

pub fn assert_stdout_lines(output: &Output, expected_lines: &[&str]) {
    let expected = format!("{}\n", expected_lines.join("\n"));
    assert_eq!(stdout_text(output), expected);
}
