use super::super::client::{SyncClient, Config, WSEvent, WSData};
use std::ffi::{c_void, c_char, CStr, c_int};
use std::mem;
use std::str;

// #[repr(C)]
// struct Config {
//     callback: *mut c_void,
    
// }

// #[repr(C)]
// struct CSyncClient<'a> {
//     client: SyncClient<'a, *mut c_void>
// }

#[no_mangle]
extern "C" fn syncClientNew<'a>() -> *mut SyncClient<'a, *mut c_void> {
    Box::into_raw(Box::new(SyncClient::new()))
}

#[no_mangle]
unsafe extern "C" fn syncClientInit<'a>(client: *mut SyncClient<'a, *mut c_void>, host: *const c_char, port: u16, path: *const c_char, callback: *mut c_void) {
    let host = str::from_utf8(CStr::from_ptr(host).to_bytes()).unwrap();
    let path = str::from_utf8(CStr::from_ptr(path).to_bytes()).unwrap();

    let callback: fn(&mut SyncClient<'a, *mut c_void>, WSEvent, Option<WSData<*mut c_void>>) = mem::transmute(callback);
    let config = Config { callback: Some(callback), data: None, protocols: None };
    // SyncClient::init(&mut *client, host, port, path, Some(config));
    let client = &mut *client;
    client.init(host, port, path, Some(config)).unwrap();
    println!("Client init");
}

#[no_mangle]
unsafe extern "C" fn syncClientLoop<'a>(client: *mut SyncClient<'a, *mut c_void>) -> c_int {
    let mut status = 1;

    let client = &mut *client;
    match client.event_loop() {
        Ok(_) => {},
        Err(e) => { 
            println!("Error {}", e);
            status = 0 
        }
    }

    return status; 
}

#[no_mangle]
unsafe extern "C" fn syncClientSend<'a>(client: *mut SyncClient<'a, *mut c_void>, message: *const c_char) -> c_int {
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