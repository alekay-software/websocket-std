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

## Python websocket server to test
Code of a python3 websocket server to test the client

### Requirements
- pip install websockets==11.0.3

### Code
```python
import asyncio
from websockets.frames import Frame
from websockets.server import serve
from websockets.extensions import ServerExtensionFactory, Extension
from websockets.typing import ExtensionParameter, ExtensionName
from typing import Optional, Sequence, Tuple, List, Union

HOST  = "0.0.0.0"
PORT  = 3000
protocol = ["scoreboard", "app"]

class PersonExtension(Extension):
    def __init__(self, name: str, person_name):
        self.name = ExtensionName(name)
        self.person_name = person_name

    def decode(self, frame: Frame, *, max_size: Union[int, None] = None) -> Frame:
        return frame
    
    def encode(self, frame: Frame) -> Frame:
        frame.data += (" " + self.person_name).encode("utf-8")
        return frame

def process_param(
        params: Sequence[ExtensionParameter], 
        accepted_extensions: Sequence[Extension]
    ) -> Tuple[List[ExtensionParameter], Extension]:

    ext = PersonExtension("person", params[0][1])

    return ([*params], ext)

sef = ServerExtensionFactory()
sef.name = "person"
sef.process_request_params = process_param

async def echo(websocket):
    async for message in websocket:
        if websocket.open:
            # print("Message: ", message)
            await websocket.send(message)

async def main():
    async with serve(echo, HOST, PORT, subprotocols=protocol, extensions=[sef]):
        await asyncio.Future()  # run forever

asyncio.run(main())
```


## Create a static lib for C++
Using the cbindgen tool
```console
cbindgen --config cbindgen.toml --crate websocket-std --output websocket-std.h
```

or just create build.rs inside the crate to generate the code when build with cargo.
Also include dev dependenci in cargo.toml

```toml
[build-dependencies]
cbindgen = "0.24.0"
```


```rust
extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
      .with_crate(crate_dir)
      .generate()
      .expect("Unable to generate bindings")
      .write_to_file("bindings.h");
}
```