use glium::{VertexBuffer, Program, Surface, Blend};
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use glium::draw_parameters::DrawParameters;
use data::Vec2;
use super::{Model, Point};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    param: [f32; 3]
}

implement_vertex!(Vertex, position, param);

struct PathBuffer {
    vertices: VertexBuffer<Vertex>,
    colour: [f32; 3]
}

pub struct ModelBuffers {
    paths: Vec<PathBuffer>
}

pub struct ModelRenderer<'a> {
    indices: NoIndices,
    params: DrawParameters<'a>,
    program: Program,
    width: f32,
    height: f32
}

impl<'a> ModelRenderer<'a> {
    pub fn new<F: Facade>(display: &F) -> ModelRenderer {
        ModelRenderer {
            indices: NoIndices(PrimitiveType::TrianglesList),
            params: DrawParameters {
                blend: Blend::alpha_blending(),
                .. Default::default()
            },
            program: Program::from_source(
                display,
                include_str!("model.vert"),
                include_str!("model.frag"),
                None)
                .unwrap(),
            width: 1.0,
            height: 1.0
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn draw<S: Surface>(&self, surface: &mut S, x: f32, y: f32, scale: f32, model: &ModelBuffers) {
        for path in &model.paths {
            let uniforms = uniform! {
                viewport_size: [self.width, self.height],
                translate: [x, y],
                scale: scale,
                colour: path.colour
            };

            surface.draw(
                &path.vertices,
                self.indices,
                &self.program,
                &uniforms,
                &self.params)
                .unwrap();
        }
    }
}

pub fn prepare_model<F: Facade>(display: &F, model: &Model) -> ModelBuffers {
    let paths = model
        .paths
        .iter()
        .map(|path| {
            let vertices = build_path_vertices(&path.points);

            PathBuffer {
                vertices: VertexBuffer::new(display, &vertices).unwrap(),
                colour: [path.colour.0 as f32, path.colour.1 as f32, path.colour.2 as f32]
            }
        })
        .collect();

    ModelBuffers {
        paths: paths
    }
}

struct VertexNode {
    next: usize,
    point: Point,
}

fn build_path_vertices(points: &Vec<Point>) -> Vec<Vertex> {
    let mut nodes = build_vertex_nodes(points);
    let first = if nodes[0].point.curve_bias > 0.0 { 1 } else { 0 };

    let mut output = Vec::new();

    build_curve_triangles(&mut nodes, first, &mut output);
    build_inner_triangles(&mut nodes, first, &mut output);

    output
}

fn build_vertex_nodes(points: &Vec<Point>) -> Vec<VertexNode> {
    let mut nodes = Vec::with_capacity(points.len() * 2);
    nodes.push(VertexNode {
        next: 0,
        point: points[points.len() - 1]
    });
    let mut last = 0;

    for (i, point) in points.iter().enumerate() {
        if point.curve_bias > 0.0 && nodes[last].point.curve_bias > 0.0 {
            let a = nodes[last].point.location;
            let b = point.location;
            let t = nodes[last].point.curve_bias;
            let x = a.0 + t * (b.0 - a.0);
            let y = a.1 + t * (b.1 - a.1);

            nodes.push(VertexNode {
                next: 0,
                point: Point {
                    location: (x, y),
                    curve_bias: 0.0
                }
            });
            nodes[last].next = last + 1;
            last += 1;
        }

        if i != points.len() - 1 {
            nodes.push(VertexNode {
                next: 0,
                point: *point
            });
            nodes[last].next = last + 1;
            last += 1;
        }
    }

    nodes
}

fn build_curve_triangles(nodes: &mut Vec<VertexNode>, first: usize, output: &mut Vec<Vertex>) {
    let mut is_first = true;
    let mut i = first;
    while is_first || i != first {
        if nodes[nodes[i].next].point.curve_bias > 0.0 {
            let p1 = nodes[i].point.location;
            let p2 = nodes[nodes[i].next].point.location;
            let p3 = nodes[nodes[nodes[i].next].next].point.location;

            let cross = vec2_cross(p1, p2, p3);
            let sign = if cross == 0.0 {
                nodes[i].next = nodes[nodes[i].next].next;
                continue;
            } else if cross > 0.0 {
                nodes[i].next = nodes[nodes[i].next].next;
                -1
            } else {
                1
            };

            output.push(Vertex {
                position: [p1.0 as f32, p1.1 as f32],
                param: [0.0, 0.0, sign as f32],
            });
            output.push(Vertex {
                position: [p2.0 as f32, p2.1 as f32],
                param: [0.5, 0.0, sign as f32],
            });
            output.push(Vertex {
                position: [p3.0 as f32, p3.1 as f32],
                param: [1.0, 1.0, sign as f32],
            });
        }

        i = nodes[i].next;
        is_first = false;
    }
}

fn build_inner_triangles(nodes: &mut Vec<VertexNode>, first: usize, output: &mut Vec<Vertex>) {
    let mut v1 = first;
    let mut last_success = v1;

    loop {
        let v3 = nodes[nodes[v1].next].next;

        if v1 == v3 {
            return;
        }

        let p1 = nodes[v1].point.location;
        let p2 = nodes[nodes[v1].next].point.location;
        let p3 = nodes[v3].point.location;
        let mut success = false;

        if vec2_cross(p1, p3, p2) < 0.0 {
            let mut empty = true;

            let mut v = nodes[v3].next;
            while v != v1 {
                if triangle_contains(p1, p2, p3, nodes[v].point.location) {
                    empty = false;
                    break;
                }

                v = nodes[v].next;
            }

            if empty {
                output.push(Vertex {
                    position: [p1.0 as f32, p1.1 as f32],
                    param: [0.0f32, 1.0f32, -1.0f32]
                });
                output.push(Vertex {
                    position: [p2.0 as f32, p2.1 as f32],
                    param: [0.0f32, 1.0f32, -1.0f32]
                });
                output.push(Vertex {
                    position: [p3.0 as f32, p3.1 as f32],
                    param: [0.0f32, 1.0f32, -1.0f32]
                });

                nodes[v1].next = v3;
                success = true;
                last_success = v1;
            }
        }

        if !success {
            v1 = nodes[v1].next;

            if v1 == last_success {
                return;
            }
        }
    }
}

fn vec2_cross(a: Vec2, b: Vec2, c: Vec2) -> f64 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

fn triangle_contains(t1: Vec2, t2: Vec2, t3: Vec2, p: Vec2) -> bool {
    vec2_cross(t1, p, t2) < 0.0 &&
        vec2_cross(t2, p, t3) < 0.0 &&
        vec2_cross(t3, p, t1) < 0.0
}
