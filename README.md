# Tape

## A cell-based esoteric programming language
Tape is a cell-based, brainfuck-like programming language that has a readable syntax and a non-wasted array-like structure to manage memory.

## Example

```
#! This program outputs the letter A and a newline character
make tape[3]
make ptr: idx

incr tape[ptr]
+4

loop (
    incr ptr
    incr tape[ptr]
    +11

    incr ptr
    incr tape[ptr]
    +1

    decr ptr
    +1
    decr tape[ptr]
)

incr ptr
incr tape[ptr]
+4
putch

incr ptr
putch
```

This code example prints the letter "A" followed by a newline character.

## License
MIT [License](LICENSE)

Copyright © 2021 Gabriel Pacheco
