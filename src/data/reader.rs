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
        match try!(self.read_next()) {
            Token::Value(Value::Tag(t)) => Ok(t),
            _ => unexpected("Expected tag")
        }
    }

    fn expect_bool(&mut self) -> io::Result<bool> {
        match try!(self.read_next()) {
            Token::Value(Value::Bool(b)) => Ok(b),
            _ => unexpected("Expected bool")
        }
    }

    fn expect_bool_array(&mut self) -> io::Result<Box<[bool]>> {
        match try!(self.read_next()) {
            Token::Value(Value::BoolArray(a)) => Ok(a),
            _ => unexpected("Expected bool array")
        }
    }

    fn expect_int(&mut self) -> io::Result<i32> {
        match try!(self.read_next()) {
            Token::Value(Value::Int(i)) => Ok(i),
            _ => unexpected("Expected int")
        }
    }

    fn expect_int_array(&mut self) -> io::Result<Box<[i32]>> {
        match try!(self.read_next()) {
            Token::Value(Value::IntArray(a)) => Ok(a),
            _ => unexpected("Expected int array")
        }
    }

    fn expect_double(&mut self) -> io::Result<f64> {
        match try!(self.read_next()) {
            Token::Value(Value::Double(d)) => Ok(d),
            _ => unexpected("Expected double")
        }
    }

    fn expect_double_array(&mut self) -> io::Result<Box<[f64]>> {
        match try!(self.read_next()) {
            Token::Value(Value::DoubleArray(a)) => Ok(a),
            _ => unexpected("Expected double array")
        }
    }

    fn expect_vec2(&mut self) -> io::Result<Vec2> {
        match try!(self.read_next()) {
            Token::Value(Value::Vec2(v)) => Ok(v),
            _ => unexpected("Expected vec2")
        }
    }

    fn expect_vec2_array(&mut self) -> io::Result<Box<[Vec2]>> {
        match try!(self.read_next()) {
            Token::Value(Value::Vec2Array(a)) => Ok(a),
            _ => unexpected("Expected vec2 array")
        }
    }

    fn expect_vec3(&mut self) -> io::Result<Vec3> {
        match try!(self.read_next()) {
            Token::Value(Value::Vec3(v)) => Ok(v),
            _ => unexpected("Expected vec3")
        }
    }

    fn expect_vec3_array(&mut self) -> io::Result<Box<[Vec3]>> {
        match try!(self.read_next()) {
            Token::Value(Value::Vec3Array(a)) => Ok(a),
            _ => unexpected("Expected vec3 array")
        }
    }

    fn expect_vec4(&mut self) -> io::Result<Vec4> {
        match try!(self.read_next()) {
            Token::Value(Value::Vec4(v)) => Ok(v),
            _ => unexpected("Expected vec4")
        }
    }

    fn expect_vec4_array(&mut self) -> io::Result<Box<[Vec4]>> {
        match try!(self.read_next()) {
            Token::Value(Value::Vec4Array(a)) => Ok(a),
            _ => unexpected("Expected vec4 array")
        }
    }

    fn expect_box2(&mut self) -> io::Result<Box2> {
        match try!(self.read_next()) {
            Token::Value(Value::Box2(b)) => Ok(b),
            _ => unexpected("Expected box2")
        }
    }

    fn expect_box2_array(&mut self) -> io::Result<Box<[Box2]>> {
        match try!(self.read_next()) {
            Token::Value(Value::Box2Array(a)) => Ok(a),
            _ => unexpected("Expected box2 array")
        }
    }

    fn expect_string(&mut self) -> io::Result<Box<str>> {
        match try!(self.read_next()) {
            Token::Value(Value::String(s)) => Ok(s),
            _ => unexpected("Expected string")
        }
    }

    fn expect_blob(&mut self) -> io::Result<Box<[u8]>> {
        match try!(self.read_next()) {
            Token::Value(Value::Blob(b)) => Ok(b),
            _ => unexpected("Expected blob")
        }
    }
}
