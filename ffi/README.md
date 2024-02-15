# Static library 

## Compiling the library
Steps to Generate the Static Library:
- Add the following lines to the **Cargo.toml** file located in ``websocket/Cargo.toml``, right after the keywords section:
```toml
[lib]
name = "websocket_std"
crate-type = ["staticlib"]
```
This instructs Cargo to generate a static library named ``websocket_std``

- Compile the project using Cargo, specifying the target platform if the architecture where you compile the project differs from the architecture where the library will be linked. You can view the list of available platforms in your Rust compiler as follows:
```console
rustc --print target-list
```

If the target platform is not listed, you can use the ``rustup toolchain`` tool to add the necessary platforms.</br>

Go to the root of the project.

```console
# Compile for the same platform
cargo build --release

# Compile for another platform
cargo build --release --target=<platform>
```

- If you compiled the code without specifying a target platform, the generated static library will be located at ``target/release/libwebsocket_std.a``.
- If you compiled it for a specific target platform, the library will be in ``<platform>/target/release/libwebsocket_std.a``.

# Linking the library

In this folder you will find a ``main.c`` example using the library and also a header file in ``websocket/websocket-std.h`` which contains the API and the types for the websocket. You can use your prefered compiler, in my case I'll use clang.

- `-I` [path where websocket_std.h header file is]
- `-L` [path where libwebsocket_std.a is] Path to find static libaries 
- `-lwebsocket_std` Link websocket library

In this example the structure of the project is: 
```console
├── src 
│   ├── main.c
│   ├── websocket
│   │   ├── websocket-std.h
│   │   ├── libwebsocket_std.a
```

```console
clang -o main main.c -I websocket -L websocket -lwebsocket_std
```