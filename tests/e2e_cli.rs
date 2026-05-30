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
    let stderr = stderr_text(&output);
    assert!(stderr.contains("line 1: Invalid set statement: set"));
}

#[test]
fn rejects_unknown_statements_with_line_number() {
    let workspace = create_workspace("unknown_statement");
    let script = write_workspace_file(&workspace, "main.vx", "print 1\nnonsense statement\n");
    let arg = script.to_string_lossy().to_string();

    let output = run_vexel(&workspace, &[&arg]);
    assert!(!output.status.success());
    assert!(stderr_text(&output).contains("line 2: Unknown statement: nonsense statement"));
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
fn reports_runtime_errors_for_invalid_bracket_property_access() {
    let workspace = create_workspace("runtime_error_bracket_access");
    let script = write_workspace_file(&workspace, "main.vx", "set x 123\nprint x[\"nope\"]\n");
    let arg = script.to_string_lossy().to_string();

    let output = run_vexel(&workspace, &[&arg]);
    assert!(!output.status.success());
    assert!(
        stderr_text(&output).contains("Property access target must evaluate to an object or array")
    );
}

#[test]
fn rejects_calling_unexported_module_functions() {
    let workspace = create_workspace("private_module_function");
    write_workspace_file(
        &workspace,
        "module.vx",
        r#"
function hidden() start
    return 99
end

export function visible() start
    return hidden()
end
"#,
    );
    let script = write_workspace_file(
        &workspace,
        "main.vx",
        r#"
import m from "./module.vx"
print m.visible()
print m.hidden()
"#,
    );
    let arg = script.to_string_lossy().to_string();

    let output = run_vexel(&workspace, &[&arg]);
    assert!(!output.status.success());
    assert!(stdout_text(&output).contains("99\n"));
    assert!(stderr_text(&output).contains("Function 'm.hidden' is not exported"));
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
