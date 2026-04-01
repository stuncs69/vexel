mod common;

use common::{
    create_workspace, run_script, run_vexel, stderr_text, stdout_text, write_workspace_file,
};

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

#[test]
fn test_blocks_do_not_run_without_test_flag() {
    let workspace = create_workspace("tests_skipped");
    write_workspace_file(
        &workspace,
        "main.vx",
        r#"
print "script"
test "sample" start
    print "test"
end
"#,
    );

    let output = run_script(&workspace, "main.vx");
    assert!(output.status.success());
    assert_eq!(stdout_text(&output), "script\n");
}

#[test]
fn test_flag_runs_only_test_blocks() {
    let workspace = create_workspace("tests_only");
    let script = write_workspace_file(
        &workspace,
        "main.vx",
        r#"
print "script"

function helper() start
    return "from helper"
end

test "sample" start
    print helper()
end
"#,
    );
    let arg = script.to_string_lossy().to_string();

    let output = run_vexel(&workspace, &["--test", &arg]);
    assert!(
        output.status.success(),
        "stderr was: {}",
        stderr_text(&output)
    );
    assert_eq!(
        stdout_text(&output),
        "Running test: sample\nfrom helper\nTest 'sample' finished\n"
    );
}
