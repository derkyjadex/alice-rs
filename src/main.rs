#[macro_use]
extern crate glium;
extern crate alice;
extern crate rand;

use std::fs::File;
use std::io::Cursor;
use glium::{DisplayBuild, Surface};
use glium::glutin::{Event, ElementState, VirtualKeyCode, MouseScrollDelta};
use alice::model::rendering::{ModelRenderer, prepare_model};
use alice::model::io::read_model;
use alice::model::{Model, Path, Point};
use alice::data::{Vec2, Vec3};
use rand::{thread_rng, Rng};

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
        read_model(file).unwrap()
    } else {
        let bytes = include_bytes!("cat.model");
        let file: Cursor<&[u8]> = Cursor::new(bytes);
        read_model(file).unwrap()
    };

    let mut wobble = WobbleModel::new(&model);

    let mut x = 512.0;
    let mut y = 384.0;
    let mut scale = 1.0;

    loop {
        let mut target = display.draw();
        target.clear_color(0.02, 0.02, 0.02, 1.0);

        let (w, h) = window.get_inner_size_points().unwrap();
        renderer.set_size(w as f32, h as f32);

        wobble.tick();
        let model = prepare_model(&display, &wobble.model());
        renderer.draw(&mut target, x, y, scale, &model);

        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Escape)) => return,
                Event::ReceivedCharacter(' ') => wobble.shuffle(),
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

struct WobbleModel {
    paths: Vec<WobblePath>
}

struct WobblePath {
    colour: Vec3,
    points: Vec<WobblePoint>
}

struct WobblePoint {
    location: Vec2,
    curve_bias: f64,
    offset: Vec2,
    velocity: Vec2
}

const N: f64 = 30.0;
const K: f64 = 0.8;
const D: f64 = 0.9;

impl WobbleModel {
    pub fn new(model: &Model) -> WobbleModel {
        WobbleModel {
            paths: model.paths
                .iter()
                .map(|path| WobblePath {
                    colour: path.colour,
                    points: path.points
                        .iter()
                        .map(|point| WobblePoint {
                            location: point.location,
                            curve_bias: point.curve_bias,
                            offset: (0.0, 0.0),
                            velocity: (0.0, 0.0)
                        })
                        .collect()
                })
                .collect()
        }
    }

    pub fn model(&self) -> Model {
        Model {
            paths: self.paths
                .iter()
                .map(|path| Path {
                    colour: path.colour,
                    points: path.points
                        .iter()
                        .map(|point| {
                            let x = point.location.0 + point.offset.0;
                            let y = point.location.1 + point.offset.1;
                            Point {
                                location: (x, y),
                                curve_bias: point.curve_bias
                            }
                        })
                        .collect()
                })
                .collect()
        }
    }

    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        for path in self.paths.iter_mut() {
            for point in path.points.iter_mut() {
                point.offset.0 += rng.gen_range(-N, N);
                point.offset.1 += rng.gen_range(-N, N);
            }
        }
    }

    pub fn tick(&mut self) {
        for path in self.paths.iter_mut() {
            for point in path.points.iter_mut() {
                let (mut ox, mut oy) = point.offset;
                let (mut vx, mut vy) = point.velocity;

                vx += -K * ox;
                vy += -K * oy;
                vx *= D;
                vy *= D;

                ox += vx;
                oy += vy;

                point.offset = (ox, oy);
                point.velocity = (vx, vy);
            }
        }
    }
}
