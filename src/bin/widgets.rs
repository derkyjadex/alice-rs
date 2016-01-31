#[macro_use]
extern crate glium;
extern crate alice;
extern crate rand;

use std::net::{TcpListener};
use std::sync::{Arc, Mutex};

use glium::{DisplayBuild, Surface};
use glium::glutin::{Event, ElementState, VirtualKeyCode};
use alice::data::Reader;
use alice::widget::rendering::ElementRenderer;
use alice::widget::{Widget};

fn main() {
    let display = glium::glutin::WindowBuilder::new()
        .with_title(format!("Alice"))
        .with_dimensions(1024, 768)
        .with_vsync()
        .build_glium()
        .unwrap();
    let window = display.get_window().unwrap();

    let mut renderer = ElementRenderer::new(&display);

    let root = Arc::new(Mutex::new(Default::default()));

    start_updater(&root);

    loop {
        let mut target = display.draw();
        target.clear_color(0.02, 0.02, 0.02, 1.0);

        let (w, h) = window.get_inner_size_points().unwrap();
        renderer.set_size(w as f32, h as f32);

        {
            let root = root.lock().unwrap();
            renderer.draw_root(&mut target, &root);
        }

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

fn start_updater(root: &Arc<Mutex<Widget>>) {
    let root = root.clone();
    std::thread::spawn(move || {
        updater(root);
    });
}

fn updater(root: Arc<Mutex<Widget>>) {
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap();
    for stream in listener.incoming() {
        let mut reader = alice::data::TextReader::new(stream.unwrap());

        loop {
            let result = reader.expect_start();
            if !result.is_ok() {
                break;
            }
            let result = reader.expect_tag();
            if !result.is_ok() {
                break;
            }

            let mut root = root.lock().unwrap();
            let result = root.update(&mut reader);
            if !result.is_ok() {
                println!("{:?}", result);
                *root = Default::default();
                break;
            } else {
                println!("Update complete!");
            }
        }
    }
}
