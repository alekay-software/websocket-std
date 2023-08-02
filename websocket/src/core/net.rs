use std::io::{Read, Error, self, ErrorKind};

/*
Reads an entire tcp package from the Reader.
When using TCPStreams, to get the full message multiples calls to read method
has to be done because the reader has a buffer capacity, each time the fill_buf function is called
the buffer is going to fill with all the data. If the amount of data is greatest than the capacity of 
the buffer, then you should call fill_buff multiples times in order to read the entire data.
 */
pub fn read_into_buffer(reader: &mut dyn Read, buf: &mut [u8]) -> Result<usize, Error> {
    match reader.read(buf) {
        Ok(amount) => {
            // Reached end of file (error in the connection)
            if amount <= 0 {
                return Err(Error::new(io::ErrorKind::ConnectionReset, "Connection reset by peer"));
            } else {
                return Ok(amount);
            }
        },
        Err(e) => {
            if e.kind() == ErrorKind::WouldBlock { return Ok(0) }
            return Err(e);
        }
    }
}

#[cfg(test)]
mod test {}