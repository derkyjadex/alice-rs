#[macro_use]
extern crate glium;
extern crate alice;
extern crate rand;

use std::fs::File;
use std::io::Cursor;
use glium::{DisplayBuild, Surface};
use glium::glutin::{Event, ElementState, VirtualKeyCode/*, MouseScrollDelta, MouseButton*/};
//use alice::model::rendering::{ModelRenderer, prepare_model};
use alice::model::{Model/*, Path, Point*/};
use alice::widget::rendering::ElementRenderer;
use alice::widget::{Element, Widget, Group, Grid, ModelElement, Text};
//use alice::data::{Vec2, Vec3};
//use rand::{thread_rng, Rng};

fn main() {
    let display = glium::glutin::WindowBuilder::new()
        .with_title(format!("Alice"))
        .with_dimensions(1024, 768)
        .with_vsync()
        .build_glium()
        .unwrap();
    let window = display.get_window().unwrap();

    //let mut renderer = ModelRenderer::new(&display);

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

    //let mut wobble = WobbleModel::new(&model, 0.5, 0.95);

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

    /*let mut x = 512.0;
    let mut y = 384.0;
    let mut scale = 1.0;
    let mut mouse_pos = (0, 0);*/

    loop {
        let mut target = display.draw();
        target.clear_color(0.02, 0.02, 0.02, 1.0);

        let (w, h) = window.get_inner_size_points().unwrap();
        renderer.set_size(w as f32, h as f32);

        renderer.draw(&mut target, &root);

        /*wobble.tick();
        let model = prepare_model(&display, &wobble.model());
        renderer.draw(&mut target, x, y, scale, &model);*/


        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                Event::Closed => return,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Escape)) => return,
                /*Event::ReceivedCharacter(' ') => wobble.shuffle(10.0 / scale as f64),
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
                Event::MouseMoved(p) => mouse_pos = p,
                Event::MouseInput(ElementState::Pressed, MouseButton::Left) => {
                    let f = window.hidpi_factor() as f64;
                    let (mx, my) = (mouse_pos.0 as f64 / f, h as f64 - mouse_pos.1 as f64 / f);
                    let (mx, my) = (mx - x as f64, my - y as f64);
                    let (mx, my) = (mx / scale as f64, my / scale as f64);
                    wobble.push((mx, my), 0.3, 100.0 / scale as f64)
                },*/
                _ => ()
            }
        }
    }
}

/*struct WobbleModel {
    spring: f64,
    damping: f64,
    paths: Vec<WobblePath>,
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

fn gauss(c: f64, x: f64) -> f64 {
    (-x * x / (2.0 * c * c)).exp()
}

impl WobbleModel {
    pub fn new(model: &Model, spring: f64, damping: f64) -> WobbleModel {
        WobbleModel {
            spring: spring,
            damping: damping,
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

    pub fn shuffle(&mut self, v: f64) {
        let mut rng = rand::thread_rng();
        for path in self.paths.iter_mut() {
            for point in path.points.iter_mut() {
                point.velocity.0 += rng.gen_range(-v, v);
                point.velocity.1 += rng.gen_range(-v, v);
            }
        }
    }

    pub fn push(&mut self, (x, y): Vec2, s: f64, w: f64) {
        for path in self.paths.iter_mut() {
            for point in path.points.iter_mut() {
                let (px, py) = point.location;
                let (dx, dy) = (px - x, py - y);
                let d = (dx * dx + dy * dy).sqrt();
                let d = s * gauss(w, d);

                point.velocity.0 += d * dx;
                point.velocity.1 += d * dy;
            }
        }
    }

    pub fn tick(&mut self) {
        for path in self.paths.iter_mut() {
            for point in path.points.iter_mut() {
                let (mut ox, mut oy) = point.offset;
                let (mut vx, mut vy) = point.velocity;

                vx += -self.spring * ox;
                vy += -self.spring * oy;
                vx *= self.damping;
                vy *= self.damping;

                ox += vx;
                oy += vy;

                point.offset = (ox, oy);
                point.velocity = (vx, vy);
            }
        }
    }
}
*/
