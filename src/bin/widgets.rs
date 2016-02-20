#[macro_use]
extern crate glium;
extern crate alice;
extern crate rand;

use std::net::{TcpListener};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::io::Write;
use std::io;
use std::net::TcpStream;

use glium::{DisplayBuild, Surface};
use glium::glutin::{Event, ElementState, VirtualKeyCode, MouseButton};
use alice::data::{Reader, Vec2};
use alice::widget::rendering::ElementRenderer;
use alice::widget::{Element, Widget, Group, EventType, Binding};

fn main() {
    let display = glium::glutin::WindowBuilder::new()
        .with_title(format!("Alice"))
        .with_dimensions(800, 600)
        .with_vsync()
        .build_glium()
        .unwrap();
    let window = display.get_window().unwrap();

    let mut renderer = ElementRenderer::new(&display);

    let root = Arc::new(Mutex::new(Default::default()));
    let mut mouse_pos = (0.0, 0.0);

    let (sender, receiver) = channel();
    start_updater(&root, receiver);

    loop {
        let mut target = display.draw();
        target.clear_color(0.02, 0.02, 0.02, 1.0);

        let (w, h) = window.get_inner_size_points().unwrap();
        renderer.set_size(w as f32, h as f32);

        {
            let root = root.lock().unwrap();
            renderer.draw_root(&mut target, &root);

            target.finish().unwrap();

            let mut events = Vec::new();

            for ev in display.poll_events() {
                match ev {
                    Event::Closed => return,
                    Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Escape)) => return,
                    Event::MouseMoved((x, y)) => {
                        let f = window.hidpi_factor() as f64;
                        mouse_pos = (x as f64 / f, h as f64 - y as f64 / f);
                    },
                    Event::MouseInput(ElementState::Pressed, MouseButton::Left) => {
                        if let (true, Some(binding)) = down_event(&root, mouse_pos) {
                            events.push((binding, alice::widget::Event::Down));
                        }
                    }
                    _ => ()
                }
            }

            if events.len() > 0 {
                sender.send(events).unwrap();
            }
        }
    }
}

fn start_updater(root: &Arc<Mutex<Widget>>, receiver: Receiver<Vec<(Binding, alice::widget::Event)>>) {
    let (writer_sender, writer_receiver) = channel();

    let root = root.clone();
    std::thread::spawn(move || {
        updater(root, writer_sender);
    });
    std::thread::spawn(move || {
        event_sender(receiver, writer_receiver);
    });
}

fn updater(root: Arc<Mutex<Widget>>, writer_sender: Sender<TcpStream>) {
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap();
    for stream in listener.incoming() {
        println!("Starting new connection");
        let stream = stream.unwrap();

        writer_sender.send(stream.try_clone().unwrap()).unwrap();

        let mut reader = alice::data::TextReader::new(stream);

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

fn event_sender(event_receiver: Receiver<Vec<(Binding, alice::widget::Event)>>, writer_receiver: Receiver<TcpStream>) {
    let mut output = None;

    loop {
        match event_receiver.recv() {
            Ok(events) => {
                match writer_receiver.try_recv() {
                    Ok(new_output) => output = Some(new_output),
                    Err(TryRecvError::Empty) => (),
                    Err(TryRecvError::Disconnected) => return,
                }

                let result = if let Some(ref mut output) = output {
                    try_write_events(output, events)
                } else {
                    Ok(())
                };
                if !result.is_ok() {
                    output = None;
                }
            },
            Err(_) => return
        }
    }
}

fn try_write_events(output: &mut TcpStream, events: Vec<(Binding, alice::widget::Event)>) -> io::Result<()> {
    try!(write!(output, "("));
    for (binding, event) in events {
        try!(write!(output, "\n ({} {:?})", binding, event));
    }
    try!(write!(output, ")\n"));

    Ok(())
}

fn down_event(widget: &Widget, pos: Vec2) -> (bool, Option<Binding>) {
    if !widget.is_in_bounds(pos) {
        return (false, None);
    }

    let pos = (pos.0 - widget.location.0, pos.1 - widget.location.1);

    for child in &widget.children {
        match child {
            &Element::Widget(ref child_widget) => {
                match down_event(child_widget, pos) {
                    (true, Some(binding)) => return (true, Some(binding)),
                    (true, None) => break,
                    _ => ()
                }
            },
            &Element::Group(ref group) => {
                match down_event_group(group, pos) {
                    (true, Some(binding)) => return (true, Some(binding)),
                    (true, None) => break,
                    _ => ()
                }
            },
            _ => ()
        }
    }

    (true, widget.find_binding(EventType::Down))
}

fn down_event_group(group: &Group, pos: Vec2) -> (bool, Option<Binding>) {
    let pos = (pos.0 - group.location.0, pos.1 - group.location.1);

    for child in &group.children {
        match child {
            &Element::Widget(ref widget) => {
                match down_event(widget, pos) {
                    (true, binding) =>
                        return (true, binding),
                    _ => ()
                }
            },
            &Element::Group(ref group) => {
                match down_event_group(group, pos) {
                    (true, binding) =>
                        return (true, binding),
                    _ => ()
                }
            },
            _ => ()
        }
    }

    (false, None)
}
