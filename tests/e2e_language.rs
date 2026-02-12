mod common;

use common::{
    assert_stdout_lines, create_workspace, run_script, stderr_text, write_workspace_file,
};

#[test]
fn executes_arithmetic_loops_functions_and_conditionals() {
    let workspace = create_workspace("core_flow");
    write_workspace_file(
        &workspace,
        "main.vx",
        r#"
set sum 0
for x in array_range(5) start
    set sum math_add(sum, x)
end
function is_ten(v) start
    if v == 10 start
        return "yes"
    end
    return "no"
end
print sum
print is_ten(sum)
set i 0
while i < 3 start
    print i
    set i math_add(i, 1)
end
"#,
    );

    let output = run_script(&workspace, "main.vx");
    assert!(
        output.status.success(),
        "script failed: {}",
        stderr_text(&output)
    );
    assert_stdout_lines(&output, &["10", "yes", "0", "1", "2"]);
}

#[test]
fn executes_array_and_object_features() {
    let workspace = create_workspace("arrays_objects");
    write_workspace_file(
        &workspace,
        "main.vx",
        r#"
set arr [1,2]
set arr array_push(arr,3,4)
print array_length(arr)
print array_get(arr,2)
set arr array_set(arr,1,9)
print array_join(arr,",")
set obj {a: 1}
set obj.b 2
set obj.nested.inner "x"
print object_has_property(obj,"b")
print obj.nested.inner
print type_of(obj.nested.inner)
"#,
    );

    let output = run_script(&workspace, "main.vx");
    assert!(
        output.status.success(),
        "script failed: {}",
        stderr_text(&output)
    );
    assert_stdout_lines(&output, &["4", "3", "1,9,3,4", "true", "x", "string"]);
}

#[test]
fn executes_string_and_interpolation_features() {
    let workspace = create_workspace("strings");
    write_workspace_file(
        &workspace,
        "main.vx",
        r#"
set name "Tim"
set msg "Hello ${name}, ${math_add(2,3)}!"
print msg
print "Cost: \$5"
print type_of(1 > 0)
set src "éx"
print string_substring(src,0,1)
print string_from_number(42)
print number_from_string("42")
"#,
    );

    let output = run_script(&workspace, "main.vx");
    assert!(
        output.status.success(),
        "script failed: {}",
        stderr_text(&output)
    );
    assert_stdout_lines(
        &output,
        &["Hello Tim, 5!", "Cost: \\$5", "boolean", "é", "42", "42"],
    );
}

#[test]
fn executes_imported_module_functions() {
    let workspace = create_workspace("import_module");
    write_workspace_file(
        &workspace,
        "module.vx",
        r#"
function helper(x) start
    return math_add(x, 1)
end

export function inc(x) start
    return helper(x)
end
"#,
    );
    write_workspace_file(
        &workspace,
        "main.vx",
        r#"
import m from "./module.vx"
print m.inc(5)
"#,
    );

    let output = run_script(&workspace, "main.vx");
    assert!(
        output.status.success(),
        "script failed: {}",
        stderr_text(&output)
    );
    assert_stdout_lines(&output, &["6"]);
}

#[test]
fn resolves_imports_relative_to_importing_script() {
    let workspace = create_workspace("import_relative");
    write_workspace_file(
        &workspace,
        "app/lib/math.vx",
        r#"
export function inc(x) start
    return math_add(x, 1)
end
"#,
    );
    write_workspace_file(
        &workspace,
        "app/main.vx",
        r#"
import m from "./lib/math.vx"
print m.inc(9)
"#,
    );

    let output = run_script(&workspace, "app/main.vx");
    assert!(
        output.status.success(),
        "script failed: {}",
        stderr_text(&output)
    );
    assert_stdout_lines(&output, &["10"]);
}

#[test]
fn executes_json_round_trip() {
    let workspace = create_workspace("json_roundtrip");
    write_workspace_file(
        &workspace,
        "main.vx",
        r#"
set obj {x: 1, ok: true}
set serialized json_stringify(obj)
print serialized
set parsed json_parse(serialized)
print parsed.x
print parsed.ok
"#,
    );

    let output = run_script(&workspace, "main.vx");
    assert!(
        output.status.success(),
        "script failed: {}",
        stderr_text(&output)
    );
    let stdout = common::stdout_text(&output);
    assert!(stdout.contains("{\"x\":1") || stdout.contains("{\"ok\":true"));
    assert!(stdout.contains("1\n"));
    assert!(stdout.contains("true\n"));
}

#[test]
fn executes_filesystem_functions() {
    let workspace = create_workspace("fs_functions");
    write_workspace_file(
        &workspace,
        "main.vx",
        r#"
set p "a.txt"
print file_exists(p)
set _ write_file(p, "a")
set _ append_file(p, "b")
print read_file(p)
set _ rename_file(p, "b.txt")
print file_exists("b.txt")
set _ create_dir("dir1")
set _ write_file("dir1/c.txt", "x")
set entries list_dir("dir1")
print array_length(entries)
set _ delete_file("b.txt")
print file_exists("b.txt")
"#,
    );

    let output = run_script(&workspace, "main.vx");
    assert!(
        output.status.success(),
        "script failed: {}",
        stderr_text(&output)
    );
    assert_stdout_lines(&output, &["false", "ab", "true", "1", "false"]);
}

#[test]
fn executes_thread_message_passing() {
    let workspace = create_workspace("threading");
    write_workspace_file(
        &workspace,
        "main.vx",
        r#"
set ch thread_channel()
set _ thread_send(ch, 5)
print thread_recv(ch)
set _ thread_send(ch, "done")
print thread_recv(ch)
set _ thread_close(ch)
"#,
    );

    let output = run_script(&workspace, "main.vx");
    assert!(
        output.status.success(),
        "script failed: {}",
        stderr_text(&output)
    );
    assert_stdout_lines(&output, &["5", "done"]);
}
