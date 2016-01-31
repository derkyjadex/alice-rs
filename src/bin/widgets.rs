#[macro_use]
extern crate glium;
extern crate alice;
extern crate rand;

use std::fs::File;
use std::io::Cursor;
use glium::{DisplayBuild, Surface};
use glium::glutin::{Event, ElementState, VirtualKeyCode};
use alice::model::{Model};
use alice::widget::rendering::ElementRenderer;
use alice::widget::{Element, Widget, Group, Grid, ModelElement, Text};

fn main() {
    let display = glium::glutin::WindowBuilder::new()
        .with_title(format!("Alice"))
        .with_dimensions(1024, 768)
        .with_vsync()
        .build_glium()
        .unwrap();
    let window = display.get_window().unwrap();

    let model = if let Some(path) = std::env::args().nth(1) {
        let file = File::open(path).unwrap();
        let mut reader = alice::data::BinaryReader::new(file);
        Model::read(&mut reader).unwrap()
    } else {
        let bytes = include_bytes!("cat.model");
        let file: Cursor<&[u8]> = Cursor::new(bytes);
        let mut reader = alice::data::BinaryReader::new(file);
        Model::read(&mut reader).unwrap()
    };

    let mut renderer = ElementRenderer::new(&display);

    let root = Element::Widget(Widget {
        location: (200.0, 100.0),
        size: (500.0, 300.0),
        fill_colour: (0.9, 0.0, 0.0, 1.0),

        children: vec![
            Element::Widget(Widget {
                location: (10.0, 20.0),
                size: (100.0, 100.0),
                fill_colour: (0.7, 0.3, 0.8, 1.0),
                .. Default::default()
            }),
            Element::Widget(Widget {
                location: (120.0, 20.0),
                size: (100.0, 100.0),
                fill_colour: (0.3, 0.8, 0.7, 1.0),
                border_colour: (0.9, 0.3, 0.9),
                border_width: 10,
                .. Default::default()
            }),
            Element::Group(Group {
                location: (300.0, 200.0),
                children: vec![
                    Element::Widget(Widget {
                        location: (0.0, 0.0),
                        size: (20.0, 20.0),
                        fill_colour: (0.8, 0.7, 0.3, 1.0),
                        .. Default::default()
                    }),
                    Element::Widget(Widget {
                        location: (25.0, 0.0),
                        size: (20.0, 20.0),
                        fill_colour: (0.3, 0.3, 0.7, 1.0),
                        .. Default::default()
                    }),
                ]
            }),
            Element::Grid(Grid {
                bounds: ((100.0, 100.0), (400.0, 300.0)),
                size: (50.0, 20.0),
                offset: (0.0, 0.0),
                colour: (0.9, 0.9, 0.9),
                .. Default::default()
            }),
            Element::Model(ModelElement {
                location: (250.0, 150.0),
                scale: 0.2,
                model: model
            }),
            Element::Text(Text {
                location: (2.0, 2.0),
                size: 12.0,
                colour: (1.0, 1.0, 1.0),
                value: "Hello World!".to_string()
            })
        ],
        .. Default::default()
    });

    loop {
        let mut target = display.draw();
        target.clear_color(0.02, 0.02, 0.02, 1.0);

        let (w, h) = window.get_inner_size_points().unwrap();
        renderer.set_size(w as f32, h as f32);

        renderer.draw(&mut target, &root);

        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Escape)) => return,
                _ => ()
            }
        }
    }
}
