use std::io::{self, Read, Write};
use data::{Tag, Value, Vec3, Reader, Writer};
use super::{Model, Path, Point};

const SHAPE: Tag = tag!(S H A P);
const PATHS: Tag = tag!(P T H S);
const COLOUR: Tag = tag!(C O L R);
const POINTS: Tag = tag!(P N T S);

struct ModelReader<'a, R: 'a> {
    reader: &'a mut Reader<R>
}

impl<'a, R: Read> ModelReader<'a, R> {
    pub fn new(reader: &'a mut Reader<R>) -> ModelReader<'a, R> {
        ModelReader {
            reader: reader
        }
    }

    pub fn read_model(&mut self) -> io::Result<Model> {
        try!(self.reader.expect_start());
        if try!(self.reader.expect_tag()) != SHAPE {
            return Err(io::Error::new(io::ErrorKind::Other, "Unexpected data"));
        }

        let mut paths = None;

        while try!(self.reader.expect_start_or_end()) {
            match try!(self.reader.expect_tag()) {
                PATHS => paths = Some(try!(self.read_paths())),
                _ => try!(self.reader.skip_to_end())
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

    fn read_paths(&mut self) -> io::Result<Vec<Path>> {
        let count = try!(self.reader.expect_int());
        let mut paths = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let path = try!(self.read_path());
            paths.push(path);
        }

        try!(self.reader.skip_to_end());

        Ok(paths)
    }

    fn read_path(&mut self) -> io::Result<Path> {
        let mut colour = None;
        let mut points = None;

        try!(self.reader.expect_start());

        while try!(self.reader.expect_start_or_end()) {
            match try!(self.reader.expect_tag()) {
                COLOUR => colour = Some(try!(self.read_colour())),
                POINTS => points = Some(try!(self.read_points())),
                _ => try!(self.reader.skip_to_end())
            }
        }

        if let (Some(colour), Some(points)) = (colour, points) {
            Ok(Path {
                colour: colour,
                points: points
            })
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Unexpected data"))
        }
    }

    fn read_colour(&mut self) -> io::Result<Vec3> {
        let colour = try!(self.reader.expect_vec3());
        try!(self.reader.skip_to_end());

        Ok(colour)
    }

    fn read_points(&mut self) -> io::Result<Vec<Point>> {
        let locations = try!(self.reader.expect_vec2_array());
        let biases = try!(self.reader.expect_double_array());

        try!(self.reader.skip_to_end());

        Ok(locations.iter()
            .zip(biases.iter())
            .map(|(&l, &b)| Point {
              location: l,
              curve_bias: b
            })
            .collect())
    }
}

struct ModelWriter<'a, W: 'a> {
    writer: &'a mut Writer<W>
}

impl<'a, W: Write> ModelWriter<'a, W> {
    pub fn new(writer: &'a mut Writer<W>) -> ModelWriter<'a, W> {
        ModelWriter {
            writer: writer
        }
    }

    pub fn write_model(&mut self, model: &Model) -> io::Result<()> {
        try!(self.writer.write_start());
        try!(self.writer.write_value(&Value::Tag(SHAPE)));
        try!(self.write_paths(&model.paths));
        try!(self.writer.write_end());
        Ok(())
    }

    fn write_paths(&mut self, paths: &Vec<Path>) -> io::Result<()> {
        try!(self.writer.write_start());
        try!(self.writer.write_value(&Value::Tag(PATHS)));
        try!(self.writer.write_value(&Value::Int(paths.len() as i32)));

        for path in paths {
            try!(self.write_path(path));
        }

        try!(self.writer.write_end());
        Ok(())
    }

    fn write_path(&mut self, path: &Path) -> io::Result<()> {
        try!(self.writer.write_start());

        try!(self.writer.write_start());
        try!(self.writer.write_value(&Value::Tag(COLOUR)));
        try!(self.writer.write_value(&Value::Vec3(path.colour)));
        try!(self.writer.write_end());

        try!(self.writer.write_start());
        try!(self.writer.write_value(&Value::Tag(POINTS)));

        let locations = path.points.iter()
            .map(|p| p.location)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let curve_biases = path.points.iter()
            .map(|p| p.curve_bias)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        try!(self.writer.write_value(&Value::Vec2Array(locations)));
        try!(self.writer.write_value(&Value::DoubleArray(curve_biases)));
        try!(self.writer.write_end());

        try!(self.writer.write_end());
        Ok(())
    }
}

pub fn read_model<R: Read>(input: R) -> io::Result<Model> {
    let mut reader = Reader::new(input);
    ModelReader::new(&mut reader).read_model()
}

pub fn write_model<W: Write>(output: W, model: &Model) -> io::Result<()> {
    let mut writer = Writer::new(output);
    ModelWriter::new(&mut writer).write_model(model)
}
