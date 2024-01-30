# Stats

## Websocket Server (Python)
Echo server
```python
#!/usr/bin/env python
import asyncio
from websockets.server import serve

HOST = "localhost"
PORT = 3000

async def echo(websocket):
    async for message in websocket:
        await websocket.send(message)

async def main():
    async with serve(echo, HOST, PORT):
        await asyncio.Future()  # run forever

asyncio.run(main())
```

### Rust websocket-std client

3 seconds sending and reading messages: 
- Mac M1: Messages received --> 75949 (server and client in the same machine)
- ESP32 Rust: Messages received --> 1598 - 1186 - 880
- ESP32 Arduino C++: Messages received --> 859

```rust
use std::thread;
use std::time::{Duration, Instant};
use websocket_std::client::{sync_connect, SyncClient};
use websocket_std::result::WebSocketResult;

struct Data {
    count: usize,
}

unsafe fn on_message(msg: String, data: *mut Data) {
    (*data).count += 1;
}

fn main() -> WebSocketResult<()> {
    let host: &'static str = "localhost"; // Make static lifetime, &str lives for the entire lifetime of the running program.
    let port: u16 = 3000;
    let path: &'static str = "/";
    let data_box = Box::new(Data { count: 0 });
    let data = Box::into_raw(data_box);

    println!(
        "Connecting to {host}:{port}{path}",
        host = host,
        port = port,
        path = path
    );

    let mut client: SyncClient<'static, Data> = sync_connect(host, port, path)?;
    println!("Connected to VAM Scoreboard");

    client.set_response_cb(on_message, data);

    client.set_message_size(1024);

    let start = Instant::now();

    loop {
        client.send_message(String::from("Hello world"))?;
        if !client.is_connected() {
            println!("Disconnected");
            break;
        }
        client.event_loop()?;
        if start.elapsed().as_secs() >= 3 { break }
    }

    println!("Messages sent: {}", unsafe { (*data).count} );
    Ok(())
}

```

### Rust websocket-std client Apple Mac M2
