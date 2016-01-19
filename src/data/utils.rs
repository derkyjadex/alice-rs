extern crate byteorder;

use std::io;

pub fn map_to_io(err: byteorder::Error) -> io::Error {
    match err {
        byteorder::Error::Io(err) => err,
        byteorder::Error::UnexpectedEOF =>
            io::Error::new(io::ErrorKind::Other, "unexpected EOF"),
    }
}
