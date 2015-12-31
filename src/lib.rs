extern crate byteorder;

use std::io::{self, Read, Write};
use byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};

pub type Tag = u32;
pub type Vec2 = (f64, f64);
pub type Vec3 = (f64, f64, f64);
pub type Vec4 = (f64, f64, f64, f64);
pub type Box2 = (Vec2, Vec2);

#[derive(PartialEq, Debug)]
pub enum Value {
    Tag(Tag),
    Bool(bool),
    BoolArray(Box<[bool]>),
    Int(i32),
    IntArray(Box<[i32]>),
    Double(f64),
    DoubleArray(Box<[f64]>),
    Vec2(Vec2),
    Vec2Array(Box<[Vec2]>),
    Vec3(Vec3),
    Vec3Array(Box<[Vec3]>),
    Vec4(Vec4),
    Vec4Array(Box<[Vec4]>),
    Box2(Box2),
    Box2Array(Box<[Box2]>),
    String(Box<str>),
    Blob(Box<[u8]>)
}

#[derive(PartialEq, Debug)]
pub enum Token {
    Start,
    End,
    Value(Value),
    EndOfFile
}

macro_rules! char_num {
    (0) => (30); (1) => (31); (2) => (32); (3) => (33); (4) => (34); (5) => (35);
    (6) => (36); (7) => (37); (8) => (38); (9) => (39); (A) => (65); (B) => (66);
    (C) => (67); (D) => (68); (E) => (69); (F) => (70); (G) => (71); (H) => (72);
    (I) => (73); (J) => (74); (K) => (75); (L) => (76); (M) => (77); (N) => (78);
    (O) => (79); (P) => (80); (Q) => (81); (R) => (82); (S) => (83); (T) => (84);
    (U) => (85); (V) => (86); (W) => (87); (X) => (88); (Y) => (89); (Z) => (90);
}

#[macro_export]
macro_rules! tag {
    ($a:tt $b:tt $c:tt $d:tt) => (
        char_num!($a) << 24 |
        char_num!($b) << 16 |
        char_num!($c) << 8 |
        char_num!($d)
    );
}

pub struct Reader<R> {
    input: R
}

fn invalid_token<T>() -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::Other, "Invalid token"))
}

fn map_to_io(err: byteorder::Error) -> io::Error {
    match err {
        byteorder::Error::Io(err) => err,
        byteorder::Error::UnexpectedEOF =>
            io::Error::new(io::ErrorKind::Other, "unexpected EOF"),
    }
}

impl<R: Read> Reader<R> {
    pub fn new(input: R) -> Reader<R> {
        Reader { input: input }
    }

    fn read_exact(&mut self, mut buf: &mut [u8]) -> io::Result<()> {
        while !buf.is_empty() {
            match self.input.read(buf) {
                Ok(0) => break,
                Ok(n) => { let tmp = buf; buf = &mut tmp[n..]; },
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {},
                Err(e) => return Err(e),
            }
        }

        if !buf.is_empty() {
            Err(io::Error::new(io::ErrorKind::InvalidData, "failed to fill whole buffer"))
        } else {
            Ok(())
        }
    }

    pub fn read_next(&mut self) -> io::Result<Token> {
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
            0x80 => self.read_array_values(Reader::read_bool)
                .map(Value::BoolArray),
            0x81 => self.read_array_values(Reader::read_int)
                .map(Value::IntArray),
            0x82 => self.read_array_values(Reader::read_double)
                .map(Value::DoubleArray),
            0x83 => self.read_array_values(Reader::read_vec2)
                .map(Value::Vec2Array),
            0x84 => self.read_array_values(Reader::read_vec3)
                .map(Value::Vec3Array),
            0x85 => self.read_array_values(Reader::read_vec4)
                .map(Value::Vec4Array),
            0x86 => self.read_array_values(Reader::read_box2)
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
        try!(self.read_exact(&mut buffer));

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

            try!(self.read_exact(&mut buffer));

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
        self.read_double()
            .and_then(|x| self.read_double()
                      .map(|y| (x, y)))
    }

    fn read_vec3(&mut self) -> io::Result<Vec3> {
        self.read_double()
            .and_then(|x| self.read_double()
                      .and_then(|y| self.read_double()
                                .map(|z| (x, y, z))))
    }

    fn read_vec4(&mut self) -> io::Result<Vec4> {
        self.read_double()
            .and_then(|x| self.read_double()
                      .and_then(|y| self.read_double()
                                .and_then(|z| self.read_double()
                                          .map(|w| (x, y, z, w)))))
    }

    fn read_box2(&mut self) -> io::Result<Box2> {
        self.read_vec2()
            .and_then(|min|
                      self.read_vec2()
                      .map(|max| (min, max)))
    }

    fn read_string(&mut self) -> io::Result<Box<str>> {
        let length = try!(self.read_uint()) as usize;
        let mut buffer = vec![0; length];
        try!(self.read_exact(&mut buffer[..]));

        String::from_utf8(buffer)
            .map(|s| s.into_boxed_str())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn read_blob(&mut self) -> io::Result<Box<[u8]>> {
        let length = try!(self.read_uint()) as usize;
        let mut buffer = vec![0; length];
        try!(self.read_exact(&mut buffer[..]));

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

pub struct Writer<W> {
    output: W
}

fn get_type(value: &Value) -> u8 {
    match value {
        &Value::Bool(_) => 0x00,
        &Value::Int(_) => 0x01,
        &Value::Double(_) => 0x02,
        &Value::Vec2(_) => 0x03,
        &Value::Vec3(_) => 0x04,
        &Value::Vec4(_) => 0x05,
        &Value::Box2(_) => 0x06,
        &Value::String(_) => 0x07,
        &Value::Blob(_) => 0x08,
        &Value::Tag(_) => 0xee,

        &Value::BoolArray(_) => 0x80,
        &Value::IntArray(_) => 0x81,
        &Value::DoubleArray(_) => 0x82,
        &Value::Vec2Array(_) => 0x83,
        &Value::Vec3Array(_) => 0x84,
        &Value::Vec4Array(_) => 0x85,
        &Value::Box2Array(_) => 0x86,
    }
}

impl<W: Write> Writer<W> {
    pub fn new(output: W) -> Writer<W> {
        Writer { output: output }
    }

    pub fn write_start(&mut self) -> io::Result<()> {
        self.output.write_all(&[0xfe])
    }

    pub fn write_end(&mut self) -> io::Result<()> {
        self.output.write_all(&[0xef])
    }

    pub fn write_value(&mut self, value: &Value) -> io::Result<()> {
        try!(self.output.write_all(&[get_type(value)]));

        match value {
            &Value::Bool(value) => self.write_bool(value),
            &Value::Int(value) => self.write_int(value),
            &Value::Double(value) => self.write_double(value),
            &Value::Vec2(value) => self.write_vec2(value),
            &Value::Vec3(value) => self.write_vec3(value),
            &Value::Vec4(value) => self.write_vec4(value),
            &Value::Box2(value) => self.write_box2(value),
            &Value::String(ref value) => self.write_string(value),
            &Value::Blob(ref value) => self.write_blob(value),
            &Value::Tag(value) => self.write_tag(value),

            &Value::BoolArray(ref values) => self.write_array(values, Writer::write_bool),
            &Value::IntArray(ref values) => self.write_array(values, Writer::write_int),
            &Value::DoubleArray(ref values) => self.write_array(values, Writer::write_double),
            &Value::Vec2Array(ref values) => self.write_array(values, Writer::write_vec2),
            &Value::Vec3Array(ref values) => self.write_array(values, Writer::write_vec3),
            &Value::Vec4Array(ref values) => self.write_array(values, Writer::write_vec4),
            &Value::Box2Array(ref values) => self.write_array(values, Writer::write_box2),
        }
    }

    fn write_tag(&mut self, value: Tag) -> io::Result<()> {
        self.output.write_u32::<BigEndian>(value)
            .map_err(map_to_io)
    }

    fn write_bool(&mut self, value: bool) -> io::Result<()> {
        self.output.write_all(&[
            if value { 0x01 } else { 0x00 }
        ])
    }

    fn write_uint(&mut self, value: u64) -> io::Result<()> {
        let mut buffer = Vec::with_capacity(10);
        let mut value = value;

        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;

            if value != 0 {
                byte |= 0x80;
            }

            buffer.push(byte);

            if value == 0 {
                break;
            }
        }

        self.output.write_all(&buffer)
    }

    fn write_sint(&mut self, value: i64) -> io::Result<()> {
        self.write_uint(if value < 0 {
            !(value << 1)
        } else {
            value << 1
        } as u64)
    }

    fn write_int(&mut self, value: i32) -> io::Result<()> {
        self.write_sint(value as i64)
    }

    fn write_double(&mut self, value: f64) -> io::Result<()> {
        self.output.write_f64::<LittleEndian>(value)
            .map_err(map_to_io)
    }

    fn write_vec2(&mut self, value: Vec2) -> io::Result<()> {
        try!(self.write_double(value.0));
        try!(self.write_double(value.1));
        Ok(())
    }

    fn write_vec3(&mut self, value: Vec3) -> io::Result<()> {
        try!(self.write_double(value.0));
        try!(self.write_double(value.1));
        try!(self.write_double(value.2));
        Ok(())
    }

    fn write_vec4(&mut self, value: Vec4) -> io::Result<()> {
        try!(self.write_double(value.0));
        try!(self.write_double(value.1));
        try!(self.write_double(value.2));
        try!(self.write_double(value.3));
        Ok(())
    }

    fn write_box2(&mut self, value: Box2) -> io::Result<()> {
        try!(self.write_vec2(value.0));
        try!(self.write_vec2(value.1));
        Ok(())
    }

    fn write_string(&mut self, value: &Box<str>) -> io::Result<()> {
        let bytes = value.as_bytes();
        try!(self.write_uint(bytes.len() as u64));
        self.output.write_all(bytes)
    }

    fn write_blob(&mut self, value: &Box<[u8]>) -> io::Result<()> {
        try!(self.write_uint(value.len() as u64));
        self.output.write_all(value)
    }

    fn write_array<T, F>(&mut self, values: &Box<[T]>, f: F) -> io::Result<()>
        where F: Fn(&mut Self, T) -> io::Result<()>, T: Copy {

        try!(self.write_uint(values.len() as u64));
        for &v in values.iter() {
            try!(f(self, v));
        }
        Ok(())

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor};

    fn setup(data: Vec<u8>) -> Reader<Cursor<Vec<u8>>> {
        Reader::new(Cursor::new(data))
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
                0x75, 0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 0x65, 0x6E, 0x69,
                0x6D, 0x20, 0x61, 0x64, 0x20, 0x6D, 0x69, 0x6E, 0x69, 0x6D,
                0x20, 0x76, 0x65, 0x6E, 0x69, 0x61, 0x6D,
        ]);

        assert!(is_string(reader.read_next(), ""));
        assert!(is_string(reader.read_next(), "Hello"));
        assert!(is_string(reader.read_next(), "Héllø"));
        assert!(is_string(reader.read_next(), "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam"));
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
                0x75, 0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 0x65, 0x6E, 0x69,
                0x6D, 0x20, 0x61, 0x64, 0x20, 0x6D, 0x69, 0x6E, 0x69, 0x6D,
                0x20, 0x76, 0x65, 0x6E, 0x69, 0x61, 0x6D,
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
            0x75, 0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 0x65, 0x6E, 0x69,
            0x6D, 0x20, 0x61, 0x64, 0x20, 0x6D, 0x69, 0x6E, 0x69, 0x6D,
            0x20, 0x76, 0x65, 0x6E, 0x69, 0x61, 0x6D,
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
    fn tag_macro() {
        const SHAP: Tag = tag!(S H A P);
        const PATH: Tag = tag!(P A T H);

        assert_eq!(SHAP, 1397244240u32);
        assert_eq!(PATH, 1346458696u32);
    }

    #[test]
    fn write_simple() {
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        writer.write_start().unwrap();
        writer.write_end().unwrap();

        assert_eq!(writer.output.into_inner(), vec![
            0xfe, 0xef
        ]);
    }

    #[test]
    fn write_simple_values() {
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        writer.write_value(&Value::Bool(false)).unwrap();
        writer.write_value(&Value::Bool(true)).unwrap();
        writer.write_value(&Value::Int(0)).unwrap();
        writer.write_value(&Value::Int(6)).unwrap();
        writer.write_value(&Value::Int(128)).unwrap();
        writer.write_value(&Value::Int(1000)).unwrap();
        writer.write_value(&Value::Int(-310138)).unwrap();
        writer.write_value(&Value::Double(67245.375)).unwrap();
        writer.write_value(&Value::Vec2((67245.375, 3464.85))).unwrap();
        writer.write_value(&Value::Vec3((67245.375, 3464.85, -8769.4565))).unwrap();
        writer.write_value(&Value::Vec4((67245.375, 3464.85, -8769.4565, -1882.52))).unwrap();
        writer.write_value(&Value::Box2(((67245.375, 3464.85), (-8769.4565, -1882.52)))).unwrap();
        writer.write_value(&Value::Tag(tag!(S H A P))).unwrap();

        assert_eq!(writer.output.into_inner(), vec![
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
    }

    #[test]
    fn write_string() {
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        writer.write_value(&Value::String("".to_string().into_boxed_str())).unwrap();
        writer.write_value(&Value::String("Hello".to_string().into_boxed_str())).unwrap();
        writer.write_value(&Value::String("Héllø".to_string().into_boxed_str())).unwrap();
        writer.write_value(&Value::String("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam".to_string().into_boxed_str())).unwrap();

        assert_eq!(writer.output.into_inner(), vec![
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
                0x75, 0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 0x65, 0x6E, 0x69,
                0x6D, 0x20, 0x61, 0x64, 0x20, 0x6D, 0x69, 0x6E, 0x69, 0x6D,
                0x20, 0x76, 0x65, 0x6E, 0x69, 0x61, 0x6D,
        ]);
    }

    #[test]
    fn write_blob() {
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        writer.write_value(&Value::Blob(vec![].into_boxed_slice())).unwrap();
        writer.write_value(&Value::Blob(vec![0x48, 0x65, 0x6c, 0x6c, 0x6f].into_boxed_slice())).unwrap();
        writer.write_value(&Value::Blob(vec![0x48, 0xc3, 0xa9, 0x6c, 0x6c, 0xc3, 0xb8].into_boxed_slice())).unwrap();
        writer.write_value(&Value::Blob(vec![
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
            0x75, 0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 0x65, 0x6E, 0x69,
            0x6D, 0x20, 0x61, 0x64, 0x20, 0x6D, 0x69, 0x6E, 0x69, 0x6D,
            0x20, 0x76, 0x65, 0x6E, 0x69, 0x61, 0x6D,
        ].into_boxed_slice())).unwrap();

        assert_eq!(writer.output.into_inner(), vec![
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
                0x75, 0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 0x65, 0x6E, 0x69,
                0x6D, 0x20, 0x61, 0x64, 0x20, 0x6D, 0x69, 0x6E, 0x69, 0x6D,
                0x20, 0x76, 0x65, 0x6E, 0x69, 0x61, 0x6D,
        ]);
    }

    #[test]
    fn write_arrays() {
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        writer.write_value(&Value::BoolArray(vec![].into_boxed_slice())).unwrap();
        writer.write_value(&Value::BoolArray(vec![
            true, false, true
        ].into_boxed_slice())).unwrap();

        writer.write_value(&Value::IntArray(vec![].into_boxed_slice())).unwrap();
        writer.write_value(&Value::IntArray(vec![
            6, 128, 1000
        ].into_boxed_slice())).unwrap();

        writer.write_value(&Value::DoubleArray(vec![].into_boxed_slice())).unwrap();
        writer.write_value(&Value::DoubleArray(vec![
            67245.375, 3464.85, -8769.4565
        ].into_boxed_slice())).unwrap();

        writer.write_value(&Value::Vec2Array(vec![].into_boxed_slice())).unwrap();
        writer.write_value(&Value::Vec2Array(vec![
            (67245.375, 3464.85),
            (-8769.4565, -1882.52)
        ].into_boxed_slice())).unwrap();

        writer.write_value(&Value::Vec3Array(vec![].into_boxed_slice())).unwrap();
        writer.write_value(&Value::Vec3Array(vec![
            (67245.375, 3464.85, -8769.4565),
            (-1882.52, 67245.375, 3464.85),
        ].into_boxed_slice())).unwrap();

        writer.write_value(&Value::Vec4Array(vec![].into_boxed_slice())).unwrap();
        writer.write_value(&Value::Vec4Array(vec![
            (67245.375, 3464.85, -8769.4565, -1882.52),
            (-1882.52, -8769.4565, 3464.85, 67245.375),
        ].into_boxed_slice())).unwrap();

        writer.write_value(&Value::Box2Array(vec![].into_boxed_slice())).unwrap();
        writer.write_value(&Value::Box2Array(vec![
            ((67245.375, 3464.85), (-8769.4565, -1882.52)),
            ((-1882.52, -8769.4565), (3464.85, 67245.375)),
        ].into_boxed_slice())).unwrap();

        assert_eq!(writer.output.into_inner(), vec![
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
    }
}
