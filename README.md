# build_wrapper
### Why we need wrapper
Cargo is default build tool for Rust.

It's easy to use, but not flexible enough.

Some times we want to use custom compile args.

### How to work

It works like a proxy.

Got the args from Cargo, modify it, then pass to real compiler - rustc.

### How to use
```
RUSTC_WRAPPER=/path/to/build_wraper cargo build --release
```

### Notes
1. Some code from [Clippy](https://github.com/rust-lang/rust-clippy).
2. Cargo got some info from the output of rustc.
As a rustc wrapper, we must process sysroot.
We can't add extra output, So it's very difficult to debug.

### Plan
Now the build wrapper is for [cita](https://github.com/cryptape/cita).

Check args then add debug info if it's a file belong to cita project.

So we can reduce the size of release binary.

Maybe we can invoke another tool or a script to process the args.

It will be easy to custom compile args.

This will make it to be a general tool.

