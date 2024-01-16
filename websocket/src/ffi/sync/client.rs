use crate::result::WebSocketError;

use super::super::super::sync::client::{Config, WSEvent as RWSEvent, WSData, WSClient, Reason};
use std::ffi::{c_void, c_char, CStr, c_int, CString};
use std::alloc::{alloc, Layout};
use std::mem;
use std::ptr;
use std::str;

#[repr(C)]
enum WSEvent {
    ON_CONNECT,
    ON_TEXT,
    ON_CLOSE
}

#[repr(C)]
struct WSEvent_t {
    event: WSEvent,
    value: *const c_void
}

#[repr(C)]
enum WSReason {
    SERVER_CLOSED,
    CLIENT_CLOSED
}

#[repr(C)]
struct WSReason_t {
    reason: WSReason,
    status: u16
}

#[repr(C)]
#[derive(Debug, Clone)]
pub enum WSErrorKind { 
    ProtocolError,
    DataFrameError,
    SocketError,
    NoDataAvailable,
    IOError,
    Utf8Error,
    TryFromSliceError,
    ConnectionClose,
    HandShakeError,
    Other,
}

#[repr(C)]
pub struct WebSocketError_t {
    kind: WSErrorKind,
    msg: CString 
}

impl WebSocketError_t {
    fn new(kind: WSErrorKind, msg: Option<&str>) -> *mut WebSocketError_t {
        let size = mem::size_of::<*mut WebSocketError_t>();
        let aling = std::mem::align_of::<*mut WebSocketError_t>();
        let layout = Layout::from_size_align(size, aling);
        
        let ptr = unsafe { alloc(layout.unwrap()) };
        // let mut error_msg: *const c_char = std::ptr::null();

        let cstring = CString::new(msg.unwrap()).unwrap();
        std::mem::forget(cstring.clone());
        // if msg.is_some() {
        //     // error_msg = cstring.as_ptr();
        //     // println!("Error new: {}", &msg.unwrap());
        //     // std::mem::forget(cstring);
        // }
        
        let ws_error = WebSocketError_t { kind, msg: cstring };
        
        unsafe {
            ptr::copy_nonoverlapping(&ws_error, ptr as *mut WebSocketError_t, 1);
        }
 
        ptr as *mut WebSocketError_t 
    }
} 

fn rust_error_to_c_error(err: WebSocketError) -> *mut WebSocketError_t {
    match err {
        WebSocketError::ProtocolError(msg) => WebSocketError_t::new(WSErrorKind::ProtocolError, Some(&msg)),  
        WebSocketError::DataFrameError(msg) => WebSocketError_t::new(WSErrorKind::DataFrameError, Some(&msg)),
        WebSocketError::SocketError(msg) => WebSocketError_t::new(WSErrorKind::SocketError, Some(&msg)),
        WebSocketError::NoDataAvailable => WebSocketError_t::new(WSErrorKind::NoDataAvailable, None),
        WebSocketError::IOError(io_error) => WebSocketError_t::new(WSErrorKind::IOError, Some(&io_error.to_string())),
        WebSocketError::Utf8Error(utf8_error) => WebSocketError_t::new(WSErrorKind::Utf8Error, Some(&utf8_error.to_string())),
        WebSocketError::TryFromSliceError(arr_error)  => WebSocketError_t::new(WSErrorKind::TryFromSliceError, Some(&arr_error.to_string())),
        WebSocketError::ConnectionClose(msg) => WebSocketError_t::new(WSErrorKind::ConnectionClose, Some(&msg)),
        WebSocketError::HandShakeError(msg) => WebSocketError_t::new(WSErrorKind::HandShakeError, Some(&msg)),
        WebSocketError::Other(err) => WebSocketError_t::new(WSErrorKind::Other, Some(&err.to_string())),
    }
}

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
unsafe extern "C" fn wssclient_init<'a>(
    client: *mut WSClient<'a, *mut c_void>,
    host: *const c_char,
    port: u16,
    path: *const c_char,
    callback: *mut c_void,
) {
    let host = str::from_utf8(CStr::from_ptr(host).to_bytes()).unwrap();
    let path = str::from_utf8(CStr::from_ptr(path).to_bytes()).unwrap();

    let callback: fn(&mut WSClient<'a, *mut c_void>, &RWSEvent, Option<WSData<*mut c_void>>) = mem::transmute(callback);
    let config = Config { callback: Some(callback), data: None, protocols: None };
    
    let client = &mut *client;
    client.init(host, port, path, Some(config)).unwrap();
    println!("Client init");
}

#[no_mangle]
unsafe extern "C" fn wssclient_loop<'a>(client: *mut WSClient<'a, *mut c_void>) -> *const WebSocketError_t {
    let err = std::ptr::null();

    let client = &mut *client;

    match client.event_loop() {
        Ok(_) => {}
        Err(e) => {
            //println!("Error -> {}", e);
            return rust_error_to_c_error(e);
            //let error = rust_error_to_c_error(e);
            //println!("Error number: {}", (*error).kind.clone() as i32);
        }
    }

    return err;
}

#[no_mangle]
unsafe extern "C" fn wssclient_send<'a>(client: *mut WSClient<'a, *mut c_void>, message: *const c_char) -> c_int {
    let mut status = 1;
    let msg = str::from_utf8(CStr::from_ptr(message).to_bytes()).unwrap();

    let client = &mut *client;
    match client.send(msg) {
        Ok(_) => {}
        Err(e) => {
            println!("Error {}", e);
            status = 0
        }
    }

    return status;
}

#[no_mangle]
extern "C" fn wssclient_drop<'a>(client: *mut WSClient<'a, *mut c_void>) {
    // Create a box from the raw pointer, at the end of the function the client will be dropped and the memory will be free.
    unsafe {
        let _c = Box::from_raw(client);
    }
}

#[no_mangle]
unsafe extern "C" fn fromRustEvent(event: &RWSEvent) -> WSEvent_t {
    match event {
        RWSEvent::ON_CONNECT(msg) => {
            let mut value = ptr::null_mut();
            if let Some(m) = msg {
                println!("[RUST] M: {}", m);
                let m = m.replace('\0', "");
                let m = m.trim();
                let c_str = CString::new(m).map_err(|err| {
                    eprintln!("Error converting to CString: {err}");
                }).unwrap();
                value = c_str.into_raw();
            }
            WSEvent_t { event: WSEvent::ON_CONNECT, value: value as *const c_void }
        }
        , 
        RWSEvent::ON_TEXT(msg) => {
            let c_str = CString::new(msg.clone()).unwrap();
            WSEvent_t { event: WSEvent::ON_TEXT, value: c_str.into_raw() as *const c_void }
        },
        RWSEvent::ON_CLOSE(reason) => {
            let (reason, status) = match reason {
                Reason::SERVER_CLOSE(status) => (WSReason::SERVER_CLOSED, status.clone()),   
                Reason::CLIENT_CLOSE(status) => (WSReason::CLIENT_CLOSED, status.clone())
            };
            let reason = WSReason_t { reason, status };
            let reason = Box::into_raw(Box::new(reason));
            WSEvent_t { event: WSEvent::ON_CLOSE, value: reason as *const c_void } 
        }
    }
}