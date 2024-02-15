# websocket-std

This is a WebSocket implementation in Rust that utilizes the standard library. Its goal is to function on low-resource devices such as microcontrollers, although it can also be applied in high-performance computer programs. I started the project for three main reasons:
- Learn rust.
- Publish a library in crates.io.
- How to use the library from other languages. 

The project is currently in beta phase, and while it is functional, further work is needed for the final version. Since I am a sole developer and cannot dedicate as much time as I would like, I have decided to publish it for those who want to try it out and contribute to the project [CONTRIBUTING.md](./CONTRIBUTING.md).

You can use the library in the following ways:

- In any Rust project that allows the use of the standard library, such as ``esp-rs`` with ``std`` support. Check out the [esp-rs docs](https://esp-rs.github.io/book/overview/using-the-standard-library.html) for more information.
- In any C project, as it has a compatible FFI (Foreign Function Interface). You’ll need to compile the project as a static library and link it appropriately. Refer to this guide ([static lib usage](./ffi/README.md)) for more details.

**Feel free to explore the project and contribute! 🚀**

---

## Installation
```toml
[dependencies]
websocket-std = "0.0.5"
```

## Static library
In the ``ffi/`` folder you will find the ``websocket-std.h`` header and a compiled static library for the xtensa architecture of the esp32 microcontroller from espressif.

You can use this static library in your esp idf and arduino projects for esp32. Check [ffi/xtensa-esp32-idf](./ffi/xtensa-esp32-espidf/README.md) for more information.

## Examples

The [examples](./examples/) folder contains various examples of how to use ``websocket-std``.

## Features

### Sync Client

The sync client manage an internal event loop, when you call a function to perform a websocket operation (``init``, ``send``, ...)
it will be queued and as soon as you call the ``event_loop`` function it will perform one input (something was received)
and one output (something to send to server) operations in one execution.

You can also use ``threads`` to work with the library. Check [examples](./examples/) for more information.

#### What works
- Send text messages.
- Handle received text messages.
- Handle on connection events.
- Handle on close events.
- Work with websocket protocols.
- Set the maximun length of the text that the websocket will send for each dataframe.

#### Comming
- Websocket over SSL.
- Send and receive binary data.
- Websocket extensions.

### Sync Server

I'm planning also to introduce in the library a ``sync server`` following the same philosophy as the sync client.

---

## MCUs Tested

- ``ESP32`` using **esp-rs** with std support
- ``ESP32`` using ``arduino`` framework in a ``PlatformIO`` project. (Should also work with esp-idf proyects).

---

# Test

Since is my first rust big project I started using the following tools for testing and code coveragera, but I would like to
define another way of doing that because the test coverage reports are not the bests. I'm open to hear better ways of doing testing in rust.

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

---

## Python websocket server

Since the library doesn't have a way to create websocket servers, here you will find an echo server example in python to test
the client.

### Requirements
- pip install websockets==11.0.3

### Code
```python
import asyncio
from websockets.server import serve

HOST  = "0.0.0.0"
PORT  = 3000
protocol = ["superchat", "app", "chat"]

async def echo(websocket):
    async for message in websocket:
        if websocket.open:
            await websocket.send(message)

async def main():
    async with serve(echo, HOST, PORT, subprotocols=protocol):
        print(f"Websocket server running on: {HOST}:{PORT}")
        await asyncio.Future()  # run forever

asyncio.run(main())
```
