use std::collections::HashMap;

pub const END_LINE: &str = "\r\n";
pub type Headers<'a> = HashMap<&'a str, &'a str>;