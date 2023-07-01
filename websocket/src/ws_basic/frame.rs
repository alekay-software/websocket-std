use super::{header::{Header}};
use super::super::core::traits::Serialize;

pub trait Frame {
    fn get_data(&self) -> &[u8];
    fn get_header(&self) -> &Header;

    fn serialize(&self) -> Vec<u8> {
        let mut serialized_data = vec![];
        
        serialized_data.extend(self.get_header().serialize());

        match self.get_header().get_mask() {
            // Apply mask to data
            Some(mask) => {
                let mut i = 0;
                for &byte in self.get_data() {
                    serialized_data.push(byte ^ mask[i]);
                    i += 1;
                    if i >= mask.len() { i = 0 };
                }
            },
            // Just insert App data without mask
            None => serialized_data.extend(self.get_data())
        }

        return serialized_data; 
    }
}

pub struct DataFrame {
    header: Header,
    data: Vec<u8>
}

impl DataFrame {
    pub fn new(header: Header, data: Vec<u8>) -> Self {
        DataFrame { header, data }
    }
}

impl Frame for DataFrame {
    fn get_data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn get_header(&self) -> &Header {
        &self.header
    }
}


pub struct ControlFrame {
    header: Header,
    data: Vec<u8>,
}

impl ControlFrame {
    pub fn new(header: Header, status_code: u16, data: Vec<u8>) -> Self {
        // Data len should be less than 123 (2-bytes of status code + data <= 125) Where I shold veritfy this condition?
        let mut merge_data = vec![];
        merge_data.extend(status_code.to_be_bytes());
        merge_data.extend(data);
        ControlFrame { header, data: merge_data }
    }
}

impl Frame for ControlFrame {
    fn get_data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn get_header(&self) -> &Header {
        &self.header
    }
}