# Vexel Language Guide

This document describes the current language behavior implemented in this repository.

## 1. Running Scripts

Build and run:

```sh
cargo run -- your_script.vx
```

Run REPL:

```sh
cargo run
```

Run WebCore mode:

```sh
cargo run -- webcore ./routes_folder
```

## 2. Syntax Basics

- Statements are line-based.
- `#` starts a comment (outside strings).
- Blocks use explicit `start` and `end`.
- Invalid/missing block delimiters are parse errors.

Example:

```vx
function greet(name) start
    print "Hello ${name}"
end
```

## 3. Data Types

Current runtime values:

- `number` (32-bit integer)
- `boolean` (`true` / `false`)
- `string`
- `array`
- `object`
- `null`

Important: `null` is a runtime value but **not** currently parsed as a literal token. A common way to obtain it is:

```vx
set nil json_parse("null")
```

## 4. Statements

### 4.1 Variable assignment

```vx
set x 42
set name "Vexel"
set arr [1,2,3]
set obj {a: 1, b: true}
```

### 4.2 Property assignment

```vx
set user {name: "A"}
set user.profile.rank "gold"
```

### 4.3 Printing

```vx
print "hello"
print x
```

### 4.4 Conditionals

```vx
if x > 10 start
    print "big"
end
```

### 4.5 Loops

```vx
for item in arr start
    print item
end

while x < 10 start
    set x math_add(x, 1)
end
```

### 4.6 Functions

```vx
function add(a, b) start
    return math_add(a, b)
end

print add(2, 3)
```

### 4.7 Module import/export

```vx
import mathx from "./mathx.vx"
print mathx.inc(5)
```

In module file:

```vx
export function inc(x) start
    return math_add(x, 1)
end
```

### 4.8 Test blocks

```vx
test "my check" start
    print "inside test"
end
```

Behavior:

- test body runs immediately when reached.
- tests run in isolated variable scope (outer variables are not visible).
- functions are available inside tests.

## 5. Expressions

Supported expression forms:

- literals: `123`, `true`, `"text"`
- variables: `name`
- function calls: `f(1,2)`
- comparisons: `==`, `!=`, `<`, `>`, `<=`, `>=`
- arrays: `[1,2,3]`
- objects: `{a: 1, b: "x"}`
- property access: `obj.field.nested`
- interpolation: `"hello ${name}"`

### String interpolation

```vx
set who "world"
print "hello ${who}"
```

### `+` operator behavior

`+` is parsed as chained `string_concat(...)`, not numeric addition.

Use `math_add(a, b)` for numeric addition.

## 6. Runtime and Errors

Vexel uses fail-fast execution:

- parse errors stop execution.
- runtime errors stop execution.
- CLI exits non-zero on failure.

Native built-ins return `None` on invalid arguments; runtime treats this as an error with a message like:

- `Native function 'name' failed for provided arguments`

## 7. Imports and Path Resolution

Imports are resolved relative to the importing script file directory.

```vx
import m from "./lib/module.vx"
```

Nested imports also resolve relative to their own file locations.

## 8. Standard Library

## 8.1 Math

- `math_add(a, b)`
- `math_subtract(a, b)`
- `math_multiply(a, b)`
- `math_divide(a, b)`
- `math_power(a, b)`
- `math_sqrt(a)`
- `math_abs(a)`

## 8.2 Arrays

- `array_push(arr, ...values)`
- `array_pop(arr)`
- `array_length(arr)`
- `array_get(arr, index)`
- `array_set(arr, index, value)`
- `array_slice(arr, start, end)`
- `array_join(arr, sep)`
- `array_to_string(arr)`
- `array_range(n)`

## 8.3 Strings

- `string_length(s)`
- `string_concat(a, b, ...)`
- `string_from_number(n)`
- `number_from_string(s)`
- `string_substring(s, start, length)`
- `string_contains(s, sub)`
- `string_replace(s, old, new)`
- `string_to_upper(s)`
- `string_to_lower(s)`
- `string_trim(s)`
- `string_starts_with(s, prefix)`
- `string_ends_with(s, suffix)`

## 8.4 Objects

- `object_to_string(value)`
- `object_keys(obj)`
- `object_values(obj)`
- `object_has_property(obj, key)`
- `object_merge(a, b)`
- `object_create(k1, v1, k2, v2, ...)`

## 8.5 JSON

- `json_parse(text)`
- `json_stringify(value)`

## 8.6 Filesystem

- `read_file(path)`
- `write_file(path, content)`
- `append_file(path, content)`
- `file_exists(path)`
- `delete_file(path)`
- `rename_file(from, to)`
- `create_dir(path)`
- `list_dir(path)`

## 8.7 Core

- `sleep(seconds)`
- `type_of(value)`
- `is_null(value)`
- `exec(command)`

## 8.8 HTTP

- `http_get(url)`
- `http_post(url, body)`
- `http_put(url, body)`
- `http_delete(url)`

## 8.9 Thread Messaging

- `thread_channel()` -> returns channel id
- `thread_send(channel_id, value)`
- `thread_recv(channel_id)` (blocking)
- `thread_close(channel_id)`

## 8.10 Debug

- `dump(value)`
- `dump_type(value)`
- `assert_equal(a, b)`

Note: debug helpers may be incompatible with strict fail-fast flows because some return no value.

## 9. WebCore

A `.vx` file can define an HTTP route using top-level variables:

```vx
set path "/users/{id}"
set method "GET"
set mime "application/json"

function request(id) start
    return json_stringify({id: id})
end
```

WebCore behavior:

- loads all `.vx` files from a folder.
- default route path is `/<filename_without_ext>` if `path` is absent.
- default method is `GET` if `method` is absent.
- default MIME type is `text/plain` if `mime` is absent.
- `mime` sets the HTTP `Content-Type` response header for that route.
- current path templating supports one captured segment pattern per route usage.

## 10. Current Limitations / Gotchas

- No `else` keyword.
- No logical operators like `&&` / `||`.
- Expressions are mostly expected on one line.
- `null` is not a direct parser literal token.
- Numeric type is integer-only (`i32`).
- Function argument counts must match exactly.
- `test` blocks do not inherit outer variables.

## 11. Minimal Example

```vx
import m from "./module.vx"

set i 0
while i < 3 start
    print m.inc(i)
    set i math_add(i, 1)
end
```

`module.vx`:

```vx
export function inc(x) start
    return math_add(x, 1)
end
```
