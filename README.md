# Vexel

Vexel is a high-performance, JIT-compiled scripting language designed for efficiency and ease of use. It features a simple and intuitive syntax while leveraging just-in-time (JIT) compilation for optimal execution speed. 🚀

## Features

- **JIT Compilation**: Execute code with high efficiency using just-in-time compilation.
- **Simple Syntax**: Clean and easy-to-read syntax for quick development.
- **Dynamic Typing**: No need to explicitly declare variable types.

## Example Code

```vexel
function x(a) start
    print a
end

x("test")

if true == true start
    x("true!!")
end

if false != true start
    x("false!!")
end

set y string_concat("hello", "world")

x(y)
```

## Installation

Vexel is built using Rust. To install and build Vexel, ensure you have Rust installed and run:

```sh
git clone https://github.com/yourusername/vexel.git
cd vexel
cargo build --release
```

## Running Vexel Scripts

After building, you can execute Vexel scripts using:

```sh
target/release/vexel script.vx
```

## Roadmap

- [ ] Add more built-in functions
- [ ] Implement loops (`for`, `while`)
- [ ] Improve error handling
- [ ] Add package management support
- [x] Array support

## Contributing

Contributions are welcome! Feel free to submit issues and pull requests to improve Vexel.

## License

Vexel is licensed under the MIT License. See `LICENSE` for details.
