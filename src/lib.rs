extern crate byteorder;

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

#[macro_use]
pub mod macros;
mod reader;
mod writer;
mod text_writer;
mod utils;

pub use writer::Writer;
pub use reader::Reader;
pub use text_writer::TextWriter;
