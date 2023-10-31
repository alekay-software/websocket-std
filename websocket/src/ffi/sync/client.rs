use super::super::super::sync::client::{Config, WSEvent as RWSEvent, WSData, WSClient};
use std::ffi::{c_void, c_char, CStr, c_int};
use std::alloc::{alloc, dealloc, Layout};
use std::mem;
use std::str;
use std::ptr;

// #[repr(C)]
// struct Config {
//     callback: *mut c_void,
    
// }

// #[repr(C)]
// struct CWSClient<'a> {
//     client: WSClient<'a, *mut c_void>
// }

#[no_mangle]
extern "C" fn SyncWSClientNew<'a>() -> *mut WSClient<'a, *mut c_void> {
    // let size = mem::size_of::<WSClient<*mut c_void>>();
    // let layout = Layout::from_size_align(size, size);
    
    // if layout.is_err() { return std::ptr::null_mut() }

    // let client = unsafe { alloc(layout.unwrap()) };
    
    // client as *mut WSClient<*mut c_void>
    Box::into_raw(Box::new(WSClient::new()))
}

#[no_mangle]
unsafe extern "C" fn SyncWSClientInit<'a>(client: *mut WSClient<'a, *mut c_void>, host: *const c_char, port: u16, path: *const c_char, callback: *mut c_void) {
    let host = str::from_utf8(CStr::from_ptr(host).to_bytes()).unwrap();
    let path = str::from_utf8(CStr::from_ptr(path).to_bytes()).unwrap();

    let callback: fn(&mut WSClient<'a, *mut c_void>, &RWSEvent, Option<WSData<*mut c_void>>) = mem::transmute(callback);
    let config = Config { callback: Some(callback), data: None, protocols: None };
    
    let client = &mut *client;
    client.init(host, port, path, Some(config)).unwrap();
    println!("Client init");
}

#[no_mangle]
unsafe extern "C" fn SyncWSClientLoop<'a>(client: *mut WSClient<'a, *mut c_void>) -> c_int {
    let mut status = 1;

    let client = &mut *client;
    
    match client.event_loop() {
        Ok(_) => {},
        Err(e) => { 
            println!("Error {}", e);
            status = 0;
        }
    }

    return status; 
}

#[no_mangle]
unsafe extern "C" fn SyncWSClientSend<'a>(client: *mut WSClient<'a, *mut c_void>, message: *const c_char) -> c_int {
    let mut status = 1; 
    let msg = str::from_utf8(CStr::from_ptr(message).to_bytes()).unwrap(); 

    let client = &mut *client;
    match client.send(msg) {
        Ok(_) => {},
        Err(e) => {
            println!("Error {}", e);
            status = 0
        } 
    }

    return status;
}

#[no_mangle]
extern "C" fn SyncWSClientDrop<'a>(mut client: *mut WSClient<'a, *mut c_void>) -> *mut c_void {
    // Create a box from the raw pointer, at the end of the function the client will be dropped and the memory will be free.
    unsafe {
       let _ = Box::from_raw(client);
    }
    client = ptr::null_mut();
    ptr::null_mut()
}

#[repr(C)]
enum WSEvent_t {
    ON_CONNECT,
    ON_TEXT,
    ON_CLOSE,
}

// #[repr(C)]
// struct WSEvent {
//     event: WSEvent_t,
//     data: *mut c_void 
// }

#[no_mangle]
extern "C" fn fromRustEvent(event: &RWSEvent) -> WSEvent_t {
    match event {
        RWSEvent::ON_CONNECT => { WSEvent_t::ON_CONNECT },
        RWSEvent::ON_TEXT(_) => { WSEvent_t::ON_TEXT },
        RWSEvent::ON_CLOSE(_) => { WSEvent_t::ON_CLOSE }
    }
}

// #[no_mangle]
// extern "C" fn WSEventConnect() ->  WSEvent {
//     WSEvent::ON_TEXT()
// }

// #[no_mangle]
// extern "C" fn WSEventConnect() ->  WSEvent {
//     WSEvent::CLOSE
// }