use std::io::{self, Write};
use super::{Value, Tag, Vec2, Vec3, Vec4, Box2, Writer};

pub struct TextWriter<W> {
    output: W,
    first: bool,
    indent: i32,
}

impl<W: Write> TextWriter<W> {
    pub fn new(output: W) -> TextWriter<W> {
        TextWriter {
            output: output,
            first: true,
            indent: 0,
        }
    }

    pub fn into_inner(self) -> W {
        self.output
    }

    fn write_indent(&mut self, n: i32) -> io::Result<()> {
        for _ in 0..n {
            try!(self.output.write_all(b"  "));
        }
        Ok(())
    }

    fn write_single_value(&mut self, value: &Value) -> io::Result<()> {
        match value {
            &Value::Bool(v) => self.write_bool(v),
            &Value::Int(v) => self.write_int(v),
            &Value::Double(v) => self.write_double(v),
            &Value::Vec2(v) => self.write_vec2(v),
            &Value::Vec3(v) => self.write_vec3(v),
            &Value::Vec4(v) => self.write_vec4(v),
            &Value::Box2(v) => self.write_box2(v),
            &Value::String(ref v) => self.write_string(v),
            &Value::Blob(ref v) => self.write_blob(v),
            &Value::Tag(v) => self.write_tag(v),

            &Value::BoolArray(ref v) => self.write_array(v, TextWriter::write_bool),
            &Value::IntArray(ref v) => self.write_array(v, TextWriter::write_int),
            &Value::DoubleArray(ref v) => self.write_array(v, TextWriter::write_double),
            &Value::Vec2Array(ref v) => self.write_array(v, TextWriter::write_vec2),
            &Value::Vec3Array(ref v) => self.write_array(v, TextWriter::write_vec3),
            &Value::Vec4Array(ref v) => self.write_array(v, TextWriter::write_vec4),
            &Value::Box2Array(ref v) => self.write_array(v, TextWriter::write_box2),
        }
    }

    fn write_tag(&mut self, value: Tag) -> io::Result<()> {
        let a = ((0xff000000 & value) >> 24) as u8;
        let b = ((0x00ff0000 & value) >> 16) as u8;
        let c = ((0x0000ff00 & value) >> 8) as u8;
        let d = ((0x000000ff & value) >> 0) as u8;

        write!(self.output, "{}{}{}{}", a as char, b as char, c as char, d as char)
    }

    fn write_bool(&mut self, value: bool) -> io::Result<()> {
        self.output.write_all(
            if value { b"true" } else { b"false" })
    }

    fn write_int(&mut self, value: i32) -> io::Result<()> {
        write!(self.output, "{}", value)
    }

    fn write_double(&mut self, value: f64) -> io::Result<()> {
        if value.fract() == 0.0 {
           write!(self.output, "{:.1}", value)
        } else {
            write!(self.output, "{}", value)
        }
    }

    fn write_vec2(&mut self, (x, y): Vec2) -> io::Result<()> {
        try!(self.output.write(b"["));
        try!(self.write_double(x));
        try!(self.output.write(b" "));
        try!(self.write_double(y));
        try!(self.output.write(b"]"));
        Ok(())
    }

    fn write_vec3(&mut self, (x, y, z): Vec3) -> io::Result<()> {
        try!(self.output.write(b"["));
        try!(self.write_double(x));
        try!(self.output.write(b" "));
        try!(self.write_double(y));
        try!(self.output.write(b" "));
        try!(self.write_double(z));
        try!(self.output.write(b"]"));
        Ok(())
    }

    fn write_vec4(&mut self, (x, y, z, w): Vec4) -> io::Result<()> {
        try!(self.output.write(b"["));
        try!(self.write_double(x));
        try!(self.output.write(b" "));
        try!(self.write_double(y));
        try!(self.output.write(b" "));
        try!(self.write_double(z));
        try!(self.output.write(b" "));
        try!(self.write_double(w));
        try!(self.output.write(b"]"));
        Ok(())
    }

    fn write_box2(&mut self, ((x, y), (z, w)): Box2) -> io::Result<()> {
        try!(self.output.write(b"[["));
        try!(self.write_double(x));
        try!(self.output.write(b" "));
        try!(self.write_double(y));
        try!(self.output.write(b"] ["));
        try!(self.write_double(z));
        try!(self.output.write(b" "));
        try!(self.write_double(w));
        try!(self.output.write(b"]]"));
        Ok(())
    }

    fn write_string(&mut self, value: &Box<str>) -> io::Result<()> {
        write!(self.output, "\"{}\"", value.replace("\"", "\"\""))
    }

    fn write_blob(&mut self, value: &Box<[u8]>) -> io::Result<()> {
        try!(write!(self.output, "0x"));
        for b in value.iter() {
            try!(write!(self.output, "{:02x}", b));
        }
        Ok(())
    }

    fn write_array<T, F>(&mut self, values: &Box<[T]>, f: F) -> io::Result<()>
        where F: Fn(&mut Self, T) -> io::Result<()>, T: Copy {

        let indent = self.indent;
        try!(self.output.write_all(b"{\n"));
        for &v in values.iter() {
            try!(self.write_indent(indent + 1));
            try!(f(self, v));
            try!(self.output.write_all(b"\n"));
        }
        try!(self.write_indent(indent));
        try!(self.output.write_all(b"}"));
        Ok(())
    }
}

impl<W: Write> Writer for TextWriter<W> {
    fn write_start(&mut self) -> io::Result<()> {
        let indent = self.indent;
        if (indent == 0 && !self.first) || indent > 0 {
            try!(self.output.write_all(b"\n"));
        }

        try!(self.write_indent(indent));
        try!(self.output.write_all(b"("));

        self.first = true;
        self.indent += 1;

        Ok(())
    }

    fn write_end(&mut self) -> io::Result<()> {
        self.indent -= 1;
        self.output.write_all(b")")
    }

    fn write_value(&mut self, value: &Value) -> io::Result<()> {
        if self.first {
            self.first = false;
        } else {
            try!(self.output.write_all(b" "));
        }

        self.write_single_value(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Value, Writer};
    use std::io::Cursor;

    fn setup() -> TextWriter<Cursor<Vec<u8>>> {
        TextWriter::new(Cursor::new(Vec::new()))
    }

    fn result(writer: TextWriter<Cursor<Vec<u8>>>) -> String {
        String::from_utf8(writer.output.into_inner()).unwrap()
    }

    #[test]
    fn write_simple() {
        let mut writer = setup();

        writer.write_start().unwrap();
        writer.write_end().unwrap();

        assert_eq!(result(writer), "()");
    }

    #[test]
    fn write_simple_values() {
        let mut writer = setup();

        writer.write_value(&Value::Bool(false)).unwrap();
        writer.write_value(&Value::Bool(true)).unwrap();
        writer.write_value(&Value::Int(0)).unwrap();
        writer.write_value(&Value::Int(6)).unwrap();
        writer.write_value(&Value::Int(128)).unwrap();
        writer.write_value(&Value::Int(1000)).unwrap();
        writer.write_value(&Value::Int(-310138)).unwrap();
        writer.write_value(&Value::Double(0.0)).unwrap();
        writer.write_value(&Value::Double(67245.375)).unwrap();
        writer.write_value(&Value::Vec2((0.0, 1.0))).unwrap();
        writer.write_value(&Value::Vec2((67245.375, 3464.85))).unwrap();
        writer.write_value(&Value::Vec3((0.0, 1.0, 2.0))).unwrap();
        writer.write_value(&Value::Vec3((67245.375, 3464.85, -8769.4565))).unwrap();
        writer.write_value(&Value::Vec4((0.0, 1.0, 2.0, 3.0))).unwrap();
        writer.write_value(&Value::Vec4((67245.375, 3464.85, -8769.4565, -1882.52))).unwrap();
        writer.write_value(&Value::Box2(((0.0, 1.0), (2.0, 3.0)))).unwrap();
        writer.write_value(&Value::Box2(((67245.375, 3464.85), (-8769.4565, -1882.52)))).unwrap();
        writer.write_value(&Value::Tag(tag!(S H A P))).unwrap();

        assert_eq!(result(writer), "\
            false \
            true \
            0 \
            6 \
            128 \
            1000 \
            -310138 \
            0.0 \
            67245.375 \
            [0.0 1.0] \
            [67245.375 3464.85] \
            [0.0 1.0 2.0] \
            [67245.375 3464.85 -8769.4565] \
            [0.0 1.0 2.0 3.0] \
            [67245.375 3464.85 -8769.4565 -1882.52] \
            [[0.0 1.0] [2.0 3.0]] \
            [[67245.375 3464.85] [-8769.4565 -1882.52]] \
            SHAP\
            ");
    }

    #[test]
    fn write_string() {
        let mut writer = setup();

        writer.write_value(&Value::String("".to_string().into_boxed_str())).unwrap();
        writer.write_value(&Value::String("Hello".to_string().into_boxed_str())).unwrap();
        writer.write_value(&Value::String("Héllø".to_string().into_boxed_str())).unwrap();
        writer.write_value(&Value::String(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, \
             sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
             Ut enim ad minim veniam".to_string().into_boxed_str())).unwrap();
        writer.write_value(&Value::String("abc\"def\"ghi".to_string().into_boxed_str())).unwrap();

        assert_eq!(result(writer), "\
            \"\" \
            \"Hello\" \
            \"Héllø\" \
            \"Lorem ipsum dolor sit amet, consectetur adipiscing elit, \
             sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
             Ut enim ad minim veniam\" \
            \"abc\"\"def\"\"ghi\"\
            ");
    }

    #[test]
    fn write_blob() {
        let mut writer = setup();

        writer.write_value(&Value::Blob(vec![].into_boxed_slice())).unwrap();
        writer.write_value(&Value::Blob(vec![0x48, 0x65, 0x0c, 0x6c, 0x6f].into_boxed_slice())).unwrap();
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

        assert_eq!(result(writer), "\
            0x \
            0x48650c6c6f \
            0x48c3a96c6cc3b8 \
            0x4c6f72656d20697073756d20646f6c6f722073697420616d65742c20636f6e73656\
            374657475722061646970697363696e6720656c69742c2073656420646f2065697573\
            6d6f642074656d706f7220696e6369646964756e74207574206c61626f72652065742\
            0646f6c6f7265206d61676e6120616c697175612e20557420656e696d206164206d69\
            6e696d2076656e69616d\
            ");
    }

    #[test]
    fn write_arrays() {
        let mut writer = setup();

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

        assert_eq!(result(writer), "\
{
} {
  true
  false
  true
} {
} {
  6
  128
  1000
} {
} {
  67245.375
  3464.85
  -8769.4565
} {
} {
  [67245.375 3464.85]
  [-8769.4565 -1882.52]
} {
} {
  [67245.375 3464.85 -8769.4565]
  [-1882.52 67245.375 3464.85]
} {
} {
  [67245.375 3464.85 -8769.4565 -1882.52]
  [-1882.52 -8769.4565 3464.85 67245.375]
} {
} {
  [[67245.375 3464.85] [-8769.4565 -1882.52]]
  [[-1882.52 -8769.4565] [3464.85 67245.375]]
}\
            ");
    }

    #[test]
    fn indentation() {
        let mut writer = setup();

        writer.write_start().unwrap();
        writer.write_value(&Value::Tag(tag!(D I C T))).unwrap();
        writer.write_value(&Value::String("one".to_string().into_boxed_str())).unwrap();
        writer.write_value(&Value::Int(1)).unwrap();
        writer.write_value(&Value::String("two".to_string().into_boxed_str())).unwrap();
        writer.write_start().unwrap();
        writer.write_value(&Value::Tag(tag!(A B C D))).unwrap();
        writer.write_value(&Value::Vec2Array(vec![
            (1.0, 2.0),
            (3.0, 4.0),
            (5.0, 6.0),
        ].into_boxed_slice())).unwrap();
        writer.write_end().unwrap();
        writer.write_value(&Value::String("three".to_string().into_boxed_str())).unwrap();
        writer.write_value(&Value::Bool(false)).unwrap();
        writer.write_end().unwrap();

        assert_eq!(result(writer),
r#"(DICT "one" 1 "two"
  (ABCD {
      [1.0 2.0]
      [3.0 4.0]
      [5.0 6.0]
    }) "three" false)"#);
    }
}
