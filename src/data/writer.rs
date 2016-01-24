use std::io::{self};
use super::Value;

pub trait Writer {
    fn write_start(&mut self) -> io::Result<()>;
    fn write_end(&mut self) -> io::Result<()>;
    fn write_value(&mut self, value: &Value) -> io::Result<()>;
}
