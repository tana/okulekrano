use std::sync::Arc;

use glium::{
    glutin::surface::WindowSurface,
    implement_vertex,
    index::{NoIndices, PrimitiveType},
    uniform, Display, Program, Surface, VertexBuffer,
};
use na::Matrix4;

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

pub struct Renderer {
    display: Arc<Display<WindowSurface>>,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: NoIndices,
    program: Program,
    capturer: Capturer,
    transform: Matrix4<f32>,
}

impl Renderer {
    pub fn new(display: Arc<Display<WindowSurface>>, output_to_capture: Option<&str>) -> Self {
        let vertex_buffer = VertexBuffer::new(display.as_ref(), QUAD_VERTICES).unwrap();
        let index_buffer = NoIndices(PrimitiveType::TrianglesList);
        let program = Program::from_source(
            display.as_ref(),
            include_str!("shader.vert"),
            include_str!("shader.frag"),
            None,
        )
        .unwrap();

        let capturer = Capturer::new(Arc::clone(&display), output_to_capture);

        Self {
            display,
            vertex_buffer,
            index_buffer,
            program,
            capturer,
            transform: Matrix4::identity(),
        }
    }

    pub fn render(&mut self) {
        let texture = self.capturer.get_current_texture();

        let mut frame = self.display.draw();

        frame.clear_color(0.0, 0.0, 0.0, 1.0);

        let uniforms = uniform! {
            tex: texture.as_ref(),
            transform: Into::<[[f32; 4]; 4]>::into(self.transform),
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

    pub fn set_transform(&mut self, transform: &Matrix4<f32>) {
        self.transform = transform.clone();
    }
}
