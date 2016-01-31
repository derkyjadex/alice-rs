use std::io::{self, Read};
use super::*;

fn from_hex(b: u8) -> u8 {
    match b {
        b'0' ... b'9' =>
            b - b'0',
        b'a' ... b'f' =>
            b - b'a' + 10,
        _ => b
    }
}

#[derive(Debug)]
enum SubToken {
    Start, End,
    VecStart, VecEnd,
    ArrayStart, ArrayEnd,
    Tag(u32),
    Bool(bool),
    Int(i32),
    Double(f64),
    String(String),
    Blob(Vec<u8>),
    EndOfFile
}

struct SubReader<R> {
    input: R,
    line: i32,
    pos: i32,
    last: Option<u8>
}

impl<R: Read> SubReader<R> {
    fn new(input: R) -> SubReader<R> {
        SubReader {
            input: input,
            line: 1,
            pos: 0,
            last: None
        }
    }

    fn invalid_token<T>(&self) -> io::Result<T> {
        let msg = format!("Invalid token at {}:{}", self.line, self.pos);
        Err(io::Error::new(io::ErrorKind::InvalidData, msg))
    }

    fn next_byte(&mut self) -> io::Result<Option<u8>> {
        if self.last == None {
            let mut buffer = [0; 1];
            let n = try!(self.input.read(&mut buffer));

            self.last = if n == 0 {
                None
            } else {
                if buffer[0] == b'\n' {
                    self.line += 1;
                    self.pos = 0;
                } else {
                    self.pos += 1;
                }
                Some(buffer[0])
            }
        }

        Ok(self.last)
    }

    fn consume(&mut self) {
        self.last = None
    }

    fn read_next(&mut self) -> io::Result<SubToken> {
        while let Some(b) = try!(self.next_byte()) {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' =>
                    self.consume(),
                b'(' => {
                    self.consume();
                    return Ok(SubToken::Start);
                },
                b')' => {
                    self.consume();
                    return Ok(SubToken::End);
                },
                b'[' => {
                    self.consume();
                    return Ok(SubToken::VecStart);
                },
                b']' => {
                    self.consume();
                    return Ok(SubToken::VecEnd);
                },
                b'{' => {
                    self.consume();
                    return Ok(SubToken::ArrayStart);
                },
                b'}' => {
                    self.consume();
                    return Ok(SubToken::ArrayEnd);
                },
                b'"' => {
                    self.consume();
                    return self.read_string()
                },
                b'A' ... b'Z' =>
                    return self.read_tag(),

                b'0' ... b'9' | b'-' =>
                    return self.read_number(),

                b't' => return self.read_bool(b"true", true),
                b'f' => return self.read_bool(b"false", false),
                _ =>
                    return self.invalid_token()
            }
        }

        Ok(SubToken::EndOfFile)
    }

    fn read_tag(&mut self) -> io::Result<SubToken> {
        let mut count = 0;
        let mut tag = 0;

        while let Some(b) = try!(self.next_byte()) {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' |
                b')' | b']' | b'}' if count == 4 =>
                    return Ok(SubToken::Tag(tag)),

                b'A' ... b'Z' | b'0' ... b'9' | b'_' if count < 4 => {
                    count += 1;
                    tag = tag << 8 | b as u32;
                    self.consume();
                },
                _ => return self.invalid_token()
            }
        }

        self.invalid_token()
    }

    fn read_bool(&mut self, expected: &[u8], result: bool) -> io::Result<SubToken> {
        for &b in expected {
            if try!(self.next_byte()) != Some(b) {
                return self.invalid_token();
            }
            self.consume();
        }

        if let Some(b) = try!(self.next_byte()) {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' |
                b')' | b']' | b'}' =>
                    return Ok(SubToken::Bool(result)),

                _ => return self.invalid_token()
            }
        }

        self.invalid_token()
    }

    fn read_number(&mut self) -> io::Result<SubToken> {
        let mut buffer = Vec::new();

        while let Some(b) = try!(self.next_byte()) {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' |
                b')' | b']' | b'}' =>
                    return Ok(SubToken::Int(String::from_utf8(buffer).unwrap().parse().unwrap())),

                b'-' if buffer.len() == 0 => {
                    buffer.push(b);
                    self.consume();
                },
                b'0' ... b'9' => {
                    buffer.push(b);
                    self.consume();
                },
                b'.' if buffer.len() > 0 && buffer.last() != Some(&b'-') => {
                    buffer.push(b);
                    self.consume();
                    return self.read_double(buffer);
                },
                b'x' if buffer.len() == 1 && buffer.last() == Some(&b'0') => {
                    self.consume();
                    return self.read_blob();
                }
                _ => return self.invalid_token()
            }
        }

        self.invalid_token()
    }

    fn read_double(&mut self, buffer: Vec<u8>) -> io::Result<SubToken> {
        let mut buffer = buffer;
        let mut has_exp = false;

        while let Some(b) = try!(self.next_byte()) {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' |
                b')' | b']' | b'}' =>
                    return Ok(SubToken::Double(String::from_utf8(buffer).unwrap().parse().unwrap())),

                b'0' ... b'9' => {
                    buffer.push(b);
                    self.consume();
                },
                b'e' if !has_exp => {
                    has_exp = true;
                    buffer.push(b);
                    self.consume();
                },
                b'-' if buffer.last() == Some(&b'e') => {
                    buffer.push(b);
                    self.consume();
                },
                _ => return self.invalid_token()
            }
        }

        self.invalid_token()
    }

    fn read_blob(&mut self) -> io::Result<SubToken> {
        let mut buffer = Vec::new();
        let mut top_half = None;

        while let Some(b) = try!(self.next_byte()) {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' |
                b')' | b']' | b'}' if top_half == None =>
                    return Ok(SubToken::Blob(buffer)),

                b'0' ... b'9' | b'a' ... b'f' => {
                    let v = from_hex(b);
                    if let Some(top) = top_half {
                        buffer.push(top | v);
                        top_half = None;
                    } else {
                        top_half = Some(v << 4);
                    }
                    self.consume()
                },
                _ => return self.invalid_token()
            }
        }

        self.invalid_token()
    }

    fn read_string(&mut self) -> io::Result<SubToken> {
        let mut buffer = Vec::new();
        let mut escaped = false;

        while let Some(b) = try!(self.next_byte()) {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' |
                b')' | b']' | b'}' if escaped =>
                    return Ok(SubToken::String(String::from_utf8(buffer).unwrap())),

                b'"' if escaped => {
                    escaped = false;
                    buffer.push(b);
                    self.consume();
                },
                _ if escaped =>
                    return self.invalid_token(),

                b'"' => {
                    escaped = true;
                    self.consume();
                },
                _ => {
                    buffer.push(b);
                    self.consume();
                }
            }
        }

        self.invalid_token()
    }
}

pub struct TextReader<R> {
    sub: SubReader<R>
}

impl<R: Read> TextReader<R> {
    pub fn new(input: R) -> TextReader<R> {
        TextReader {
            sub: SubReader::new(input)
        }
    }

    fn invalid_token<T>(&self) -> io::Result<T> {
        let msg = format!("Invalid token at {}:{}", self.sub.line, self.sub.pos);
        Err(io::Error::new(io::ErrorKind::InvalidData, msg))
    }

    fn read_vec(&mut self) -> io::Result<Token> {
        let mut xs = Vec::new();

        loop {
            match try!(self.sub.read_next()) {
                SubToken::VecEnd => match xs.len() {
                    2 => return Ok(Token::Value(Value::Vec2((xs[0], xs[1])))),
                    3 => return Ok(Token::Value(Value::Vec3((xs[0], xs[1], xs[2])))),
                    4 => return Ok(Token::Value(Value::Vec4((xs[0], xs[1], xs[2], xs[3])))),
                    _ => return self.invalid_token()
                },
                SubToken::VecStart if xs.len() == 0 =>
                    return self.read_box2(),
                SubToken::Double(v) if xs.len() < 4 =>
                    xs.push(v),
                _ => return self.invalid_token()
            }
        }
    }

    fn read_vec2(&mut self) -> io::Result<Vec2> {
        if let Token::Value(Value::Vec2(v)) = try!(self.read_vec()) {
            Ok(v)
        } else {
            self.invalid_token()
        }
    }

    fn read_box2(&mut self) -> io::Result<Token> {
        let min = try!(self.read_vec2());
        match try!(self.sub.read_next()) {
            SubToken::VecStart => {
                let max = try!(self.read_vec2());
                match try!(self.sub.read_next()) {
                    SubToken::VecEnd =>
                        Ok(Token::Value(Value::Box2((min, max)))),
                    _ => self.invalid_token()
                }
            },
            _ => self.invalid_token()
        }
    }

    fn read_array(&mut self) -> io::Result<Token> {
        match try!(self.sub.read_next()) {
            SubToken::Bool(v) => return self.read_bool_array(v),
            SubToken::Int(v) => return self.read_int_array(v),
            SubToken::Double(v) => return self.read_double_array(v),
            SubToken::VecStart => match try!(self.read_vec()) {
                Token::Value(Value::Vec2(v)) => return self.read_vec2_array(v),
                Token::Value(Value::Vec3(v)) => return self.read_vec3_array(v),
                Token::Value(Value::Vec4(v)) => return self.read_vec4_array(v),
                Token::Value(Value::Box2(v)) => return self.read_box2_array(v),
                _ => self.invalid_token()
            },
            _ => self.invalid_token()
        }
    }

    fn read_bool_array(&mut self, first: bool) -> io::Result<Token> {
        let mut values = vec![first];
        loop {
            match try!(self.sub.read_next()) {
                SubToken::Bool(v) => values.push(v),
                SubToken::ArrayEnd =>
                    return Ok(Token::Value(Value::BoolArray(values.into_boxed_slice()))),
                _ => return self.invalid_token()
            }
        }
    }

    fn read_int_array(&mut self, first: i32) -> io::Result<Token> {
        let mut values = vec![first];
        loop {
            match try!(self.sub.read_next()) {
                SubToken::Int(v) => values.push(v),
                SubToken::ArrayEnd =>
                    return Ok(Token::Value(Value::IntArray(values.into_boxed_slice()))),
                _ => return self.invalid_token()
            }
        }
    }

    fn read_double_array(&mut self, first: f64) -> io::Result<Token> {
        let mut values = vec![first];
        loop {
            match try!(self.sub.read_next()) {
                SubToken::Double(v) => values.push(v),
                SubToken::ArrayEnd =>
                    return Ok(Token::Value(Value::DoubleArray(values.into_boxed_slice()))),
                _ => return self.invalid_token()
            }
        }
    }

    fn read_vec2_array(&mut self, first: Vec2) -> io::Result<Token> {
        let mut values = vec![first];
        loop {
            match try!(self.sub.read_next()) {
                SubToken::VecStart =>
                    if let Token::Value(Value::Vec2(v)) = try!(self.read_vec()) {
                        values.push(v)
                    } else {
                        return self.invalid_token()
                    },
                SubToken::ArrayEnd =>
                    return Ok(Token::Value(Value::Vec2Array(values.into_boxed_slice()))),
                _ => return self.invalid_token()
            }
        }
    }

    fn read_vec3_array(&mut self, first: Vec3) -> io::Result<Token> {
        let mut values = vec![first];
        loop {
            match try!(self.sub.read_next()) {
                SubToken::VecStart =>
                    if let Token::Value(Value::Vec3(v)) = try!(self.read_vec()) {
                        values.push(v)
                    } else {
                        return self.invalid_token()
                    },
                SubToken::ArrayEnd =>
                    return Ok(Token::Value(Value::Vec3Array(values.into_boxed_slice()))),
                _ => return self.invalid_token()
            }
        }
    }

    fn read_vec4_array(&mut self, first: Vec4) -> io::Result<Token> {
        let mut values = vec![first];
        loop {
            match try!(self.sub.read_next()) {
                SubToken::VecStart =>
                    if let Token::Value(Value::Vec4(v)) = try!(self.read_vec()) {
                        values.push(v)
                    } else {
                        return self.invalid_token()
                    },
                SubToken::ArrayEnd =>
                    return Ok(Token::Value(Value::Vec4Array(values.into_boxed_slice()))),
                _ => return self.invalid_token()
            }
        }
    }

    fn read_box2_array(&mut self, first: Box2) -> io::Result<Token> {
        let mut values = vec![first];
        loop {
            match try!(self.sub.read_next()) {
                SubToken::VecStart =>
                    if let Token::Value(Value::Box2(v)) = try!(self.read_vec()) {
                        values.push(v)
                    } else {
                        return self.invalid_token()
                    },
                SubToken::ArrayEnd =>
                    return Ok(Token::Value(Value::Box2Array(values.into_boxed_slice()))),
                _ => return self.invalid_token()
            }
        }
    }
}

impl<R: Read> Reader for TextReader<R> {
    fn read_next(&mut self) -> io::Result<Token> {
        match try!(self.sub.read_next()) {
            SubToken::Start => return Ok(Token::Start),
            SubToken::End => return Ok(Token::End),
            SubToken::EndOfFile => return Ok(Token::EndOfFile),
            SubToken::Tag(tag) => return Ok(Token::Value(Value::Tag(tag))),
            SubToken::Bool(v) => return Ok(Token::Value(Value::Bool(v))),
            SubToken::Int(v) => return Ok(Token::Value(Value::Int(v))),
            SubToken::Double(v) => return Ok(Token::Value(Value::Double(v))),
            SubToken::String(v) => return Ok(Token::Value(Value::String(v.into_boxed_str()))),
            SubToken::Blob(v) => return Ok(Token::Value(Value::Blob(v.into_boxed_slice()))),
            SubToken::VecStart => return self.read_vec(),
            SubToken::ArrayStart => return self.read_array(),
            SubToken::VecEnd | SubToken::ArrayEnd =>
                return self.invalid_token()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Value, Token, Reader};
    use std::io::{self, Cursor};

    fn setup(data: &[u8]) -> TextReader<Cursor<&[u8]>> {
        TextReader::new(Cursor::new(data))
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
        let mut reader = setup(b"()");

        assert!(is_token(reader.read_next(), Token::Start));
        assert!(is_token(reader.read_next(), Token::End));
        assert!(is_token(reader.read_next(), Token::EndOfFile));
    }

    #[test]
    fn read_simple_values() {
        let mut reader = setup(br"
            false true
            0 6 128 1000 -310138
            0.0 1.0 -67245.375 -1.25e123 1.25e-123
            [67245.375 3464.85]
            [67245.375 3464.85 -8769.4565]
            [67245.375 3464.85 -8769.4565 -1882.52]
            [[67245.375 3464.85] [-8769.4565 -1882.52]]
            SHAP ");

        assert!(is_value(reader.read_next(), Value::Bool(false)));
        assert!(is_value(reader.read_next(), Value::Bool(true)));
        assert!(is_value(reader.read_next(), Value::Int(0)));
        assert!(is_value(reader.read_next(), Value::Int(6)));
        assert!(is_value(reader.read_next(), Value::Int(128)));
        assert!(is_value(reader.read_next(), Value::Int(1000)));
        assert!(is_value(reader.read_next(), Value::Int(-310138)));
        assert!(is_value(reader.read_next(), Value::Double(0.0)));
        assert!(is_value(reader.read_next(), Value::Double(1.0)));
        assert!(is_value(reader.read_next(), Value::Double(-67245.375)));
        assert!(is_value(reader.read_next(), Value::Double(-1.25e123)));
        assert!(is_value(reader.read_next(), Value::Double(1.25e-123)));
        assert!(is_value(reader.read_next(), Value::Vec2((67245.375, 3464.85))));
        assert!(is_value(reader.read_next(), Value::Vec3((67245.375, 3464.85, -8769.4565))));
        assert!(is_value(reader.read_next(), Value::Vec4((67245.375, 3464.85, -8769.4565, -1882.52))));
        assert!(is_value(reader.read_next(), Value::Box2(((67245.375, 3464.85), (-8769.4565, -1882.52)))));
        assert!(is_value(reader.read_next(), Value::Tag(tag!(S H A P))));
        assert!(is_token(reader.read_next(), Token::EndOfFile));
    }

    #[test]
    fn read_string() {
        let mut reader = setup(b"
            \"\"
            \"Hello\"
            \"H\xC3\xA9ll\xC3\xB8\"
            \"one \"\"two\"\" three\"
            \"Lorem ipsum dolor sit amet, consectetur adipiscing elit, \
             sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
             Ut enim ad minim veniam\" ");

        assert!(is_string(reader.read_next(), ""));
        assert!(is_string(reader.read_next(), "Hello"));
        assert!(is_string(reader.read_next(), "Héllø"));

        let result = reader.read_next();
        println!("{:?}", result);

        assert!(is_string(result, "one \"two\" three"));
        assert!(is_string(reader.read_next(),
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, \
             sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
             Ut enim ad minim veniam"));
        assert!(is_token(reader.read_next(), Token::EndOfFile));
    }

    #[test]
    fn read_blob() {
        let mut reader = setup(br"
            0x
            0x48656c6c6f
            0x48c3a96c6cc3b8
            0x4c6f72656d20697073756d20646f6c6f722073697420616d65742c20636f6e73656374657475722061646970697363696e6720656c69742c2073656420646f20656975736d6f642074656d706f7220696e6369646964756e74207574206c61626f726520657420646f6c6f7265206d61676e6120616c697175612e20557420656e696d206164206d696e696d2076656e69616d ");

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
        let mut reader = setup(br"
            {true false true}
            {6 128 1000}
            {67245.375 3464.85 -8769.4565}
            {[67245.375 3464.85] [-8769.4565 -1882.52]}
            {[67245.375 3464.85 -8769.4565] [-1882.52 67245.375 3464.85]}
            {[67245.375 3464.85 -8769.4565 -1882.52]
             [-1882.52 -8769.4565 3464.85 67245.375]}
            {[[67245.375 3464.85] [-8769.4565 -1882.52]]
             [[-1882.52 -8769.4565] [3464.85 67245.375]]}");

        assert!(is_value(reader.read_next(), Value::BoolArray(vec![
            true, false, true
        ].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::IntArray(vec![
            6, 128, 1000
        ].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::DoubleArray(vec![
            67245.375, 3464.85, -8769.4565
        ].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::Vec2Array(vec![
            (67245.375, 3464.85),
            (-8769.4565, -1882.52)
        ].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::Vec3Array(vec![
            (67245.375, 3464.85, -8769.4565),
            (-1882.52, 67245.375, 3464.85),
        ].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::Vec4Array(vec![
            (67245.375, 3464.85, -8769.4565, -1882.52),
            (-1882.52, -8769.4565, 3464.85, 67245.375),
        ].into_boxed_slice())));
        assert!(is_value(reader.read_next(), Value::Box2Array(vec![
            ((67245.375, 3464.85), (-8769.4565, -1882.52)),
            ((-1882.52, -8769.4565), (3464.85, 67245.375)),
        ].into_boxed_slice())));
        assert!(is_token(reader.read_next(), Token::EndOfFile));
    }
}
