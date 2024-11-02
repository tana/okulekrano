use std::{os::fd::AsFd, sync::Arc};

use glium::{
    glutin::surface::WindowSurface,
    implement_vertex,
    index::{NoIndices, PrimitiveType},
    uniform, Display, Program, Surface, VertexBuffer,
};
use khronos_egl as egl;

use crate::capturer::Capturer;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

const QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, 0.5, 0.0],
        tex_coords: [0.0, 0.0],
    }, // top left
    Vertex {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0, 1.0],
    }, // bottom left
    Vertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [1.0, 1.0],
    }, // bottom right
    Vertex {
        position: [-0.5, 0.5, 0.0],
        tex_coords: [0.0, 0.0],
    }, // top left
    Vertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [1.0, 1.0],
    }, // bottom right
    Vertex {
        position: [0.5, 0.5, 0.0],
        tex_coords: [1.0, 0.0],
    }, // top right
];

pub struct Renderer<T: AsFd> {
    display: Arc<Display<WindowSurface>>,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: NoIndices,
    program: Program,
    capturer: Capturer<T>,
}

impl<T: AsFd> Renderer<T> {
    pub fn new(display: Arc<Display<WindowSurface>>, gbm: gbm::Device<T>) -> Self {
        let vertex_buffer = VertexBuffer::new(display.as_ref(), QUAD_VERTICES).unwrap();
        let index_buffer = NoIndices(PrimitiveType::TrianglesList);
        let program = Program::from_source(
            display.as_ref(),
            include_str!("shader.vert"),
            include_str!("shader.frag"),
            None,
        )
        .unwrap();

        let egl = Arc::new(egl::Instance::new(khronos_egl::Static));

        let capturer = Capturer::new(
            Arc::clone(&display),
            gbm,
            Arc::clone(&egl),
        );

        Self {
            display,
            vertex_buffer,
            index_buffer,
            program,
            capturer,
        }
    }

    pub fn render(&mut self) {
        let mut frame = self.display.draw();

        frame.clear_color(0.0, 0.0, 1.0, 1.0);

        let texture = self.capturer.get_current_texture();

        let uniforms = uniform! {
            tex: texture.as_ref(),
        };

        frame
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();

        frame.finish().unwrap();
    }
}
