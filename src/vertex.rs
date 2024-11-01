use derive_getters::Getters;
use derive_new::new;
use glium::implement_vertex;

#[derive(Debug, Clone, Copy, Getters, new)]
pub struct Vertex {
    position: [f32; 3],
}

implement_vertex!(Vertex, position);
