# watto.

a collection for tools for my own cpu - watto

(yes this is take 2 with cpus, this time much more successful!)

## wasp.

the assembler

## weser.

the emulator

## example.

to run any example program (which can be found in `progs/`) you will first want to compile it:
```shell
$ cargo run -p wasp --release -- -s progs/nums.wts -o nums.wte
```

then run it using the emulator:
```shell
$ cargo run -p weser --release -- nums.wte serial
```
