use std::io::{self, Read};
use byteorder::{LittleEndian, BigEndian, ReadBytesExt};
use super::{Value, Tag, Vec2, Vec3, Vec4, Box2, Token, Reader};

use data::utils::{map_to_io};

pub struct BinaryReader<R> {
    input: R
}

fn invalid_token<T>() -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid token"))
}

impl<R: Read> BinaryReader<R> {
    pub fn new(input: R) -> BinaryReader<R> {
        BinaryReader { input: input }
    }

    pub fn into_inner(self) -> R {
        self.input
    }

    fn read_value(&mut self, t: u8) -> io::Result<Value> {
        match t {
            0x00 => self.read_bool().map(Value::Bool),
            0x01 => self.read_int().map(Value::Int),
            0x02 => self.read_double().map(Value::Double),
            0x03 => self.read_vec2().map(Value::Vec2),
            0x04 => self.read_vec3().map(Value::Vec3),
            0x05 => self.read_vec4().map(Value::Vec4),
            0x06 => self.read_box2().map(Value::Box2),
            0x07 => self.read_string().map(Value::String),
            0x08 => self.read_blob().map(Value::Blob),
            0xee => self.read_tag().map(Value::Tag),
            _ => invalid_token(),
        }
    }

    fn read_array(&mut self, t:u8) -> io::Result<Value> {
        match t {
            0x80 => self.read_array_values(BinaryReader::read_bool)
                .map(Value::BoolArray),
            0x81 => self.read_array_values(BinaryReader::read_int)
                .map(Value::IntArray),
            0x82 => self.read_array_values(BinaryReader::read_double)
                .map(Value::DoubleArray),
            0x83 => self.read_array_values(BinaryReader::read_vec2)
                .map(Value::Vec2Array),
            0x84 => self.read_array_values(BinaryReader::read_vec3)
                .map(Value::Vec3Array),
            0x85 => self.read_array_values(BinaryReader::read_vec4)
                .map(Value::Vec4Array),
            0x86 => self.read_array_values(BinaryReader::read_box2)
                .map(Value::Box2Array),
            _ => invalid_token(),
        }
    }

    fn read_tag(&mut self) -> io::Result<Tag> {
        self.input.read_u32::<BigEndian>()
            .map_err(map_to_io)
    }

    fn read_bool(&mut self) -> io::Result<bool> {
        let mut buffer = [0; 1];
        try!(self.input.read_exact(&mut buffer));

        Ok(buffer[0] != 0)
    }

    fn read_uint(&mut self) -> io::Result<u64> {
        let mut buffer = [0; 1];
        let mut length = 0;
        let mut result = 0u64;

        loop {
            if length > 10 {
                return invalid_token();
            }

            try!(self.input.read_exact(&mut buffer));

            result |= (u64::from(buffer[0]) & 0x7f) << (length * 7);
            length += 1;

            if buffer[0] & 0x80 == 0 {
                break;
            }
        }

        Ok(result)
    }

    fn read_sint(&mut self) -> io::Result<i64> {
        let uvalue = try!(self.read_uint());

        Ok(if uvalue & 1 != 0 {
            !(uvalue >> 1)
        } else {
            uvalue >> 1
        } as i64)
    }

    fn read_int(&mut self) -> io::Result<i32> {
        let value = try!(self.read_sint());

        if value > i32::max_value() as i64 ||
            value < i32::min_value() as i64 {
                return invalid_token()
            }

        Ok(value as i32)
    }

    fn read_double(&mut self) -> io::Result<f64> {
        self.input.read_f64::<LittleEndian>()
            .map_err(map_to_io)
    }

    fn read_vec2(&mut self) -> io::Result<Vec2> {
        let x = try!(self.read_double());
        let y = try!(self.read_double());
        Ok((x, y))
    }

    fn read_vec3(&mut self) -> io::Result<Vec3> {
        let x = try!(self.read_double());
        let y = try!(self.read_double());
        let z = try!(self.read_double());
        Ok((x, y, z))
    }

    fn read_vec4(&mut self) -> io::Result<Vec4> {
        let x = try!(self.read_double());
        let y = try!(self.read_double());
        let z = try!(self.read_double());
        let w = try!(self.read_double());
        Ok((x, y, z, w))
    }

    fn read_box2(&mut self) -> io::Result<Box2> {
        let min = try!(self.read_vec2());
        let max = try!(self.read_vec2());
        Ok((min, max))
    }

    fn read_string(&mut self) -> io::Result<Box<str>> {
        let length = try!(self.read_uint()) as usize;
        let mut buffer = vec![0; length];
        try!(self.input.read_exact(&mut buffer[..]));

        String::from_utf8(buffer)
            .map(|s| s.into_boxed_str())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn read_blob(&mut self) -> io::Result<Box<[u8]>> {
        let length = try!(self.read_uint()) as usize;
        let mut buffer = vec![0; length];
        try!(self.input.read_exact(&mut buffer[..]));

        Ok(buffer.into_boxed_slice())
    }

    fn read_array_values<F, T>(&mut self, f: F) -> io::Result<Box<[T]>>
        where F: Fn(&mut Self) -> io::Result<T> {

        let length = try!(self.read_uint()) as usize;
        let mut result = Vec::with_capacity(length);
        for _ in 0..length {
            let v = try!(f(self));
            result.push(v);
        }

        Ok(result.into_boxed_slice())
    }
}

impl<R: Read> Reader for BinaryReader<R> {
    fn read_next(&mut self) -> io::Result<Token> {
        let mut buffer = [0; 1];
        let read = try!(self.input.read(&mut buffer));

        if read == 0 {
            return Ok(Token::EndOfFile);
        }

        match buffer[0] {
            0xfe => Ok(Token::Start),
            0xef => Ok(Token::End),

            0x00 ... 0x08 | 0xee =>
                self.read_value(buffer[0]).map(Token::Value),

            0x80 ... 0x86 =>
                self.read_array(buffer[0]).map(Token::Value),

            _ => invalid_token(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Value, Token, Reader};
    use std::io::{self, Cursor};

    fn setup(data: Vec<u8>) -> BinaryReader<Cursor<Vec<u8>>> {
        BinaryReader::new(Cursor::new(data))
    }

    fn is_token(result: io::Result<Token>, expected: Token) -> bool {
        if let Ok(result) = result {
            result == expected
        } else {
            false
        }
    }

    fn is_value(result: io::Result<Token>, expected: Value) -> bool {
        is_token(result, Token::Value(expected))
    }

    fn is_string(result: io::Result<Token>, expected: &str) -> bool {
        let s = expected.to_string().into_boxed_str();
        is_value(result, Value::String(s))
    }

    fn is_blob(result: io::Result<Token>, expected: Vec<u8>) -> bool {
        let b = expected.into_boxed_slice();
        is_value(result, Value::Blob(b))
    }

    #[test]
    fn read_simple() {
        let mut reader = setup(vec![
            0xfe, 0xef
        ]);

        assert!(is_token(reader.read_next(), Token::Start));
        assert!(is_token(reader.read_next(), Token::End));
        assert!(is_token(reader.read_next(), Token::EndOfFile));
    }

    #[test]
    fn read_simple_values() {
        let mut reader = setup(vec![
            0x00, 0x00,
            0x00, 0x01,

            0x01, 0x00,
            0x01, 0x0c,
            0x01, 0x80, 0x02,
            0x01, 0xd0, 0x0f,
            0x01, 0xf3, 0xed, 0x25,

            0x02,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
            0x03,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
            0x04,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
                0x50, 0x8d, 0x97, 0x6e, 0xba, 0x20, 0xc1, 0xc0,
            0x05,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
                0x50, 0x8d, 0x97, 0x6e, 0xba, 0x20, 0xc1, 0xc0,
                0xae, 0x47, 0xe1, 0x7a, 0x14, 0x6a, 0x9d, 0xc0,
            0x06,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
                0x50, 0x8d, 0x97, 0x6e, 0xba, 0x20, 0xc1, 0xc0,
                0xae, 0x47, 0xe1, 0x7a, 0x14, 0x6a, 0x9d, 0xc0,

            0xee, 0x53, 0x48, 0x41, 0x50,
        ]);

        assert!(is_value(reader.read_next(), Value::Bool(false)));
        assert!(is_value(reader.read_next(), Value::Bool(true)));
        assert!(is_value(reader.read_next(), Value::Int(0)));
        assert!(is_value(reader.read_next(), Value::Int(6)));
        assert!(is_value(reader.read_next(), Value::Int(128)));
        assert!(is_value(reader.read_next(), Value::Int(1000)));
        assert!(is_value(reader.read_next(), Value::Int(-310138)));
        assert!(is_value(reader.read_next(), Value::Double(67245.375)));
        assert!(is_value(reader.read_next(), Value::Vec2((67245.375, 3464.85))));
        assert!(is_value(reader.read_next(), Value::Vec3((67245.375, 3464.85, -8769.4565))));
        assert!(is_value(reader.read_next(), Value::Vec4((67245.375, 3464.85, -8769.4565, -1882.52))));
        assert!(is_value(reader.read_next(), Value::Box2(((67245.375, 3464.85), (-8769.4565, -1882.52)))));
        assert!(is_value(reader.read_next(), Value::Tag(tag!(S H A P))));
        assert!(is_token(reader.read_next(), Token::EndOfFile));
    }

    #[test]
    fn read_string() {
        let mut reader = setup(vec![
            0x07, 0x00,
            0x07, 0x05,
                0x48, 0x65, 0x6c, 0x6c, 0x6f,
            0x07, 0x07,
                0x48, 0xc3, 0xa9, 0x6c, 0x6c, 0xc3, 0xb8,
            0x07, 0x93, 0x01,
                0x4c, 0x6f, 0x72, 0x65, 0x6d, 0x20, 0x69, 0x70, 0x73, 0x75,
                0x6d, 0x20, 0x64, 0x6f, 0x6c, 0x6f, 0x72, 0x20, 0x73, 0x69,
                0x74, 0x20, 0x61, 0x6d, 0x65, 0x74, 0x2c, 0x20, 0x63, 0x6f,
                0x6e, 0x73, 0x65, 0x63, 0x74, 0x65, 0x74, 0x75, 0x72, 0x20,
                0x61, 0x64, 0x69, 0x70, 0x69, 0x73, 0x63, 0x69, 0x6e, 0x67,
                0x20, 0x65, 0x6c, 0x69, 0x74, 0x2c, 0x20, 0x73, 0x65, 0x64,
                0x20, 0x64, 0x6f, 0x20, 0x65, 0x69, 0x75, 0x73, 0x6d, 0x6f,
                0x64, 0x20, 0x74, 0x65, 0x6d, 0x70, 0x6f, 0x72, 0x20, 0x69,
                0x6e, 0x63, 0x69, 0x64, 0x69, 0x64, 0x75, 0x6e, 0x74, 0x20,
                0x75, 0x74, 0x20, 0x6c, 0x61, 0x62, 0x6f, 0x72, 0x65, 0x20,
                0x65, 0x74, 0x20, 0x64, 0x6f, 0x6c, 0x6f, 0x72, 0x65, 0x20,
                0x6d, 0x61, 0x67, 0x6e, 0x61, 0x20, 0x61, 0x6c, 0x69, 0x71,
                0x75, 0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 0x65, 0x6e, 0x69,
                0x6d, 0x20, 0x61, 0x64, 0x20, 0x6d, 0x69, 0x6e, 0x69, 0x6d,
                0x20, 0x76, 0x65, 0x6e, 0x69, 0x61, 0x6d,
        ]);

        assert!(is_string(reader.read_next(), ""));
        assert!(is_string(reader.read_next(), "Hello"));
        assert!(is_string(reader.read_next(), "Héllø"));
        assert!(is_string(reader.read_next(),
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, \
             sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
             Ut enim ad minim veniam"));
        assert!(is_token(reader.read_next(), Token::EndOfFile));
    }

    #[test]
    fn read_blob() {
        let mut reader = setup(vec![
            0x08, 0x00,
            0x08, 0x05,
                0x48, 0x65, 0x6c, 0x6c, 0x6f,
            0x08, 0x07,
                0x48, 0xc3, 0xa9, 0x6c, 0x6c, 0xc3, 0xb8,
            0x08, 0x93, 0x01,
                0x4c, 0x6f, 0x72, 0x65, 0x6d, 0x20, 0x69, 0x70, 0x73, 0x75,
                0x6d, 0x20, 0x64, 0x6f, 0x6c, 0x6f, 0x72, 0x20, 0x73, 0x69,
                0x74, 0x20, 0x61, 0x6d, 0x65, 0x74, 0x2c, 0x20, 0x63, 0x6f,
                0x6e, 0x73, 0x65, 0x63, 0x74, 0x65, 0x74, 0x75, 0x72, 0x20,
                0x61, 0x64, 0x69, 0x70, 0x69, 0x73, 0x63, 0x69, 0x6e, 0x67,
                0x20, 0x65, 0x6c, 0x69, 0x74, 0x2c, 0x20, 0x73, 0x65, 0x64,
                0x20, 0x64, 0x6f, 0x20, 0x65, 0x69, 0x75, 0x73, 0x6d, 0x6f,
                0x64, 0x20, 0x74, 0x65, 0x6d, 0x70, 0x6f, 0x72, 0x20, 0x69,
                0x6e, 0x63, 0x69, 0x64, 0x69, 0x64, 0x75, 0x6e, 0x74, 0x20,
                0x75, 0x74, 0x20, 0x6c, 0x61, 0x62, 0x6f, 0x72, 0x65, 0x20,
                0x65, 0x74, 0x20, 0x64, 0x6f, 0x6c, 0x6f, 0x72, 0x65, 0x20,
                0x6d, 0x61, 0x67, 0x6e, 0x61, 0x20, 0x61, 0x6c, 0x69, 0x71,
                0x75, 0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 0x65, 0x6e, 0x69,
                0x6d, 0x20, 0x61, 0x64, 0x20, 0x6d, 0x69, 0x6e, 0x69, 0x6d,
                0x20, 0x76, 0x65, 0x6e, 0x69, 0x61, 0x6d,
        ]);

        assert!(is_blob(reader.read_next(), vec![]));
        assert!(is_blob(reader.read_next(), vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]));
        assert!(is_blob(reader.read_next(), vec![0x48, 0xc3, 0xa9, 0x6c, 0x6c, 0xc3, 0xb8]));
        assert!(is_blob(reader.read_next(), vec![
            0x4c, 0x6f, 0x72, 0x65, 0x6d, 0x20, 0x69, 0x70, 0x73, 0x75,
            0x6d, 0x20, 0x64, 0x6f, 0x6c, 0x6f, 0x72, 0x20, 0x73, 0x69,
            0x74, 0x20, 0x61, 0x6d, 0x65, 0x74, 0x2c, 0x20, 0x63, 0x6f,
            0x6e, 0x73, 0x65, 0x63, 0x74, 0x65, 0x74, 0x75, 0x72, 0x20,
            0x61, 0x64, 0x69, 0x70, 0x69, 0x73, 0x63, 0x69, 0x6e, 0x67,
            0x20, 0x65, 0x6c, 0x69, 0x74, 0x2c, 0x20, 0x73, 0x65, 0x64,
            0x20, 0x64, 0x6f, 0x20, 0x65, 0x69, 0x75, 0x73, 0x6d, 0x6f,
            0x64, 0x20, 0x74, 0x65, 0x6d, 0x70, 0x6f, 0x72, 0x20, 0x69,
            0x6e, 0x63, 0x69, 0x64, 0x69, 0x64, 0x75, 0x6e, 0x74, 0x20,
            0x75, 0x74, 0x20, 0x6c, 0x61, 0x62, 0x6f, 0x72, 0x65, 0x20,
            0x65, 0x74, 0x20, 0x64, 0x6f, 0x6c, 0x6f, 0x72, 0x65, 0x20,
            0x6d, 0x61, 0x67, 0x6e, 0x61, 0x20, 0x61, 0x6c, 0x69, 0x71,
            0x75, 0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 0x65, 0x6e, 0x69,
            0x6d, 0x20, 0x61, 0x64, 0x20, 0x6d, 0x69, 0x6e, 0x69, 0x6d,
            0x20, 0x76, 0x65, 0x6e, 0x69, 0x61, 0x6d,
        ]));
        assert!(is_token(reader.read_next(), Token::EndOfFile));
    }

    #[test]
    fn read_arrays() {
        let mut reader = setup(vec![
            0x80, 0x00,
            0x80, 0x03,
                0x01, 0x00, 0x01,

            0x81, 0x00,
            0x81, 0x03,
                0x0c, 0x80, 0x02, 0xd0, 0x0f,

            0x82, 0x00,
            0x82, 0x03,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
                0x50, 0x8d, 0x97, 0x6e, 0xba, 0x20, 0xc1, 0xc0,

            0x83, 0x00,
            0x83, 0x02,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
                0x50, 0x8d, 0x97, 0x6e, 0xba, 0x20, 0xc1, 0xc0,
                0xae, 0x47, 0xe1, 0x7a, 0x14, 0x6a, 0x9d, 0xc0,

            0x84, 0x00,
            0x84, 0x02,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
                0x50, 0x8d, 0x97, 0x6e, 0xba, 0x20, 0xc1, 0xc0,
                0xae, 0x47, 0xe1, 0x7a, 0x14, 0x6a, 0x9d, 0xc0,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,

            0x85, 0x00,
            0x85, 0x02,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
                0x50, 0x8d, 0x97, 0x6e, 0xba, 0x20, 0xc1, 0xc0,
                0xae, 0x47, 0xe1, 0x7a, 0x14, 0x6a, 0x9d, 0xc0,
                0xae, 0x47, 0xe1, 0x7a, 0x14, 0x6a, 0x9d, 0xc0,
                0x50, 0x8d, 0x97, 0x6e, 0xba, 0x20, 0xc1, 0xc0,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,

            0x86, 0x00,
            0x86, 0x02,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
                0x50, 0x8d, 0x97, 0x6e, 0xba, 0x20, 0xc1, 0xc0,
                0xae, 0x47, 0xe1, 0x7a, 0x14, 0x6a, 0x9d, 0xc0,
                0xae, 0x47, 0xe1, 0x7a, 0x14, 0x6a, 0x9d, 0xc0,
                0x50, 0x8d, 0x97, 0x6e, 0xba, 0x20, 0xc1, 0xc0,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
        ]);

        assert!(is_value(reader.read_next(), Value::BoolArray(vec![].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::BoolArray(vec![
            true, false, true
        ].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::IntArray(vec![].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::IntArray(vec![
            6, 128, 1000
        ].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::DoubleArray(vec![].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::DoubleArray(vec![
            67245.375, 3464.85, -8769.4565
        ].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::Vec2Array(vec![].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::Vec2Array(vec![
            (67245.375, 3464.85),
            (-8769.4565, -1882.52)
        ].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::Vec3Array(vec![].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::Vec3Array(vec![
            (67245.375, 3464.85, -8769.4565),
            (-1882.52, 67245.375, 3464.85),
        ].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::Vec4Array(vec![].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::Vec4Array(vec![
            (67245.375, 3464.85, -8769.4565, -1882.52),
            (-1882.52, -8769.4565, 3464.85, 67245.375),
        ].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::Box2Array(vec![].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::Box2Array(vec![
            ((67245.375, 3464.85), (-8769.4565, -1882.52)),
            ((-1882.52, -8769.4565), (3464.85, 67245.375)),
        ].into_boxed_slice())));
        assert!(is_token(reader.read_next(), Token::EndOfFile));
    }

    #[test]
    fn skip_to_end() {
        let mut reader = setup(vec![]);
        reader.skip_to_end().unwrap();
        assert!(is_token(reader.read_next(), Token::EndOfFile));

        let mut reader = setup(vec![0xfe, 0xef]);
        reader.skip_to_end().unwrap();
        assert!(is_token(reader.read_next(), Token::EndOfFile));

        let mut reader = setup(vec![0xef, 0xef]);
        reader.skip_to_end().unwrap();
        assert!(is_token(reader.read_next(), Token::End));
        assert!(is_token(reader.read_next(), Token::EndOfFile));

        let mut reader = setup(vec![0xfe, 0xef, 0xef]);
        reader.skip_to_end().unwrap();
        assert!(is_token(reader.read_next(), Token::EndOfFile));

        let mut reader = setup(vec![0xfe, 0xef, 0xef, 0xef]);
        reader.skip_to_end().unwrap();
        assert!(is_token(reader.read_next(), Token::End));
        assert!(is_token(reader.read_next(), Token::EndOfFile));

        let mut reader = setup(vec![0xfe]);
        assert!(reader.skip_to_end().is_err());

        let mut reader = setup(vec![
            0x00, 0x01,
            0x01, 0xf3, 0xed, 0x25,
            0x03,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
            0xee, 0x53, 0x48, 0x41, 0x50,
            0x07, 0x05,
                0x48, 0x65, 0x6c, 0x6c, 0x6f,
            0x08, 0x05,
                0x48, 0x65, 0x6c, 0x6c, 0x6f,
            0x81, 0x03,
                0x0c, 0x80, 0x02, 0xd0, 0x0f,

            0xef, 0xef
        ]);
        reader.skip_to_end().unwrap();
        assert!(is_token(reader.read_next(), Token::End));
        assert!(is_token(reader.read_next(), Token::EndOfFile));

    }

    #[test]
    fn expect_start() {
        let mut reader = setup(vec![0xfe]);
        assert!(reader.expect_start().is_ok());

        let mut reader = setup(vec![0xef]);
        assert!(reader.expect_start().is_err());

        let mut reader = setup(vec![0x01, 0x01]);
        assert!(reader.expect_start().is_err());
    }

    #[test]
    fn expect_start_or_end() {
        let mut reader = setup(vec![0xfe]);
        assert_eq!(reader.expect_start_or_end().unwrap(), true);

        let mut reader = setup(vec![0xef]);
        assert_eq!(reader.expect_start_or_end().unwrap(), false);

        let mut reader = setup(vec![]);
        assert_eq!(reader.expect_start_or_end().unwrap(), false);

        let mut reader = setup(vec![0x01, 0x01]);
        assert!(reader.expect_start_or_end().is_err());
    }

    #[test]
    fn expect_simple_values() {
        let mut reader = setup(vec![
            0x00, 0x01,

            0x01, 0xd0, 0x0f,

            0x02,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
            0x03,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
            0x04,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
                0x50, 0x8d, 0x97, 0x6e, 0xba, 0x20, 0xc1, 0xc0,
            0x05,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
                0x50, 0x8d, 0x97, 0x6e, 0xba, 0x20, 0xc1, 0xc0,
                0xae, 0x47, 0xe1, 0x7a, 0x14, 0x6a, 0x9d, 0xc0,
            0x06,
                0x00, 0x00, 0x00, 0x00, 0xd6, 0x6a, 0xf0, 0x40,
                0x33, 0x33, 0x33, 0x33, 0xb3, 0x11, 0xab, 0x40,
                0x50, 0x8d, 0x97, 0x6e, 0xba, 0x20, 0xc1, 0xc0,
                0xae, 0x47, 0xe1, 0x7a, 0x14, 0x6a, 0x9d, 0xc0,

            0xee, 0x53, 0x48, 0x41, 0x50,
        ]);

        assert_eq!(reader.expect_bool().unwrap(), true);
        assert_eq!(reader.expect_int().unwrap(), 1000);
        assert_eq!(reader.expect_double().unwrap(), 67245.375);
        assert_eq!(reader.expect_vec2().unwrap(), (67245.375, 3464.85));
        assert_eq!(reader.expect_vec3().unwrap(), (67245.375, 3464.85, -8769.4565));
        assert_eq!(reader.expect_vec4().unwrap(), (67245.375, 3464.85, -8769.4565, -1882.52));
        assert_eq!(reader.expect_box2().unwrap(), ((67245.375, 3464.85), (-8769.4565, -1882.52)));
        assert_eq!(reader.expect_tag().unwrap(), tag!(S H A P));
    }

    #[test]
    fn expect_simple_values_fail() {
        assert!(setup(vec![0xfe]).expect_bool().is_err());
        assert!(setup(vec![0xfe]).expect_int().is_err());
        assert!(setup(vec![0xfe]).expect_double().is_err());
        assert!(setup(vec![0xfe]).expect_vec2().is_err());
        assert!(setup(vec![0xfe]).expect_vec3().is_err());
        assert!(setup(vec![0xfe]).expect_vec4().is_err());
        assert!(setup(vec![0xfe]).expect_box2().is_err());
        assert!(setup(vec![0xfe]).expect_tag().is_err());
    }

    #[test]
    fn expect_arrays() {
        let mut reader = setup(vec![
            0x80, 0x00,
            0x81, 0x00,
            0x82, 0x00,
            0x83, 0x00,
            0x84, 0x00,
            0x85, 0x00,
            0x86, 0x00,
        ]);

        assert_eq!(reader.expect_bool_array().unwrap(), vec![].into_boxed_slice());
        assert_eq!(reader.expect_int_array().unwrap(), vec![].into_boxed_slice());
        assert_eq!(reader.expect_double_array().unwrap(), vec![].into_boxed_slice());
        assert_eq!(reader.expect_vec2_array().unwrap(), vec![].into_boxed_slice());
        assert_eq!(reader.expect_vec3_array().unwrap(), vec![].into_boxed_slice());
        assert_eq!(reader.expect_vec4_array().unwrap(), vec![].into_boxed_slice());
        assert_eq!(reader.expect_box2_array().unwrap(), vec![].into_boxed_slice());
    }

    #[test]
    fn expect_arrays_fail() {
        assert!(setup(vec![0xfe]).expect_bool_array().is_err());
        assert!(setup(vec![0xfe]).expect_int_array().is_err());
        assert!(setup(vec![0xfe]).expect_double_array().is_err());
        assert!(setup(vec![0xfe]).expect_vec2_array().is_err());
        assert!(setup(vec![0xfe]).expect_vec3_array().is_err());
        assert!(setup(vec![0xfe]).expect_vec4_array().is_err());
        assert!(setup(vec![0xfe]).expect_box2_array().is_err());
    }

    #[test]
    fn expect_string() {
        let mut reader = setup(vec![
            0x07, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f,
            0xfe
        ]);

        assert_eq!(reader.expect_string().unwrap(), "Hello".to_string().into_boxed_str());
        assert!(reader.expect_string().is_err());
    }

    #[test]
    fn expect_blob() {
        let mut reader = setup(vec![
            0x08, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f,
            0xfe
        ]);

        assert_eq!(reader.expect_blob().unwrap(), vec![0x48, 0x65, 0x6c, 0x6c, 0x6f].into_boxed_slice());
        assert!(reader.expect_blob().is_err());
    }
}
