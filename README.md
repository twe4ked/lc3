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
Waiting for redis-cli connection...
```

From a separate shell:

```
$ redis-cli
```

## TODO

- Finish implementing TRAP codes

## More info

- http://highered.mheducation.com/sites/0072467509/student_view0/lc-3_simulator.html
