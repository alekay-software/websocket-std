use bitflags::bitflags;

use crate::result::WebSocketResult;

use super::frame::FrameKind;

bitflags! {
    #[allow(non_camel_case_types)]
    #[derive(PartialEq, Eq)]
    pub struct WSStatus: u16 {
        const NORMAL_CLOSURE = 1000;            // Close the connection, no error
        const GOING_AWAY = 1001;                // Close the connection, no error
        const PROTOCOL_ERROR = 1002;            // Close the connection, error
        const TYPE_DATA_NOT_ACCEPTABLE = 1003;  // Close the connection, error
        const RESERVED_1004 = 1004;  
        const EXPECTED_STATUS_CODE = 1005;  
        const CONNECTION_CLOSE_ABNORMALLY = 1006; 
        const INCONSISTENT_DATA_TYPE_INSIDE_MESSAGE = 1007; // Close the connection, error  For example, not utf-8 data in a text message
        const POLICY_VIOLATION = 1008; // Close, error
        const MESSAGE_TO_BIG = 1009; // Close error
        const EXPECTED_EXTENSION_NEGOTIATION_WITH_SERVER = 1010; // Close, error (Only for clients) The list of extensions needed should appear in the reason part of the frame
        const UNEXPECTED_CONDITION_ENCOUNTERED = 1011;
        const TLS_HANDSHAKE_ERROR = 1015; // (Only  for clients) the server certificate can't be verified
    }
}

// Returns if there was an error and if the connection should be closed
pub fn evaulate_status_code(status: WSStatus) -> (bool, bool) {
    let mut is_error = true;
    let mut should_close = true;

    match status {
        WSStatus::NORMAL_CLOSURE => is_error = false,
        WSStatus::GOING_AWAY => is_error = false,
        WSStatus::RESERVED_1004 => { is_error = false; should_close = false; } // TODO: should_close true or false?
        _ => {}
    }

    return (is_error, should_close);
}