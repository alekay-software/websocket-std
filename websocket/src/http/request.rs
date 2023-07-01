// Simple http parser to send handshake and read in a better way the response from the handshake

use super::super::core::traits::Serialize;
use std::collections::HashMap;

// Returns the string value from the method
fn method_to_string(method: &Method) -> String {
    match method {
        Method::GET => String::from("GET")
    }
}

const END_LINE: &str = "\r\n";
type Headers<'a> = HashMap<&'a str, &'a str>;

pub enum Method {
    GET
}

pub struct Request<'a> {
    method: Method,
    path: &'a str,
    version: &'a str,
    headers: Headers<'a>,
}

impl<'a> Request<'a> {
    pub fn new(method: Method, path: &'a str, version: &'a str, headers: Option<Headers<'a>>) -> Self {
        let h = match headers {
            Some(map) => map,
            None => HashMap::<&str, &str>::new()
        };
        
        Request { method, path, version, headers: h }
    }

    pub fn set_header(&mut self, key: &'a str, value: &'a str) {
        self.headers.insert(key, value);
    }
}

impl<'a> Serialize for Request<'a> {

    fn serialize(&self) -> Vec<u8> {
        let mut data = vec![];

        // method, path and version in the first line
        data.extend(method_to_string(&self.method).as_bytes());
        data.extend(" ".as_bytes());
        data.extend(self.path.as_bytes());
        data.extend(" ".as_bytes());
        data.extend(self.version.as_bytes());
        data.extend(END_LINE.as_bytes());


        for (key, value) in (&self.headers).into_iter() {
            data.extend(key.as_bytes());
            data.extend(": ".as_bytes());
            data.extend(value.as_bytes());
            data.extend(END_LINE.as_bytes());
        }

        data.extend(END_LINE.as_bytes());

        return data;
    }
}