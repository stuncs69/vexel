# 🐿️ Vexel

Vexel is an interpreted scripting language designed for simple, readable automation scripts. It focuses on straightforward syntax, built-in standard library helpers, and deterministic runtime behavior.

## Features

- **Simple Syntax**: Clean and easy-to-read syntax for quick development.
- **Dynamic Typing**: No need to explicitly declare variable types.
- **Fail-fast Runtime**: Parsing/runtime errors stop execution with clear stderr output.
- **Structured Control Flow**: Supports `if`, `else if`, `else`, `for`, `while`, `break`, and `continue`.
- **Runtime Error Handling**: `try` / `catch` can recover from runtime failures inside scripts.
- **Script-relative Imports**: `import` paths are resolved relative to the importing `.vx` file.
- **Literal `null` Support**: `null` is a first-class runtime value and parsed literal.
- **Message-passing Threads**: Thread primitives use channels (`thread_channel`, `thread_send`, `thread_recv`, `thread_close`).

## Example Code

```vx
set scores [42, 77, 91]
set passing []

function label(score) start
    if score >= 90 start
        return "excellent"
    else if score >= 50 start
        return "passing"
    else start
        return "failing"
    end
end

for score in scores start
    if score < 50 start
        continue
    end
    set passing array_push(passing, score)
    print "score=${score}, label=${label(score)}"
end

try start
    print missing_value
catch err start
    print "caught=${err}"
end

print array_join(passing, ",")
```

## Installation

Vexel is built using Rust. To install and build Vexel, ensure you have Rust installed and run:

```sh
git clone https://github.com/stuncs69/vexel.git
cd vexel
cargo build --release
```

## Running Vexel Scripts

After building, you can execute Vexel scripts using:

```sh
target/release/vexel script.vx
```

Run only the `test` blocks in a script:

```sh
target/release/vexel --test script.vx
```

## Documentation

- Full language reference: [LANGUAGE.md](LANGUAGE.md)

## Language Notes

- Blocks use explicit `start` / `end` delimiters.
- `if` blocks can use `else if` and `else`.
- Loops support `break` and `continue`.
- `try` / `catch` can handle runtime errors and bind the error message to a variable.
- `null` is available as a literal.
- `test` blocks only run when the CLI is invoked with `--test`.
- Runtime errors are fail-fast and return a non-zero exit code in CLI mode.
- Relative imports are resolved from the importing file's directory.
- In WebCore routes, optional `mime` controls the `Content-Type` header (default `text/plain`).

## Roadmap

- [x] Add more built-in functions
- [x] Implement loops (`for`, `while`)
- [x] Improve error handling
- [x] Add importing files support
- [ ] Add package management support
- [x] Array support
- [x] Add object support
- [x] Add REPL

## Contributing

Contributions are welcome! Feel free to submit issues and pull requests to improve Vexel.

## License

Vexel is licensed under the MIT License. See `LICENSE` for details.
