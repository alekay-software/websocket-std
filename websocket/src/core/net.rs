use std::io::{BufRead, Error, self};

/*
Reads an entire tcp package from the Reader.
When using TCPStreams, to get the full message multiples calls to read method
has to be done because the reader has a buffer capacity, each time the fill_buf function is called
the buffer is going to fill with all the data. If the amount of data is greatest than the capacity of 
the buffer, then you should call fill_buff multiples times in order to read the entire data.
 */
pub fn read_entire_tcp_package(reader: &mut dyn BufRead) -> Result<Vec<u8>, Error> {
    let mut tcp_package: Vec<u8> = Vec::new();

    loop {
        match reader.fill_buf() {
            Ok(data) => {
                let amount = data.len();
                tcp_package.extend_from_slice(data);
                reader.consume(amount);
            },
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock { break }
                else { return Err(e) }
            }
        }
    }

    return Ok(tcp_package);
}

#[cfg(test)]
mod test {}