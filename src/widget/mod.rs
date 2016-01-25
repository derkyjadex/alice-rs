pub mod rendering;

use std::io::{Result, Error, ErrorKind};
use super::data::{Value, Tag, Vec2, Vec3, Vec4, Box2, Reader, Writer};
use super::model::Model;

pub enum Element {
     Widget(Widget),
     Group(Group),
     Grid(Grid),
     Model(ModelElement),
     Text(Text)
}

#[derive(Default)]
pub struct Widget {
    pub location: Vec2,
    pub size: Vec2,
    pub fill_colour: Vec4,
    pub border_colour: Vec3,
    pub border_width: i32,

    pub bindings: Vec<(Event, Binding)>,
    pub children: Vec<Element>,
}

#[derive(Default)]
pub struct Group {
    pub location: Vec2,
    pub children: Vec<Element>
}

#[derive(Default)]
pub struct Grid {
    pub bounds: Box2,
    pub size: Vec2,
    pub offset: Vec2,
    pub colour: Vec3
}

pub struct ModelElement {
     pub location: Vec2,
     pub scale: f64,
     pub model: Model,
}

#[derive(Default)]
pub struct Text {
    pub location: Vec2,
    pub size: f64,
    pub colour: Vec3,
    pub value: String,
}

#[derive(Copy, Clone)]
pub enum Event {
    Down,
    Up,
    Motion,
    Key,
    Text,
    KeyboardFocusLost
}

pub type Binding = i32;

const WIDGET: Tag = tag!(W D G T);
const GROUP: Tag = tag!(G R U P);
const GRID: Tag = tag!(G R I D);
const MODEL: Tag = tag!(M O D L);
const TEXT: Tag = tag!(T E X T);

const DOWN: Tag = tag!(D O W N);
const UP: Tag = tag!(U P _ _);
const MOTION: Tag = tag!(M O T N);
const KEY: Tag = tag!(K E Y _);
const KEYBOARD_FOCUS_LOST: Tag = tag!(K L S T);

impl Element {
    pub fn write(&self, writer: &mut Writer) -> Result<()> {
        match self {
            &Element::Widget(ref widget) => widget.write(writer),
            &Element::Group(ref group) => group.write(writer),
            &Element::Grid(ref grid) => grid.write(writer),
            &Element::Model(ref model) => model.write(writer),
            &Element::Text(ref text) => text.write(writer),
        }
    }
}

impl Widget {
    fn write(&self, writer: &mut Writer) -> Result<()> {
        try!(writer.write_start());
        try!(writer.write_value(&Value::Tag(WIDGET)));

        try!(writer.write_start());

        try!(writer.write_value(&Value::Vec2(self.location)));
        try!(writer.write_value(&Value::Vec2(self.size)));
        try!(writer.write_value(&Value::Vec4(self.fill_colour)));

        try!(writer.write_start());
        if self.border_width > 0 {
          try!(writer.write_value(&Value::Vec3(self.border_colour)));
          try!(writer.write_value(&Value::Int(self.border_width)));
        }
        try!(writer.write_end());

        try!(writer.write_end());

        try!(writer.write_start());
        for &(event, binding) in self.bindings.iter() {
            try!(event.write(writer));
            try!(writer.write_value(&Value::Int(binding)));
        }
        try!(writer.write_end());

        try!(writer.write_start());
        for ref child in self.children.iter() {
            try!(child.write(writer));
        }
        try!(writer.write_end());

        try!(writer.write_end());
        Ok(())
    }

    fn update(&mut self, reader: &mut Reader) -> Result<()> {
        try!(reader.expect_start());
        if let Some(location) = try!(reader.expect_vec2_or_end()) {
            self.location = location;
            self.size = try!(reader.expect_vec2());
            self.fill_colour = try!(reader.expect_vec4());

            try!(reader.expect_start());
            if let Some(border_colour) = try!(reader.expect_vec3_or_end()) {
                self.border_colour = border_colour;
                self.border_width = try!(reader.expect_int());
                try!(reader.skip_to_end());
            }

            try!(reader.skip_to_end());
        }

        try!(reader.expect_start());
        self.bindings.clear();
        while let Some(tag) = try!(reader.expect_tag_or_end()) {
            if let Some(event) = Event::from_tag(tag) {
                let binding = try!(reader.expect_int());
                self.bindings.push((event, binding));
            } else {
                return Err(Error::new(ErrorKind::InvalidData, "Unknown event type"))
            }
        }

        try!(reader.expect_start());
        try!(update_children(&mut self.children, reader));

        reader.skip_to_end()
    }
}

impl Group {
    fn write(&self, writer: &mut Writer) -> Result<()> {
        try!(writer.write_start());
        try!(writer.write_value(&Value::Tag(GROUP)));
        try!(writer.write_value(&Value::Vec2(self.location)));

        try!(writer.write_start());
        for ref child in self.children.iter() {
            try!(child.write(writer));
        }
        try!(writer.write_end());

        try!(writer.write_end());
        Ok(())
    }

    fn update(&mut self, reader: &mut Reader) -> Result<()> {
        self.location = try!(reader.expect_vec2());
        try!(reader.expect_start());
        try!(update_children(&mut self.children, reader));

        reader.skip_to_end()
    }
}

impl Grid {
    fn write(&self, writer: &mut Writer) -> Result<()> {
        try!(writer.write_start());
        try!(writer.write_value(&Value::Tag(GRID)));
        try!(writer.write_value(&Value::Box2(self.bounds)));
        try!(writer.write_value(&Value::Vec2(self.size)));
        try!(writer.write_value(&Value::Vec2(self.offset)));
        try!(writer.write_value(&Value::Vec3(self.colour)));
        try!(writer.write_end());
        Ok(())
    }

    fn update(&mut self, reader: &mut Reader) -> Result<()> {
        self.bounds = try!(reader.expect_box2());
        self.size = try!(reader.expect_vec2());
        self.offset = try!(reader.expect_vec2());
        self.colour = try!(reader.expect_vec3());

        reader.skip_to_end()
    }
}

impl ModelElement {
    fn write(&self, writer: &mut Writer) -> Result<()> {
        try!(writer.write_start());
        try!(writer.write_value(&Value::Tag(MODEL)));
        try!(writer.write_value(&Value::Vec2(self.location)));
        try!(writer.write_value(&Value::Double(self.scale)));

        try!(self.model.write(writer));

        try!(writer.write_end());
        Ok(())
    }

    fn update(&mut self, reader: &mut Reader) -> Result<()> {
        self.location = try!(reader.expect_vec2());
        self.scale = try!(reader.expect_double());

        if try!(reader.expect_start_or_end()) {
            self.model = try!(Model::read_started(reader));
        }

        reader.skip_to_end()
    }
}

impl Default for ModelElement {
    fn default() -> ModelElement {
        ModelElement {
            location: (0.0, 0.0),
            scale: 0.0,
            model: Model {
                paths: Vec::new()
            }
        }
    }
}

impl Text {
    fn write(&self, writer: &mut Writer) -> Result<()> {
        try!(writer.write_start());
        try!(writer.write_value(&Value::Tag(TEXT)));
        try!(writer.write_value(&Value::Vec2(self.location)));
        try!(writer.write_value(&Value::Double(self.size)));
        try!(writer.write_value(&Value::Vec3(self.colour)));
        try!(writer.write_value(&Value::String(self.value.clone().into_boxed_str())));
        try!(writer.write_end());
        Ok(())
    }

    fn update(&mut self, reader: &mut Reader) -> Result<()> {
        self.location = try!(reader.expect_vec2());
        self.size = try!(reader.expect_double());
        self.colour = try!(reader.expect_vec3());
        self.value = try!(reader.expect_string()).into_string();

        reader.skip_to_end()
    }
}

impl Event {
    fn write(self, writer: &mut Writer) -> Result<()> {
        let tag = match self {
            Event::Down => DOWN,
            Event::Up => UP,
            Event::Motion => MOTION,
            Event::Key => KEY,
            Event::Text => TEXT,
            Event::KeyboardFocusLost => KEYBOARD_FOCUS_LOST,
        };
        writer.write_value(&Value::Tag(tag))
    }

    fn from_tag(tag: Tag) -> Option<Event> {
        match tag {
            DOWN => Some(Event::Down),
            UP => Some(Event::Up),
            MOTION => Some(Event::Motion),
            KEY => Some(Event::Key),
            TEXT => Some(Event::Text),
            KEYBOARD_FOCUS_LOST => Some(Event::KeyboardFocusLost),
            _ => None
        }
    }
}


pub fn update_children(children: &mut Vec<Element>, reader: &mut Reader) -> Result<()> {
    let mut i = 0;

    while try!(reader.expect_start_or_end()) {
        let tag = try!(reader.expect_tag());

        if i < children.len() {
            let child = &mut children[i];
            match tag {
                WIDGET =>
                    if let &mut Element::Widget(ref mut widget) = child {
                        try!(widget.update(reader));
                    } else {
                        let mut widget: Widget = Default::default();
                        try!(widget.update(reader));
                        *child = Element::Widget(widget);
                    },
                GROUP =>
                    if let &mut Element::Group(ref mut group) = child {
                        try!(group.update(reader));
                    } else {
                        let mut group: Group = Default::default();
                        try!(group.update(reader));
                        *child = Element::Group(group);
                    },
                GRID =>
                    if let &mut Element::Grid(ref mut grid) = child {
                        try!(grid.update(reader));
                    } else {
                        let mut grid: Grid = Default::default();
                        try!(grid.update(reader));
                        *child = Element::Grid(grid);
                    },
                MODEL =>
                    if let &mut Element::Model(ref mut model) = child {
                        try!(model.update(reader));
                    } else {
                        let mut model: ModelElement = Default::default();
                        try!(model.update(reader));
                        *child = Element::Model(model);
                    },
                TEXT =>
                    if let &mut Element::Text(ref mut text) = child {
                        try!(text.update(reader));
                    } else {
                        let mut text: Text = Default::default();
                        try!(text.update(reader));
                        *child = Element::Text(text);
                    },
                _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown element type"))
            }
        } else {
            match tag {
                WIDGET => {
                    let mut widget: Widget = Default::default();
                    try!(widget.update(reader));
                    children.push(Element::Widget(widget));
                },
                GROUP => {
                    let mut group: Group = Default::default();
                    try!(group.update(reader));
                    children.push(Element::Group(group));
                },
                GRID => {
                    let mut grid: Grid = Default::default();
                    try!(grid.update(reader));
                    children.push(Element::Grid(grid));
                },
                MODEL => {
                    let mut model: ModelElement = Default::default();
                    try!(model.update(reader));
                    children.push(Element::Model(model));
                },
                TEXT => {
                    let mut text: Text = Default::default();
                    try!(text.update(reader));
                    children.push(Element::Text(text));
                },
                _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown element type"))
            }
        }

        i += 1;
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::model::{Model, Path, Point};
    use super::super::data::{BinaryWriter, TextWriter, BinaryReader, Reader};
    use std::io::{stdout, Cursor, copy};
    use std::fs::File;

    fn sample() -> Element {
        Element::Widget(Widget {
            location: (0.0, 0.0),
            size: (1000.0, 1000.0),
            fill_colour: (0.0, 0.0, 0.0, 1.0),
            border_colour: (0.0, 0.0, 0.0),
            border_width: 0,
            bindings: Vec::new(),
            children: vec![
                Element::Widget(Widget {
                    location: (0.0, 0.0),
                    size: (1000.0, 1000.0),
                    fill_colour: (0.1, 0.1, 0.1, 1.0),
                    border_colour: (0.0, 0.0, 0.0),
                    border_width: 0,
                    bindings: Vec::new(),
                    children: vec![
                        Element::Grid(Grid {
                            bounds: ((0.0, 0.0), (1000.0, 1000.0)),
                            size: (20.0, 20.0),
                            offset: (0.0, 0.0),
                            colour: (0.3, 0.3, 0.3)
                        }),
                        Element::Model(ModelElement {
                            location: (500.0, 500.0),
                            scale: 50.0,
                            model: Model {
                                paths: vec![
                                    Path {
                                        colour: (0.9, 0.3, 0.7),
                                        points: vec![
                                            Point {location: (0.0, 0.0), curve_bias: 0.0},
                                            Point {location: (0.0, 0.0), curve_bias: 0.0},
                                            Point {location: (0.0, 0.0), curve_bias: 0.0},
                                            Point {location: (0.0, 0.0), curve_bias: 0.0},
                                        ]
                                    }
                                ]
                            }
                        }),
                        Element::Group(Group {
                            location: (0.0, 0.0),
                            children: vec![
                                Element::Widget(Widget {
                                    location: (0.0, 0.0),
                                    size: (5.0, 5.0),
                                    fill_colour: (1.0, 1.0, 1.0, 1.0),
                                    border_colour: (0.0, 0.0, 0.0),
                                    border_width: 1,
                                    bindings: vec![
                                        (Event::Down, 1004),
                                        (Event::Up, 1014),
                                    ],
                                    children: Vec::new()
                                }),
                                Element::Widget(Widget {
                                    location: (0.0, 0.0),
                                    size: (5.0, 5.0),
                                    fill_colour: (1.0, 1.0, 1.0, 1.0),
                                    border_colour: (0.0, 0.0, 0.0),
                                    border_width: 1,
                                    bindings: vec![
                                        (Event::Down, 1005),
                                        (Event::Up, 1005),
                                    ],
                                    children: Vec::new()
                                }),
                                Element::Widget(Widget {
                                    location: (0.0, 0.0),
                                    size: (5.0, 5.0),
                                    fill_colour: (1.0, 1.0, 1.0, 1.0),
                                    border_colour: (0.0, 0.0, 0.0),
                                    border_width: 1,
                                    bindings: vec![
                                        (Event::Down, 1006),
                                        (Event::Up, 1006),
                                    ],
                                    children: Vec::new()
                                }),
                                Element::Widget(Widget {
                                    location: (0.0, 0.0),
                                    size: (5.0, 5.0),
                                    fill_colour: (1.0, 1.0, 1.0, 1.0),
                                    border_colour: (0.0, 0.0, 0.0),
                                    border_width: 1,
                                    bindings: vec![
                                        (Event::Down, 1007),
                                        (Event::Up, 1007),
                                    ],
                                    children: Vec::new()
                                }),
                            ]
                        })
                    ]
                }),
                Element::Widget(Widget {
                    location: (10.0, 960.0),
                    size: (80.0, 30.0),
                    fill_colour: (0.1, 0.1, 0.1, 1.0),
                    border_colour: (0.0, 0.0, 0.0),
                    border_width: 0,
                    bindings: Vec::new(),
                    children: vec![
                        Element::Widget(Widget {
                            location: (5.0, 5.0),
                            size: (20.0, 20.0),
                            fill_colour: (0.9, 0.0, 0.0, 1.0),
                            border_colour: (0.0, 0.0, 0.0),
                            border_width: 0,
                            bindings: vec![(Event::Down, 1001)],
                            children: vec![
                                Element::Text(Text {
                                    location: (0.0, 0.0),
                                    size: 12.0,
                                    colour: (1.0, 1.0, 1.0),
                                    value: "New".to_string()
                                }),
                            ]
                        }),
                        Element::Widget(Widget {
                            location: (35.0, 5.0),
                            size: (20.0, 20.0),
                            fill_colour: (0.0, 0.9, 0.0, 1.0),
                            border_colour: (0.0, 0.0, 0.0),
                            border_width: 0,
                            bindings: vec![(Event::Down, 1002)],
                            children: vec![
                                Element::Text(Text {
                                    location: (0.0, 0.0),
                                    size: 12.0,
                                    colour: (1.0, 1.0, 1.0),
                                    value: "Open".to_string()
                                }),
                            ]
                        }),
                        Element::Widget(Widget {
                            location: (65.0, 5.0),
                            size: (20.0, 20.0),
                            fill_colour: (0.0, 0.0, 0.9, 1.0),
                            border_colour: (0.0, 0.0, 0.0),
                            border_width: 0,
                            bindings: vec![(Event::Down, 1003)],
                            children: vec![
                                Element::Text(Text {
                                    location: (0.0, 0.0),
                                    size: 12.0,
                                    colour: (1.0, 1.0, 1.0),
                                    value: "Save".to_string()
                                }),
                            ]
                        }),
                    ]
                })
            ]
        })
    }

    #[test]
    fn write_and_read() {
        let element = sample();

        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        element.write(&mut writer).unwrap();

        let orig = writer.into_inner().into_inner();

        let mut reader = BinaryReader::new(Cursor::new(orig.clone()));
        let mut widget: Widget = Default::default();
        reader.expect_start().unwrap();
        reader.expect_tag().unwrap();
        widget.update(&mut reader).unwrap();

        let mut writer = BinaryWriter::new(Cursor::new(Vec::new()));
        Element::Widget(widget).write(&mut writer).unwrap();

        let copy = writer.into_inner().into_inner();

        assert_eq!(copy, orig);
    }
}
