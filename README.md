# [LC-3 VM](https://en.wikipedia.org/wiki/LC-3)

A LC-3 VM written in Rust.

## Running

```
$ cargo run [path.obj]
```

Example applications:

- https://github.com/rpendleton/lc3-2048
- https://github.com/justinmeiners/lc3-rogue

### Debugging

```
$ cargo run [path.obj] --debug
Waiting for connection...
```

From a separate shell:

```
$ nc 127.0.0.1 6379
h
c, continue               Continue execution.
r, registers              Print registers.
   condition              Print condition.
d, disassemble            Disassemble current instruction.
   read <addr>            Read and display memory address. e.g. read 0x3000
   break-address <addr>   Break at address. e.g. read 0x3000
```

## TODO

- Finish implementing TRAP codes

## More info

- http://highered.mheducation.com/sites/0072467509/student_view0/lc-3_simulator.html
