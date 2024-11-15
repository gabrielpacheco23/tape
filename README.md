# Tape

## A cell-based esoteric programming language with JIT compilation
Tape is a cell-based, brainfuck-like programming language that has a readable syntax which features both a JIT compiler and a bytecode VM with a non-wasted array-like structure to manage memory.

## Example

```
#! This program outputs the letter A and a newline character
make tape[2]
make ptr: idx

incr tape[ptr]
+64
putch

incr ptr
incr tape[ptr]
+9
putch
```

This code example prints the letter "A" followed by a newline character. Check the examples folder for the implementation using loops.

## Build
To build the executable you must have cargo and rust installed.

```
cargo build --release
```

## Usage
To run using the bytecode VM:

```
tape <filename> 
```

Using the JIT compiler:

```
tape --jit <filename>
```

The flag --verbose (or -v) outputs additional information:
```
[Using JIT compiler]

Hello World!
 ```

## License
MIT [License](LICENSE)

Copyright © 2024 Gabriel Pacheco

