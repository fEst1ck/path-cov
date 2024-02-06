# Regular expression based path reduction

The `path_reduction` library provides functionality for reducing execution paths based on the control flow structure of the program. The crate can be compiled in a C compatible dynamic library. The header file is in `./header`.

# Build

```shell
cargo build --release
```

The compiled library can be found in `target/release` named `libpath_reduction.*` depending on your platform.