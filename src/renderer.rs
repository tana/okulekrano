use std::sync::Arc;

use glium::{
    glutin::surface::WindowSurface,
    implement_vertex,
    index::{NoIndices, PrimitiveType},
    uniform, Display, DrawParameters, Frame, Program, Rect, Surface, Texture2d, VertexBuffer,
};
use na::{Matrix4, Scale3, Translation3};

use crate::{
    capturer::{fake::FakeCapturer, wayland::WaylandCapturer, Capturer},
    config::Config,
    glasses::GlassesController,
};

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
    capturer: Box<dyn Capturer>,
    screen_transform: Matrix4<f32>, // Position and rotation of the virtual screen in world coordinates
}

impl Renderer {
    pub fn new(display: Arc<Display<WindowSurface>>, config: &Config) -> Self {
        let vertex_buffer = VertexBuffer::new(display.as_ref(), QUAD_VERTICES).unwrap();
        let index_buffer = NoIndices(PrimitiveType::TrianglesList);
        let program = Program::from_source(
            display.as_ref(),
            include_str!("shader.vert"),
            include_str!("shader.frag"),
            None,
        )
        .unwrap();

        let capturer: Box<dyn Capturer> =
            if let Some("_fake_desktop") = config.capture.output_name.as_deref() {
                Box::new(FakeCapturer::new(display.as_ref()))
            } else {
                Box::new(WaylandCapturer::new(
                    Arc::clone(&display),
                    config.capture.output_name.as_deref(),
                ))
            };

        let resolution = capturer.resolution();
        let aspect = resolution.0 as f32 / resolution.1 as f32;
        let screen_transform = Translation3::new(0.0, 0.0, -config.virtual_screen.distance)
            .to_homogeneous()
            * Scale3::new(
                config.virtual_screen.height * aspect,
                config.virtual_screen.height,
                1.0,
            )
            .to_homogeneous();

        Self {
            display,
            vertex_buffer,
            index_buffer,
            program,
            capturer,
            screen_transform,
        }
    }

    pub fn render(&mut self, glasses: &GlassesController) {
        let texture = self.capturer.capture();

        let mut frame = self.display.draw();

        frame.clear_color(0.0, 0.0, 0.0, 1.0);

        let (width, height) = frame.get_dimensions();
        let aspect = width as f32 / 2.0 / height as f32;
        self.render_view(
            &mut frame,
            &texture,
            &glasses.camera_mat(ar_drivers::Side::Left, aspect),
            -1.0,
            0.0,
        );
        self.render_view(
            &mut frame,
            &texture,
            &glasses.camera_mat(ar_drivers::Side::Right, aspect),
            0.0,
            1.0,
        );

        frame.finish().unwrap();
    }

    // camera_matrix: projection_matrix*world_to_camera
    fn render_view(
        &mut self,
        frame: &mut Frame,
        texture: &Texture2d,
        camera_matrix: &Matrix4<f32>,
        viewport_left_ndc: f32,
        viewport_right_ndc: f32,
    ) {
        let (width, height) = frame.get_dimensions();
        let left = remap(viewport_left_ndc, -1.0, 1.0, 0.0, width as f32).round() as u32;
        let right = remap(viewport_right_ndc, -1.0, 1.0, 0.0, width as f32).round() as u32;
        let viewport = Rect {
            left,
            bottom: 0,
            width: right - left,
            height,
        };
        let parameters = DrawParameters {
            viewport: Some(viewport),
            ..Default::default()
        };

        let uniforms = uniform! {
            tex: texture,
            transform: Into::<[[f32; 4]; 4]>::into(camera_matrix * self.screen_transform),
        };

        frame
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniforms,
                &parameters,
            )
            .unwrap();
    }
}

fn remap(x: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    to_min + (x - from_min) * (to_max - to_min) / (from_max - from_min)
}
