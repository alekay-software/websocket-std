use std::collections::HashMap;
use super::commons::{END_LINE};
use super::super::core::traits::Parse;

#[allow(dead_code)]
pub struct Response {
    version: String,
    status_code: u16,
    status_text: String,
    headers: Option<HashMap<String, String>>,
    body: Option<Vec<u8>>
}

impl Response {
    pub fn new(version: String, status_code: u16, status_text: String, headers: Option<HashMap<String, String>>, body: Option<Vec<u8>>) -> Self {
        Response {version, status_code, headers, status_text, body }
    }

    pub fn get_status_code(&self) -> u16 {
        return self.status_code;
    }
}

// TODO: Parse the rest of the response
// If an error is produced parsing the response then the status code will be -1
impl Parse for Response {
    fn parse(bytes: &[u8]) -> Self {
        let bytes = String::from_utf8(bytes.to_vec());

        if bytes.is_err() { 
            return Response::new(String::new(), 0, String::new(), None, None); 
        }

        let bytes = bytes.unwrap();
        let lines:Vec<&str> = bytes.split(END_LINE).collect();

        let response_info: Vec<&str> = lines[0].split(" ").collect();
        let version = response_info[0].to_string();
        let status_code = response_info[1].parse::<u16>();

        if status_code.is_err() {
            return Response::new(String::new(), 0, String::new(), None, None);   
        }

        let status_code = status_code.unwrap();

        let status_text = response_info[2].to_string();

        return Response { version, status_code, status_text, headers: None, body: None };

    }
}