# 🐿️ Vexel

Vexel is an interpreted scripting language designed for simple, readable automation scripts. It focuses on straightforward syntax, built-in standard library helpers, and deterministic runtime behavior.

## Features

- **Simple Syntax**: Clean and easy-to-read syntax for quick development.
- **Dynamic Typing**: No need to explicitly declare variable types.
- **Fail-fast Runtime**: Parsing/runtime errors stop execution with clear stderr output.
- **Script-relative Imports**: `import` paths are resolved relative to the importing `.vx` file.
- **Message-passing Threads**: Thread primitives use channels (`thread_channel`, `thread_send`, `thread_recv`, `thread_close`).

## Example Code

```bash
import custom from "./import.vx"

set x 5
set greeting "hello"
set is_active true

print greeting

function squirrelsay(message) start
    print string_concat("🐿️ - ", message)
end

set array ["H", "i"]
set array array_push(array, "!")

if is_active != false start
    set string_array array_to_string(array)
    squirrelsay(string_array)
end

custom.is_hello(greeting) # True

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

## Documentation

- Full language reference: [LANGUAGE.md](LANGUAGE.md)

## Language Notes

- Blocks use explicit `start` / `end` delimiters.
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
