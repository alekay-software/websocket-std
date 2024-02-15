use super::super::super::sync::client::{Config, WSEvent as RWSEvent, WSClient};
use std::ffi::{c_void, c_char, CStr};
use std::alloc::{alloc, Layout};
use std::mem;
use std::ptr;
use std::str;
use super::super::common;

#[no_mangle]
extern "C" fn wssclient_new<'a>() -> *mut WSClient<'a, *mut c_void> {
    // Box doesn't return a Result type, that the reason to use layout, to check if the system
    // gave me memory to store the client.
    let size = mem::size_of::<WSClient<*mut c_void>>();
    let aling = std::mem::align_of::<WSClient<*mut c_void>>();
    let layout = Layout::from_size_align(size, aling);

    if layout.is_err() {
        return std::ptr::null_mut();
    }

    let ptr = unsafe { alloc(layout.unwrap()) };
    let client = WSClient::<*mut c_void>::new();

    unsafe {
        ptr::copy_nonoverlapping(&client, ptr as *mut WSClient<*mut c_void>, 1);
    }

    ptr as *mut WSClient<*mut c_void>
}

#[no_mangle]
unsafe extern "C" fn wssclient_init<'a> (
    client: *mut WSClient<'a, *mut c_void>,
    host: *const c_char,
    port: u16,
    path: *const c_char,
    callback: *mut c_void,
) {
    let host = str::from_utf8(CStr::from_ptr(host).to_bytes()).unwrap();
    let path = str::from_utf8(CStr::from_ptr(path).to_bytes()).unwrap();

    let callback: fn(&mut WSClient<'a, *mut c_void>, &RWSEvent, Option<*mut c_void>) = mem::transmute(callback);
    let config = Config { callback: Some(callback), data: None, protocols: None };
    
    let client = &mut *client;

    client.init(host, port, path, Some(config));
}

#[no_mangle]
unsafe extern "C" fn wssclient_loop<'a>(client: *mut WSClient<'a, *mut c_void>) -> common::WSStatus {
    let client = &mut *client;

    match client.event_loop() {
        Ok(_) => {}
        Err(e) => {
            return common::rust_error_to_c_error(e);
        } 
    }

    common::WSStatus::OK
}

#[no_mangle]
unsafe extern "C" fn wssclient_send<'a>(client: *mut WSClient<'a, *mut c_void>, message: *const c_char) {
    let msg = str::from_utf8(CStr::from_ptr(message).to_bytes()).unwrap();
    let client = &mut *client;
    client.send(msg);
}

#[no_mangle]
extern "C" fn wssclient_drop<'a>(client: *mut WSClient<'a, *mut c_void>) {
    // Create a box from the raw pointer, at the end of the function the client will be dropped and the memory will be free.
    unsafe {
        let _c = Box::from_raw(client);
    }
}