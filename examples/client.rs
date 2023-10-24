use std::time::Instant;
use websocket_std::client::{SyncClient, Config, Reason};
use websocket_std::result::WebSocketResult;
use std::sync::Arc;
use std::cell::RefCell;

struct Data {
    count: usize,
    connected: bool,
    close: bool,
}

type WebSocket<'a> = SyncClient<'a, RefCell<Data>>;
type WSData = Arc<RefCell<Data>>;

fn on_message(ws: &mut WebSocket, _msg: String, data: Option<WSData>) {
    let data = data.unwrap();
    let mut data = data.borrow_mut();
    data.count += 1;
    println!("[SERVER]: {}", _msg);
    ws.send("Hello world").unwrap();
}

fn on_connect(ws: &mut WebSocket, data: Option<WSData>) {
    println!("Connected");
    let protocol = if ws.protocol().is_some() { ws.protocol().unwrap() } else { "--" };
    println!("Accepted protocol: {protocol}");
    let data = data.unwrap();
    let mut data = data.borrow_mut();
    data.connected = true;
    ws.send("Hello world");
}

fn on_close(reason: Reason, data: Option<WSData>) {
    let mut who_closed = "";
    let mut code = 0u16;

    match reason {
        Reason::SERVER_CLOSE(c) => {
            who_closed = "server";
            code = c;
        },

        Reason::CLIENT_CLOSE(c) => {
            who_closed = "client";
            code = c;
        }
    }

    println!("Connection closed by {who_closed}, code: {code}");

    let data = data.unwrap();
    let mut data = data.borrow_mut();
    data.close = true;
}

fn main() -> WebSocketResult<()> {
    let host: &str = "localhost";
    let port: u16 = 3000;
    let path: &str = "/";
    let data: WSData = Arc::new(RefCell::new(Data { count: 0, connected: false, close: false }));

    let config = Config {
        on_connect: Some(on_connect),
        on_data: Some(on_message),
        // on_close: Some(on_close),
        on_close: Some(|_, _| { println!("Closure para cerrar la conexion") }),
        data: Some(data.clone()),
        protocols: Some(&["chat", "superchat"])
    };
    
    // let p1 = parameter!("person");
    // let p2 = parameter!("person"; "name");
    // let p3 = parameter!("person"; "name=sergio", "apellido=ramirez ojea");
    // let p4 = parameter!("person"; "name=sergio", "apellido=ramirez", "edad=24");
    
    println!(
        "Connecting to {host}:{port}{path}",
        host = host,
        port = port,
        path = path
    );

    // List of protocols to accept
    let protocols = ["socoreboard", "chat"];
    let mut c1: WebSocket = SyncClient::new();


    if let Some(protocol) = c1.protocol() {
        println!("Accepted protocol: {}", protocol); 
    }

    c1.set_message_size(1024);
    c1.init(host, port, path, Some(config))?;

    // Esperar hasta que se conecte para enviar mensajes
    // Poner los mensajes de http a enviar al principio siempre...

    // c1.send("Hello world")?;
    let start = Instant::now();
    while !data.borrow().close {
        c1.event_loop()?; 
        if start.elapsed().as_secs() >= 10 { break }
    }

    drop(c1); 
    println!("Finishing connection");
    let count = data.borrow().count;
    println!("Total messages received: {}", count);
    Ok(())
}