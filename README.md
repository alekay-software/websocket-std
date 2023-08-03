Implementation of websocket for rust std.

# Test

---

## Execute all test

```console
cargo test
```

## Generate coverage report

### Requirements

#### Install ``grcov`` tool

```console
cargo install grcov
```

### Run all test before generate the report
```console
cargo test
```

### Generate coverage report as HTML
```console
grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage --excl-line grcov-excl-line
```

The report will be generated at ``target/coverage/``, open index.html with a browser to see the results.

## Puede que no haga falta (probar luego)
The instructions were taken from this [repo](https://github.com/mozilla/grcov#how-to-get-grcov).

1. Install the llvm-tools or llvm-tools-preview component
```console
rustup component add llvm-tools-preview
```

2. Ensure that the following environment variable is set up
```console
export RUSTFLAGS="-Cinstrument-coverage"
```

3. Build the code
```console
cargo build
```

4. Ensure each test runs gets its own profile information by defining the LLVM_PROFILE_FILE environment variable (%p will be replaced by the process ID, and %m by the binary signature)
```console
export LLVM_PROFILE_FILE="websocket-std-%p-%m.profraw"
```

5. Finally run tests
```console
cargo test
```