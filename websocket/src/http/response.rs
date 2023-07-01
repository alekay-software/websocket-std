use super::super::core::traits::Serialize;
use super::super::core::traits::Parse;

pub struct Response<'a> {
    version: &'a str,
    status_code: u16,
    status_text: &'a str
}

// impl<'a> Parse for Response<'a> {
//     fn parse(bytes: &Vec<u8>) -> Self {
//         let version = 
//     }
// }