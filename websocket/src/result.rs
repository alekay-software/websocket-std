// Copyright (c) 2014 Rust Websockets Developers

// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:

// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std::str::Utf8Error;
use std::fmt;
use std::convert::From;
use std::error::Error;
use std::io;

// Define type for WebSocketStdResult
pub type WebSocketResult<T> = Result<T, WebSocketError>;

// Represents a WebSocket error
#[derive(Debug)]
pub enum WebSocketError {
	/// A WebSocket protocol error
	ProtocolError(&'static str),
	/// Invalid WebSocket data frame error
	DataFrameError(&'static str),
	// Socket error
	SocketError(&'static str),
	/// No data available
	NoDataAvailable,
	// IO Error
	IOError(io::Error),
	/// A UTF-8 error
	Utf8Error(Utf8Error),
	/// Custom String Error
	Custom(String),
    /// Other error from higher-level crate, for downcasting
	Other(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl fmt::Display for WebSocketError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.write_str("WebSocketError: ")?;
		match self {
			WebSocketError::ProtocolError(_) => fmt.write_str("WebSocket protocol error"),
			WebSocketError::DataFrameError(_) => fmt.write_str("WebSocket data frame error"),
			WebSocketError::SocketError(msg) => fmt.write_str(msg),
			WebSocketError::NoDataAvailable => fmt.write_str("No data available"),
			WebSocketError::IOError(_) => fmt.write_str("I/O failure"),
			WebSocketError::Utf8Error(_) => fmt.write_str("UTF-8 failure"),
			WebSocketError::Custom(s) => fmt.write_str(s.as_str()),
			WebSocketError::Other(x) => x.fmt(fmt),
		}
	}
}

impl Error for WebSocketError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match *self {
			WebSocketError::Utf8Error(ref error) => Some(error),
			WebSocketError::Other(ref error) => error.source(),
			_ => None,
		}
	}
}

impl From<Utf8Error> for WebSocketError {
    fn from(err: Utf8Error) -> Self {
        WebSocketError::Utf8Error(err)
    }
}

impl From<io::Error> for WebSocketError {
	fn from(err: io::Error) -> Self {
		match err.kind() {
			io::ErrorKind::UnexpectedEof => WebSocketError::NoDataAvailable,
			_ => WebSocketError::IOError(err)
		}
	}
}