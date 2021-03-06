use std::io::{self, Write};
use byteorder::{LittleEndian, BigEndian, WriteBytesExt};
use super::{Value, Tag, Vec2, Vec3, Vec4, Box2, Writer};

pub struct BinaryWriter<W> {
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

impl<W: Write> BinaryWriter<W> {
    pub fn new(output: W) -> BinaryWriter<W> {
        BinaryWriter { output: output }
    }

    pub fn into_inner(self) -> W {
        self.output
    }

    fn write_tag(&mut self, value: Tag) -> io::Result<()> {
        self.output.write_u32::<BigEndian>(value)
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

impl<W: Write> Writer for BinaryWriter<W> {
    fn write_start(&mut self) -> io::Result<()> {
        self.output.write_all(&[0xfe])
    }

    fn write_end(&mut self) -> io::Result<()> {
        self.output.write_all(&[0xef])
    }

    fn write_value(&mut self, value: &Value) -> io::Result<()> {
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

            &Value::BoolArray(ref values) => self.write_array(values, BinaryWriter::write_bool),
            &Value::IntArray(ref values) => self.write_array(values, BinaryWriter::write_int),
            &Value::DoubleArray(ref values) => self.write_array(values, BinaryWriter::write_double),
            &Value::Vec2Array(ref values) => self.write_array(values, BinaryWriter::write_vec2),
            &Value::Vec3Array(ref values) => self.write_array(values, BinaryWriter::write_vec3),
            &Value::Vec4Array(ref values) => self.write_array(values, BinaryWriter::write_vec4),
            &Value::Box2Array(ref values) => self.write_array(values, BinaryWriter::write_box2),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Value, Writer};
    use std::io::Cursor;

    #[test]
    fn write_simple() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));

        writer.write_start().unwrap();
        writer.write_end().unwrap();

        assert_eq!(writer.output.into_inner(), vec![
            0xfe, 0xef
        ]);
    }

    #[test]
    fn write_simple_values() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));

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
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));

        writer.write_value(&Value::String("".to_string().into_boxed_str())).unwrap();
        writer.write_value(&Value::String("Hello".to_string().into_boxed_str())).unwrap();
        writer.write_value(&Value::String("Héllø".to_string().into_boxed_str())).unwrap();
        writer.write_value(&Value::String(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, \
             sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
             Ut enim ad minim veniam".to_string().into_boxed_str())).unwrap();

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
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));

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
            0x75, 0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 0x65, 0x6e, 0x69,
            0x6d, 0x20, 0x61, 0x64, 0x20, 0x6d, 0x69, 0x6e, 0x69, 0x6d,
            0x20, 0x76, 0x65, 0x6e, 0x69, 0x61, 0x6d,
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
                0x75, 0x61, 0x2e, 0x20, 0x55, 0x74, 0x20, 0x65, 0x6e, 0x69,
                0x6d, 0x20, 0x61, 0x64, 0x20, 0x6d, 0x69, 0x6e, 0x69, 0x6d,
                0x20, 0x76, 0x65, 0x6e, 0x69, 0x61, 0x6d,
        ]);
    }

    #[test]
    fn write_arrays() {
        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));

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
