use std::io::Read;

use crate::core::traits::Serialize;
use super::header::*;
use super::mask::gen_mask;
// -------------------------------------------------------------------------------------------------------- //
//                                               header.rs
// -------------------------------------------------------------------------------------------------------- //
fn equals(b1: Vec<u8>, b2: Vec<u8>) -> bool {
    if b1.len() != b2.len() { return false }

    let mut i = 0;
    while i < b1.len() {
        let a = b1.get(i).unwrap();
        let b = b2.get(i).unwrap();

        if a != b { return false }

        i += 1;
    }

    return true;
}

// ------------------- Payload less than 126 bytes ------------------- //

#[test]
fn serialize_header_with_no_mask_data_0() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let mask = None;
    let payload_len = 0;
    let header = Header::new(flag, opcode, mask, payload_len);

    let expected_result: Vec<u8> = [0x81, 0x00].to_vec();
    
    assert!(equals(header.serialize(), expected_result));

}  

#[test]
fn serialize_header_with_no_mask_data_less_than_126() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let mask = None;
    let payload_len = 10;
    let header = Header::new(flag, opcode, mask, payload_len);

    let expected_result: Vec<u8> = [0x81, 0x0A].to_vec();
    
    assert!(equals(header.serialize(), expected_result));
}

#[test]
fn serialize_header_with_mask_data_0() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let mask = gen_mask();
    let payload_len = 0;
    let header = Header::new(flag, opcode, Some(mask), payload_len);

    let expected_result: Vec<u8> = [0x81, 0x80, mask[0], mask[1], mask[2], mask[3]].to_vec();
    
    assert!(equals(header.serialize(), expected_result));

}  

#[test]
fn serialize_header_with_mask_data_less_than_126() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let mask = gen_mask();
    let payload_len = 10;
    let header = Header::new(flag, opcode, Some(mask), payload_len);

    let expected_result: Vec<u8> = [0x81, 0x8A, mask[0], mask[1], mask[2], mask[3]].to_vec();
    
    assert!(equals(header.serialize(), expected_result));
}  

// ------------------- Payload greather or equal than 126 bytes and less than 65535 (2ˆ16 - 1) ------------------- //

#[test]
fn serialize_header_with_no_mask_data_equal_126() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let payload_len = 126;
    let header = Header::new(flag, opcode, None, payload_len);

    let expected_result: Vec<u8> = [0x81, 0x7E, 0x00, 0x7E].to_vec();
    
    assert!(equals(header.serialize(), expected_result));

}  

#[test]
fn serialize_header_with_no_mask_data_greather_than_126_less_than_65535() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let payload_len = 65530;
    let header = Header::new(flag, opcode, None, payload_len);

    let expected_result: Vec<u8> = [0x81, 0x7E, 0xFF, 0xFA].to_vec();
    
    assert!(equals(header.serialize(), expected_result));
} 

#[test]
fn serialize_header_with_no_mask_data_equal_65535() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let payload_len = 65535;
    let header = Header::new(flag, opcode, None, payload_len);

    let expected_result: Vec<u8> = [0x81, 0x7E, 0xFF, 0xFF].to_vec();
    
    assert!(equals(header.serialize(), expected_result));
} 

#[test]
fn serialize_header_with_mask_data_equal_126() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let mask = gen_mask();
    let payload_len = 126;
    let header = Header::new(flag, opcode, Some(mask), payload_len);

    let expected_result: Vec<u8> = [0x81, 0xFE, 0x00, 0x7E, mask[0], mask[1], mask[2], mask[3]].to_vec();
    
    assert!(equals(header.serialize(), expected_result));
}  

#[test]
fn serialize_header_with_mask_data_greather_than_126_less_than_65535() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let mask = gen_mask();
    let payload_len = 65530;
    let header = Header::new(flag, opcode, Some(mask), payload_len);

    let expected_result: Vec<u8> = [0x81, 0xFE, 0xFF, 0xFA, mask[0], mask[1], mask[2], mask[3]].to_vec();
    
    assert!(equals(header.serialize(), expected_result));
}  

#[test]
fn serialize_header_with_mask_data_equal_65535() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let mask = gen_mask();
    let payload_len = 65535;
    let header = Header::new(flag, opcode, Some(mask), payload_len);

    let expected_result: Vec<u8> = [0x81, 0xFE, 0xFF, 0xFF, mask[0], mask[1], mask[2], mask[3]].to_vec();
    
    assert!(equals(header.serialize(), expected_result));
}  

// ------------------- Payload greather than 65535 (2ˆ16 - 1) ------------------- //

#[test]
fn serialize_header_with_no_mask_data_equal_65536() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let payload_len = 65536;
    let header = Header::new(flag, opcode, None, payload_len);

    let expected_result: Vec<u8> = [0x81, 0x7F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00].to_vec();
    
    assert!(equals(header.serialize(), expected_result));
}

#[test]
fn serialize_header_with_no_mask_data_greather_than_65536() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let payload_len = 1<<60;
    let header = Header::new(flag, opcode, None, payload_len);

    let expected_result: Vec<u8> = [0x81, 0x7F, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00].to_vec();
    
    assert!(equals(header.serialize(), expected_result));
} 

#[test]
fn serialize_header_with_mask_data_equal_65536() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let payload_len = 65536;
    let mask = gen_mask();
    let header = Header::new(flag, opcode, Some(mask), payload_len);

    let expected_result: Vec<u8> = [0x81, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, mask[0], mask[1], mask[2], mask[3]].to_vec();
    
    assert!(equals(header.serialize(), expected_result));
}

#[test]
fn serialize_header_with_mask_data_greather_than_65536() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let payload_len = 1<<60;
    let mask = gen_mask();
    let header = Header::new(flag, opcode, Some(mask), payload_len);

    let expected_result: Vec<u8> = [0x81, 0xFF, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, mask[0], mask[1], mask[2], mask[3]].to_vec();
    
    assert!(equals(header.serialize(), expected_result));
} 

#[test]
fn serialize_header_with_no_mask_data_equal_to_max_value() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let payload_len = 18446744073709551615; // (2ˆ64) - 1
    let header = Header::new(flag, opcode, None, payload_len);

    let expected_result: Vec<u8> = [0x81, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF].to_vec();
    
    assert!(equals(header.serialize(), expected_result));
} 

#[test]
fn serialize_header_with_mask_data_equal_to_max_value() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let payload_len = 18446744073709551615;
    let mask = gen_mask();
    let header = Header::new(flag, opcode, Some(mask), payload_len);

    let expected_result: Vec<u8> = [0x81, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, mask[0], mask[1], mask[2], mask[3]].to_vec();
    
    assert!(equals(header.serialize(), expected_result));
} 

use super::frame::*;
use super::mask::Mask;
// -------------------------------------------------------------------------------------------------------- //
//                                               frame.rs
// -------------------------------------------------------------------------------------------------------- //

// ------------------- DataFrames ------------------- //
fn apply_mask(data: &[u8], mask: &Mask) -> Vec<u8> {
    let mut masked_data = Vec::new();

    let mut i = 0;
    for byte in data {
        masked_data.push(byte ^ mask[i]);
        i += 1;
        if i >= 4 { i = 0; }
    }

    return masked_data;
}

#[test]
fn serialize_data_masked() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let data = String::from("hello");
    let mask = gen_mask();
    let header = Header::new(flag, opcode, Some(mask), data.len() as u64);

    let dataframe = DataFrame::new(header, data.as_bytes().to_vec());
    let serialized_frame = dataframe.serialize();
    let serialized_data = serialized_frame[serialized_frame.len() - data.len()..serialized_frame.len()].to_vec();
    let _d = apply_mask(serialized_data.as_slice(), &mask);
    let _d = String::from_utf8(_d).unwrap();

    let expected_data = apply_mask(data.as_bytes(), &mask);
    let mut expected_frame = Vec::new();
    expected_frame.extend_from_slice(&[0x81, 0x85]);
    expected_frame.extend_from_slice(&mask);
    expected_frame.extend_from_slice(expected_data.as_slice());

    assert!(equals(serialized_data, expected_data));
    assert!(equals(serialized_frame, expected_frame));
    assert_eq!(_d, data);
}

#[test]
fn serialize_data_unmasked() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::TEXT;
    let data = String::from("hello");
    let header = Header::new(flag, opcode, None, data.len() as u64);

    let dataframe = DataFrame::new(header, data.as_bytes().to_vec());
    let serialized_frame = dataframe.serialize();
    let serialized_data = serialized_frame[serialized_frame.len() - data.len()..serialized_frame.len()].to_vec();

    let mut expected_frame = Vec::new();
    expected_frame.extend_from_slice(&[0x81, 0x05]);
    expected_frame.extend_from_slice(data.as_bytes());

    assert!(equals(serialized_data, data.as_bytes().to_vec()));
    assert!(equals(serialized_frame, expected_frame));
}

// ------------------- Control Frames ------------------- //
use super::super::core::binary::bytes_to_u16;

#[test]
fn serialize_controlframe_unmasked_without_status_code() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::PING;
    let data = String::from("hello");
    let header = Header::new(flag, opcode, None, data.len() as u64);

    let dataframe = ControlFrame::new(header, None, data.as_bytes().to_vec());
    let serialized_frame = dataframe.serialize();
    let serialized_data = serialized_frame[serialized_frame.len() - data.len()..serialized_frame.len()].to_vec();

    let mut expected_frame = Vec::new();
    expected_frame.extend_from_slice(&[0x89, 0x05]);
    expected_frame.extend_from_slice(data.as_bytes());

    assert!(equals(serialized_data, data.as_bytes().to_vec()));
    assert!(equals(serialized_frame, expected_frame));
}

#[test]
fn serialize_controlframe_unmasked_with_status_code() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::PING;
    let data = String::from("hello");
    let status_code = 1000;
    let header = Header::new(flag, opcode, None, (data.len() + 2) as u64);

    let dataframe = ControlFrame::new(header, Some(status_code), data.as_bytes().to_vec());
    let serialized_frame = dataframe.serialize();
    let serialized_data = serialized_frame[serialized_frame.len() - data.len()..serialized_frame.len()].to_vec();

    let mut expected_frame = Vec::new();
    expected_frame.extend_from_slice(&[0x89, 0x07, 0x03, 0xE8]);
    expected_frame.extend_from_slice(data.as_bytes());

    assert!(equals(serialized_data, data.as_bytes().to_vec()));
    assert!(equals(serialized_frame, expected_frame));
}


#[test]
fn serialize_controlframe_masked_without_status_code() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::PONG;
    let data = String::from("hello");
    let mask = gen_mask();
    let header = Header::new(flag, opcode, Some(mask), data.len() as u64);

    let dataframe = ControlFrame::new(header, None, data.as_bytes().to_vec());
    let serialized_frame = dataframe.serialize();
    let serialized_data = serialized_frame[serialized_frame.len() - data.len()..serialized_frame.len()].to_vec();
    let _d = apply_mask(serialized_data.as_slice(), &mask);
    let _d = String::from_utf8(_d).unwrap();

    let expected_data = apply_mask(data.as_bytes(), &mask);
    let mut expected_frame = Vec::new();
    expected_frame.extend_from_slice(&[0x8A, 0x85]);
    expected_frame.extend_from_slice(&mask);
    expected_frame.extend_from_slice(expected_data.as_slice());

    assert!(equals(serialized_data, expected_data));
    assert!(equals(serialized_frame, expected_frame));
    assert_eq!(_d, data);
}

#[test]
fn serialize_controlframe_masked_with_status_code() {
    let flag = FLAG::FIN;
    let opcode = OPCODE::PONG;
    let data = String::from("hello");
    let mask = gen_mask();
    let status_code = 1000;
    let header = Header::new(flag, opcode, Some(mask), (data.len() + 2) as u64);

    let dataframe = ControlFrame::new(header, Some(status_code), data.as_bytes().to_vec());
    let serialized_frame = dataframe.serialize();
    let serialized_data = serialized_frame[serialized_frame.len() - data.len() - 2..serialized_frame.len()].to_vec();

    let mut _data = Vec::new();
    _data.extend_from_slice(&status_code.to_be_bytes());
    _data.extend_from_slice(&data.as_bytes());
    let expected_data = apply_mask(_data.as_slice(), &mask);
    let mut expected_frame = Vec::new();
    expected_frame.extend_from_slice(&[0x8A, 0x87]);
    expected_frame.extend_from_slice(&mask);
    expected_frame.extend_from_slice(expected_data.as_slice());

    assert!(equals(serialized_data.clone(), expected_data));
    assert!(equals(serialized_frame, expected_frame));
    
    let serialized_data = apply_mask(&serialized_data, &mask);
    let _status = bytes_to_u16(&serialized_data.as_slice()[0..2]).unwrap();
    let _data = String::from_utf8((&serialized_data[2..serialized_data.len()]).to_vec()).unwrap();

    assert_eq!(status_code, _status);
    assert_eq!(data, _data);
}