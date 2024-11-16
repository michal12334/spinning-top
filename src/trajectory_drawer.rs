use glium::glutin::surface::WindowSurface;
use glium::index::{NoIndices, PrimitiveType};
use glium::{uniform, Display, DrawParameters, Program, Surface};
use nalgebra::Matrix4;

use crate::trajectory::Trajectory;

pub struct TrajectoryDrawer {
    program: Program,
}

impl TrajectoryDrawer {
    pub fn new(display: &Display<WindowSurface>) -> Self {
        let vertex_shader_src = r#"
            #version 410 core

            in vec3 position;

            uniform mat4 perspective;
            uniform mat4 view;

            void main() {
                gl_Position = perspective * view * vec4(position, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 410 core

            out vec4 color;
            
            void main() {
                color = vec4(1, 1, 1, 1);
            }
        "#;

        let program =
            Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

        Self { program }
    }

    pub fn draw(
        &self,
        target: &mut glium::Frame,
        perspective: &Matrix4<f32>,
        view: &Matrix4<f32>,
        trajectory: &Trajectory,
        drawing_parameters: &DrawParameters,
    ) {
        let index_buffer = NoIndices(PrimitiveType::LineStrip);

        target
            .draw(
                trajectory.buffer(),
                &index_buffer,
                &self.program,
                &uniform! {
                    perspective: perspective.data.0,
                    view: view.data.0,
                },
                &drawing_parameters,
            )
            .unwrap();
    }
}
