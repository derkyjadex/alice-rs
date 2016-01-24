use std::io::{self};
use super::{Value, Tag, Vec2, Vec3, Vec4, Box2};

#[derive(PartialEq, Debug)]
pub enum Token {
    Start,
    End,
    Value(Value),
    EndOfFile
}

fn unexpected<T>(msg: &str) -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::InvalidData, msg))
}

macro_rules! expect {
    ($_self:ident, $_enum:ident) => {
        match try!($_self.read_next()) {
            Token::Value(Value::$_enum(v)) => Ok(v),
            _ => unexpected(concat!("Expected ", stringify!($_enum)))
        }
    }
}

macro_rules! expect_or_end {
    ($_self:ident, $_enum:ident) => {
        match try!($_self.read_next()) {
            Token::Value(Value::$_enum(v)) => Ok(Some(v)),
            Token::End => Ok(None),
            _ => unexpected(concat!("Expected ", stringify!($_enum), " or end of group"))
        }
    }
}

pub trait Reader {
    fn read_next(&mut self) -> io::Result<Token>;

    fn skip_to_end(&mut self) -> io::Result<()> {
        let mut levels = 0;
        loop {
            match try!(self.read_next()) {
                Token::Value(_) => (),
                Token::Start => levels += 1,
                Token::End if levels == 0 => break,
                Token::End => levels -= 1,
                Token::EndOfFile if levels == 0 => break,
                Token::EndOfFile => return unexpected("Unexpected end of file")
            }
        }

        Ok(())
    }

    fn expect_start(&mut self) -> io::Result<()> {
        match try!(self.read_next()) {
            Token::Start => Ok(()),
            _ => unexpected("Expected start of group")
        }
    }

    fn expect_start_or_end(&mut self) -> io::Result<bool> {
        match try!(self.read_next()) {
            Token::Start => Ok(true),
            Token::End => Ok(false),
            Token::EndOfFile => Ok(false),
            _ => unexpected("Expected start or end of group")
        }
    }

    fn expect_tag(&mut self) -> io::Result<Tag> {
        expect!(self, Tag)
    }

    fn expect_tag_or_end(&mut self) -> io::Result<Option<Tag>> {
        expect_or_end!(self, Tag)
    }

    fn expect_bool(&mut self) -> io::Result<bool> {
        expect!(self, Bool)
    }

    fn expect_bool_or_end(&mut self) -> io::Result<Option<bool>> {
        expect_or_end!(self, Bool)
    }

    fn expect_bool_array(&mut self) -> io::Result<Box<[bool]>> {
        expect!(self, BoolArray)
    }

    fn expect_bool_array_or_end(&mut self) -> io::Result<Option<Box<[bool]>>> {
        expect_or_end!(self, BoolArray)
    }

    fn expect_int(&mut self) -> io::Result<i32> {
        expect!(self, Int)
    }

    fn expect_int_or_end(&mut self) -> io::Result<Option<i32>> {
        expect_or_end!(self, Int)
    }

    fn expect_int_array(&mut self) -> io::Result<Box<[i32]>> {
        expect!(self, IntArray)
    }

    fn expect_int_array_or_end(&mut self) -> io::Result<Option<Box<[i32]>>> {
        expect_or_end!(self, IntArray)
    }

    fn expect_double(&mut self) -> io::Result<f64> {
        expect!(self, Double)
    }

    fn expect_double_or_end(&mut self) -> io::Result<Option<f64>> {
        expect_or_end!(self, Double)
    }

    fn expect_double_array(&mut self) -> io::Result<Box<[f64]>> {
        expect!(self, DoubleArray)
    }

    fn expect_double_array_or_end(&mut self) -> io::Result<Option<Box<[f64]>>> {
        expect_or_end!(self, DoubleArray)
    }

    fn expect_vec2(&mut self) -> io::Result<Vec2> {
        expect!(self, Vec2)
    }

    fn expect_vec2_or_end(&mut self) -> io::Result<Option<Vec2>> {
        expect_or_end!(self, Vec2)
    }

    fn expect_vec2_array(&mut self) -> io::Result<Box<[Vec2]>> {
        expect!(self, Vec2Array)
    }

    fn expect_vec2_array_or_end(&mut self) -> io::Result<Option<Box<[Vec2]>>> {
        expect_or_end!(self, Vec2Array)
    }

    fn expect_vec3(&mut self) -> io::Result<Vec3> {
        expect!(self, Vec3)
    }

    fn expect_vec3_or_end(&mut self) -> io::Result<Option<Vec3>> {
        expect_or_end!(self, Vec3)
    }

    fn expect_vec3_array(&mut self) -> io::Result<Box<[Vec3]>> {
        expect!(self, Vec3Array)
    }

    fn expect_vec3_array_or_end(&mut self) -> io::Result<Option<Box<[Vec3]>>> {
        expect_or_end!(self, Vec3Array)
    }

    fn expect_vec4(&mut self) -> io::Result<Vec4> {
        expect!(self, Vec4)
    }

    fn expect_vec4_or_end(&mut self) -> io::Result<Option<Vec4>> {
        expect_or_end!(self, Vec4)
    }

    fn expect_vec4_array(&mut self) -> io::Result<Box<[Vec4]>> {
        expect!(self, Vec4Array)
    }

    fn expect_vec4_array_or_end(&mut self) -> io::Result<Option<Box<[Vec4]>>> {
        expect_or_end!(self, Vec4Array)
    }

    fn expect_box2(&mut self) -> io::Result<Box2> {
        expect!(self, Box2)
    }

    fn expect_box2_or_end(&mut self) -> io::Result<Option<Box2>> {
        expect_or_end!(self, Box2)
    }

    fn expect_box2_array(&mut self) -> io::Result<Box<[Box2]>> {
        expect!(self, Box2Array)
    }

    fn expect_box2_array_or_end(&mut self) -> io::Result<Option<Box<[Box2]>>> {
        expect_or_end!(self, Box2Array)
    }

    fn expect_string(&mut self) -> io::Result<Box<str>> {
        expect!(self, String)
    }

    fn expect_string_or_end(&mut self) -> io::Result<Option<Box<str>>> {
        expect_or_end!(self, String)
    }

    fn expect_blob(&mut self) -> io::Result<Box<[u8]>> {
        expect!(self, Blob)
    }
    fn expect_blob_or_end(&mut self) -> io::Result<Option<Box<[u8]>>> {
        expect_or_end!(self, Blob)
    }
}
