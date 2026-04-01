use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn repl_exits_cleanly_when_no_script_argument_is_provided() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vexel"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn vexel binary");

    child
        .stdin
        .as_mut()
        .expect("missing child stdin")
        .write_all(b"exit\n")
        .expect("failed to write exit command");

    let output = child.wait_with_output().expect("failed to wait on child");
    assert!(
        output.status.success(),
        "expected clean exit, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn repl_handles_nested_blocks_before_execution() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vexel"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn vexel binary");

    child
        .stdin
        .as_mut()
        .expect("missing child stdin")
        .write_all(
            b"function outer() start\nif true start\nprint \"ok\"\nend\nend\nouter()\nexit\n",
        )
        .expect("failed to write nested block commands");

    let output = child.wait_with_output().expect("failed to wait on child");
    assert!(
        output.status.success(),
        "expected clean exit, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ok"),
        "expected nested REPL script to run, stdout was: {stdout}"
    );
}
