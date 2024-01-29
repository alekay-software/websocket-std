use bitflags::bitflags;

bitflags! {
    #[allow(non_camel_case_types)]
    #[derive(PartialEq, Eq, Clone, Copy)]
    pub struct WSStatus: u16 {
        const NORMAL_CLOSURE = 1000;                                // Close the connection, no error
        const GOING_AWAY = 1001;                                    // Close the connection, no error
        const PROTOCOL_ERROR = 1002;                                // Close the connection, error
        const TYPE_DATA_NOT_ACCEPTABLE = 1003;                      // Close the connection, error
        const RESERVED_1004 = 1004;                                 // RFC6455 | Reserved. The specific meaning might be defined in the future
        const EXPECTED_STATUS_CODE = 1005;                          // MUST NOT be set as a status code in Close control frame by an endpoint. It is designated for use in applications expecting a status code to indicate that no status code was actually present. 
        const CONNECTION_CLOSE_ABNORMALLY = 1006;                   // reserved value and MUST NOT be set as a status code in a Close control frame by an endpoint 
        const INCONSISTENT_DATA_TYPE_INSIDE_MESSAGE = 1007;         // Close the connection, error  For example, not utf-8 data in a text message
        const POLICY_VIOLATION = 1008;                              // Close, error
        const MESSAGE_TO_BIG = 1009;                                // Close error
        const EXPECTED_EXTENSION_NEGOTIATION_WITH_SERVER = 1010;    // Close, error (Only for clients) The list of extensions needed should appear in the reason part of the frame
        const UNEXPECTED_CONDITION_ENCOUNTERED = 1011;              // CLOSE: indicates that a server is terminating the connection because it encountered an unexpected condition that prevented it from fulfilling the request.
        const TLS_HANDSHAKE_ERROR = 1015;                           // (Only  for clients) the server certificate can't be verified
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