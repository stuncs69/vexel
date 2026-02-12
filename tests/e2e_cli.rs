mod common;

use common::{create_workspace, run_vexel, stderr_text, write_workspace_file};

#[test]
fn rejects_non_vx_input_files() {
    let workspace = create_workspace("bad_extension");
    let input_file = write_workspace_file(&workspace, "input.txt", "print 1\n");
    let arg = input_file.to_string_lossy().to_string();

    let output = run_vexel(&workspace, &[&arg]);
    assert!(!output.status.success());
    assert!(stderr_text(&output).contains("File must have '.vx' extension"));
}

#[test]
fn reports_parse_errors_with_non_zero_exit() {
    let workspace = create_workspace("parse_error");
    let script = write_workspace_file(&workspace, "main.vx", "set\n");
    let arg = script.to_string_lossy().to_string();

    let output = run_vexel(&workspace, &[&arg]);
    assert!(!output.status.success());
    assert!(stderr_text(&output).contains("Invalid set statement: set"));
}

#[test]
fn reports_runtime_errors_with_non_zero_exit() {
    let workspace = create_workspace("runtime_error");
    let script = write_workspace_file(&workspace, "main.vx", "print missing_value\n");
    let arg = script.to_string_lossy().to_string();

    let output = run_vexel(&workspace, &[&arg]);
    assert!(!output.status.success());
    assert!(stderr_text(&output).contains("Undefined variable 'missing_value'"));
}

#[test]
fn webcore_mode_without_routes_exits_successfully() {
    let workspace = create_workspace("webcore_empty");
    let arg = workspace.to_string_lossy().to_string();

    let output = run_vexel(&workspace, &["webcore", &arg]);
    assert!(output.status.success());
    assert!(stderr_text(&output).contains("No .vx endpoints found"));
}
