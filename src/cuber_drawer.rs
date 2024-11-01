use glium::glutin::surface::WindowSurface;
use glium::{uniform, Display, DrawParameters, IndexBuffer, Program, Surface, VertexBuffer};
use nalgebra::Matrix4;

use crate::cube::Cube;
use crate::vertex::Vertex;

pub struct CubeDrawer {
    program: Program,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
}

impl CubeDrawer {
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
                color = vec4(1, 1, 1, 0.5);
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
                    Vertex::new([1.0, 0.0, 0.0]),
                    Vertex::new([0.0, 1.0, 0.0]),
                    Vertex::new([1.0, 1.0, 0.0]),
                    Vertex::new([0.0, 0.0, 1.0]),
                    Vertex::new([1.0, 0.0, 1.0]),
                    Vertex::new([0.0, 1.0, 1.0]),
                    Vertex::new([1.0, 1.0, 1.0]),
                ],
            )
            .unwrap(),
            index_buffer: IndexBuffer::new(
                display,
                glium::index::PrimitiveType::TrianglesList,
                &[
                    0u16, 2, 1, 1, 2, 3, // front
                    1, 3, 5, 3, 7, 5, //right
                    0, 4, 2, 2, 4, 6, // left
                    4, 5, 6, 5, 7, 6, // back
                    2, 6, 3, 3, 6, 7, // top
                    0, 1, 4, 1, 5, 4, // bottom
                ],
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
        target
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniform! {
                    perspective: perspective.data.0,
                    view: view.data.0,
                    model: cube.get_model_matrix().data.0,
                },
                &drawing_parameters,
            )
            .unwrap();
    }
}
