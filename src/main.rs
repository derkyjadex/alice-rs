#[macro_use]
extern crate glium;
extern crate alice;

use std::fs::File;
use std::io::Cursor;
use glium::{DisplayBuild, Surface};
use glium::glutin::{Event, ElementState, VirtualKeyCode, MouseScrollDelta};
use alice::model::rendering::{ModelRenderer, prepare_model};
use alice::model::io::read_model;

fn main() {
    let display = glium::glutin::WindowBuilder::new()
        .with_title(format!("Alice"))
        .with_dimensions(1024, 768)
        .with_vsync()
        .build_glium()
        .unwrap();
    let window = display.get_window().unwrap();

    let mut renderer = ModelRenderer::new(&display);

    let model = if let Some(path) = std::env::args().nth(1) {
        let file = File::open(path).unwrap();
        let model = read_model(file).unwrap();
        prepare_model(&display, &model)
    } else {
        let bytes = include_bytes!("cat.model");
        let file: Cursor<&[u8]> = Cursor::new(bytes);
        let model = read_model(file).unwrap();
        prepare_model(&display, &model)
    };

    let mut x = 512.0;
    let mut y = 384.0;
    let mut scale = 1.0;

    loop {
        let mut target = display.draw();
        target.clear_color(0.02, 0.02, 0.02, 1.0);

        let (w, h) = window.get_inner_size_points().unwrap();
        renderer.set_size(w as f32, h as f32);

        renderer.draw(&mut target, x, y, scale, &model);

        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Escape)) => return,
                Event::ReceivedCharacter('+') => scale *= 1.1,
                Event::ReceivedCharacter('=') => scale *= 1.1,
                Event::ReceivedCharacter('-') => scale /= 1.1,
                Event::ReceivedCharacter('0') => {
                    x = w as f32 / 2.0;
                    y = h as f32 / 2.0;
                    scale = 1.0;
                }
                Event::MouseWheel(MouseScrollDelta::PixelDelta(dx, dy)) => {
                    x -= dx;
                    y += dy;
                },
                _ => ()
            }
        }
    }
}
