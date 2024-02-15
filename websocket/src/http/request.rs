// Simple http parser to send handshake and read in a better way the response from the handshake

use super::super::core::traits::Serialize;
use std::collections::HashMap;
use super::commons::{Headers, END_LINE};

// Returns the string value from the method
fn method_to_string(method: &Method) -> String {
    match method {
        Method::GET => String::from("GET")
    }
}

pub enum Method {
    GET
}

pub struct Request {
    method: Method,
    path: String,
    version: String,
    headers: Headers<String>,
}

impl Request  {
    pub fn new(method: Method, path: &str, version: &str, headers: Option<Headers<String>>) -> Self {
        let h = match headers {
            Some(map) => map,
            None => HashMap::new()
        };
        
        Request { method, path: path.to_string(), version: version.to_string(), headers: h }
    }
}

impl Serialize for Request {

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