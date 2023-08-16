# webSocket-std

Implementation of websocket for rust with std support, means that any embeded systems which have support for the std library should works. Of course this library works on any normal computer.

## MCUs Tested

### ESP32

See ``esp-rs`` project with ``std`` support: [esp-rs docs](https://esp-rs.github.io/book/overview/using-the-standard-library.html)

The library was tested on
- ESP32

# Test

---

## Execute all test

```console
cargo test
```

## Generate coverage report

### Requirements

#### Install ``grcov`` tool

1. Install the llvm-tools or llvm-tools-preview component
```console
rustup component add llvm-tools-preview
```

2. Ensure that the following environment variable is set up
```console
export RUSTFLAGS="-Cinstrument-coverage"
```

3. Ensure each test runs gets its own profile information by defining the LLVM_PROFILE_FILE environment variable (%p will be replaced by the process ID, and %m by the binary signature)
```console
export LLVM_PROFILE_FILE="websocket-std-%p-%m.profraw"
```

4. Install grcov

```console
cargo install grcov
```

### Generate report

Ensure that there isn't compilation or test errors.
1. Build the code
```console
cargo build
```

2. Run tests and ensure that all are ``OK``
```console
cargo test
```

3. Be sure that the variables are exported. 
- RUSTFLAGS
- LLVM_PROFILE_FILE

4. Generate coverage report as HTML
```console
grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage --excl-line grcov-excl-line
```

The report will be generated at ``target/coverage/``, open ``index.html`` with a browser to see the results.
