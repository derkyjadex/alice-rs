pub mod rendering;

use std::io;
use data::{Value, Tag, Vec2, Vec3, Reader, Writer};

pub struct Model {
    pub paths: Vec<Path>
}

pub struct Path {
    pub colour: Vec3,
    pub points: Vec<Point>,
}

#[derive(Copy, Clone)]
pub struct Point {
    pub location: Vec2,
    pub curve_bias: f64
}

const SHAPE: Tag = tag!(S H A P);
const PATHS: Tag = tag!(P T H S);
const COLOUR: Tag = tag!(C O L R);
const POINTS: Tag = tag!(P N T S);

impl Model {
    pub fn write(&self, writer: &mut Writer) -> io::Result<()> {
        try!(writer.write_start());
        try!(writer.write_value(&Value::Tag(SHAPE)));

        try!(writer.write_start());
        try!(writer.write_value(&Value::Tag(PATHS)));
        try!(writer.write_value(&Value::Int(self.paths.len() as i32)));

        for ref path in &self.paths {
            try!(path.write(writer));
        }

        try!(writer.write_end());

        try!(writer.write_end());
        Ok(())
    }

    pub fn read(reader: &mut Reader) -> io::Result<Model> {
        try!(reader.expect_start());
        Model::read_started(reader)
    }

    pub fn read_started(reader: &mut Reader) -> io::Result<Model> {
        if try!(reader.expect_tag()) != SHAPE {
            return Err(io::Error::new(io::ErrorKind::Other, "Unexpected data"));
        }

        let mut paths = None;

        while try!(reader.expect_start_or_end()) {
            match try!(reader.expect_tag()) {
                PATHS => paths = Some(try!(read_paths(reader))),
                _ => try!(reader.skip_to_end())
            }
        }

        if let Some(paths) = paths {
            Ok(Model {
                paths: paths
            })
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Unexpected data"))
        }
    }
}

fn read_paths(reader: &mut Reader) -> io::Result<Vec<Path>> {
    let count = try!(reader.expect_int());
    let mut paths = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let path = try!(Path::read(reader));
        paths.push(path);
    }

    try!(reader.skip_to_end());

    Ok(paths)
}

impl Path {
    fn write(&self, writer: &mut Writer) -> io::Result<()> {
        try!(writer.write_start());

        try!(writer.write_start());
        try!(writer.write_value(&Value::Tag(COLOUR)));
        try!(writer.write_value(&Value::Vec3(self.colour)));
        try!(writer.write_end());

        try!(writer.write_start());
        try!(writer.write_value(&Value::Tag(POINTS)));

        let locations = self.points.iter()
            .map(|p| p.location)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let curve_biases = self.points.iter()
            .map(|p| p.curve_bias)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        try!(writer.write_value(&Value::Vec2Array(locations)));
        try!(writer.write_value(&Value::DoubleArray(curve_biases)));
        try!(writer.write_end());

        try!(writer.write_end());
        Ok(())
    }

    fn read(reader: &mut Reader) -> io::Result<Path> {
        let mut colour = None;
        let mut points = None;

        try!(reader.expect_start());

        while try!(reader.expect_start_or_end()) {
            match try!(reader.expect_tag()) {
                COLOUR => colour = Some(try!(read_colour(reader))),
                POINTS => points = Some(try!(read_points(reader))),
                _ => try!(reader.skip_to_end())
            }
        }

        if let (Some(colour), Some(points)) = (colour, points) {
            Ok(Path {
                colour: colour,
                points: points
            })
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, "Unexpected data"))
        }
    }
}

fn read_colour(reader: &mut Reader) -> io::Result<Vec3> {
    let colour = try!(reader.expect_vec3());
    try!(reader.skip_to_end());

    Ok(colour)
}

fn read_points(reader: &mut Reader) -> io::Result<Vec<Point>> {
    let locations = try!(reader.expect_vec2_array());
    let biases = try!(reader.expect_double_array());

    try!(reader.skip_to_end());

    Ok(locations.iter()
        .zip(biases.iter())
        .map(|(&l, &b)| Point {
          location: l,
          curve_bias: b
        })
        .collect())
}
