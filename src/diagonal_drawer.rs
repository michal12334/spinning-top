use glium::glutin::surface::WindowSurface;
use glium::{
    uniform, Display, DrawParameters, IndexBuffer, PolygonMode, Program, Surface, VertexBuffer,
};
use nalgebra::Matrix4;

use crate::cube::Cube;
use crate::vertex::Vertex;

pub struct DiagonalDrawer {
    program: Program,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
}

impl DiagonalDrawer {
    pub fn new(display: &Display<WindowSurface>) -> Self {
        let vertex_shader_src = r#"
            #version 410 core

            in vec3 position;

            uniform mat4 perspective;
            uniform mat4 view;
            uniform mat4 model;

            void main() {
                gl_Position = perspective * view * model * vec4(position, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 410 core

            out vec4 color;
            
            void main() {
                color = vec4(0, 1, 0, 0.5);
            }
        "#;

        let program =
            Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

        Self {
            program,
            vertex_buffer: VertexBuffer::new(
                display,
                &[
                    Vertex::new([0.0, 0.0, 0.0]),
                    Vertex::new([0.0, 3.0f32.sqrt(), 0.0]),
                ],
            )
            .unwrap(),
            index_buffer: IndexBuffer::new(
                display,
                glium::index::PrimitiveType::LinesList,
                &[0u16, 1],
            )
            .unwrap(),
        }
    }

    pub fn draw(
        &self,
        target: &mut glium::Frame,
        perspective: &Matrix4<f32>,
        view: &Matrix4<f32>,
        cube: &Cube,
        drawing_parameters: &DrawParameters,
    ) {
        let mut drawing_parameters = drawing_parameters.clone();
        drawing_parameters.polygon_mode = PolygonMode::Line;

        target
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniform! {
                    perspective: perspective.data.0,
                    view: view.data.0,
                    model: cube.get_scale_matrix().data.0,
                },
                &drawing_parameters,
            )
            .unwrap();
    }
}
