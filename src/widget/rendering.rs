use glium::{VertexBuffer, Program, Surface, Blend};
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use glium::draw_parameters::DrawParameters;
use super::super::data::{Vec2};
use super::super::model::rendering::{ModelRenderer, prepare_model};
use super::{Element, Widget, Group, Grid, ModelElement, Text};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2]
}

implement_vertex!(Vertex, position);

pub struct ElementRenderer<'a, F: 'a> {
    display: &'a F,
    params: DrawParameters<'a>,
    vertices: VertexBuffer<Vertex>,
    indices: NoIndices,
    widget_program: Program,
    grid_program: Program,
    model_renderer: ModelRenderer<'a>,
    width: f32,
    height: f32
}

impl<'a, F: Facade> ElementRenderer<'a, F> {
    pub fn new(display: &F) -> ElementRenderer<F> {
        let vertices = VertexBuffer::new(
            display,
            &[
                Vertex { position: [0.0, 0.0] },
                Vertex { position: [1.0, 0.0] },
                Vertex { position: [1.0, 1.0] },
                Vertex { position: [0.0, 1.0] },
            ])
            .unwrap();

        ElementRenderer {
            display: display,
            params: DrawParameters {
                blend: Blend::alpha_blending(),
                .. Default::default()
            },
            vertices: vertices,
            indices: NoIndices(PrimitiveType::TriangleFan),
            widget_program: Program::from_source(
                display,
                include_str!("widget.vert"),
                include_str!("widget.frag"),
                None)
                .unwrap(),
            grid_program: Program::from_source(
                display,
                include_str!("grid.vert"),
                include_str!("grid.frag"),
                None)
                .unwrap(),
            model_renderer: ModelRenderer::new(display),
            width: 1.0,
            height: 1.0
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.model_renderer.set_size(width, height);
    }

    pub fn draw<S: Surface>(&self, surface: &mut S, element: &Element) {
        self.draw_element(surface, element, (0.0, 0.0));
    }

    pub fn draw_root<S: Surface>(&self, surface: &mut S, widget: &Widget) {
        self.draw_widget(surface, widget, (0.0, 0.0));
    }

    fn draw_element<S: Surface>(&self, surface: &mut S, element: &Element, origin: Vec2) {
        match element {
            &Element::Widget(ref widget) => self.draw_widget(surface, widget, origin),
            &Element::Group(ref group) => self.draw_group(surface, group, origin),
            &Element::Grid(ref grid) => self.draw_grid(surface, grid, origin),
            &Element::Model(ref model) => self.draw_model(surface, model, origin),
            &Element::Text(ref text) => self.draw_text(surface, text, origin),
        }
    }

    fn draw_widget<S: Surface>(&self, surface: &mut S, widget: &Widget, origin: Vec2) {
        let location = (origin.0 + widget.location.0, origin.1 + widget.location.1);

        let uniforms = uniform! {
            viewport_size: [self.width, self.height],
            location: [location.0 as f32, location.1 as f32],
            size: [widget.size.0 as f32, widget.size.1 as f32],
            fill_colour: [
                widget.fill_colour.0 as f32,
                widget.fill_colour.1 as f32,
                widget.fill_colour.2 as f32,
                widget.fill_colour.3 as f32],
            border_colour: [
                widget.border_colour.0 as f32,
                widget.border_colour.1 as f32,
                widget.border_colour.2 as f32],
            border_width: widget.border_width as f32
        };

        surface.draw(
            &self.vertices,
            self.indices,
            &self.widget_program,
            &uniforms,
            &self.params)
            .unwrap();

        for ref child in &widget.children {
            self.draw_element(surface, child, location);
        }
    }

    fn draw_group<S: Surface>(&self, surface: &mut S, group: &Group, origin: Vec2) {
        let location = (origin.0 + group.location.0, origin.1 + group.location.1);

        for ref child in &group.children {
            self.draw_element(surface, child, location);
        }
    }

    fn draw_grid<S: Surface>(&self, surface: &mut S, grid: &Grid, origin: Vec2) {
        let location = (origin.0 + (grid.bounds.0).0, origin.1 + (grid.bounds.0).1);
        let size = ((grid.bounds.1).0 - (grid.bounds.0).0, (grid.bounds.1).1 - (grid.bounds.0).1);

        let uniforms = uniform! {
            viewport_size: [self.width, self.height],
            location: [location.0 as f32, location.1 as f32],
            size: [size.0 as f32, size.1 as f32],
            grid_size: [grid.size.0 as f32, grid.size.1 as f32],
            grid_offset: [grid.offset.0 as f32, grid.size.1 as f32],
            grid_colour: [
                grid.colour.0 as f32,
                grid.colour.1 as f32,
                grid.colour.2 as f32],
        };

        surface.draw(
            &self.vertices,
            self.indices,
            &self.grid_program,
            &uniforms,
            &self.params)
            .unwrap();
    }

    fn draw_model<S: Surface>(&self, surface: &mut S, model: &ModelElement, origin: Vec2) {
        let location = (origin.0 + model.location.0, origin.1 + model.location.1);

        let prepared = prepare_model(self.display, &model.model);
        self.model_renderer.draw(
            surface,
            location.0 as f32,
            location.1 as f32,
            model.scale as f32,
            &prepared);
    }

    fn draw_text<S: Surface>(&self, _: &mut S, _: &Text, _: Vec2) {
    }
}
