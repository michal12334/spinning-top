use std::{collections::LinkedList, sync::Arc};

use concurrent_queue::ConcurrentQueue;
use derive_getters::Getters;
use glium::{glutin::surface::WindowSurface, Display, VertexBuffer};
use nalgebra::Vector3;

use crate::vertex::Vertex;

#[derive(Getters)]
pub struct Trajectory {
    points: LinkedList<Vertex>,
    buffer: VertexBuffer<Vertex>,
    size: usize,
}

impl Trajectory {
    pub fn new(size: usize, display: &Display<WindowSurface>) -> Self {
        let buffer = VertexBuffer::empty_dynamic(display, size).unwrap();
        Self {
            buffer,
            size,
            points: LinkedList::new(),
        }
    }

    pub fn add_points(&mut self, points_qeue: Arc<ConcurrentQueue<Vector3<f32>>>) {
        while !points_qeue.is_empty() {
            let point = points_qeue.pop().unwrap();
            let point = Vertex::new([point.x, point.y, point.z]);
            if self.points.len() < self.size {
                self.points.push_back(point);
            } else {
                self.points.pop_front();
                self.points.push_back(point);
            }
        }

        if !self.points.is_empty() {
            let last_point = self.points.back().unwrap();
            self.buffer.write(
                &self
                    .points
                    .iter()
                    .chain(vec![last_point; self.size - self.points.len()])
                    .map(|f| *f)
                    .collect::<Vec<Vertex>>(),
            );
        }
    }

    pub fn clear(&mut self) {
        self.points.clear();
    }

    pub fn resize(&mut self, size: usize, display: &Display<WindowSurface>) {
        self.size = size;
        while self.points.len() > size {
            self.points.pop_front();
        }
        self.buffer = VertexBuffer::empty_dynamic(display, size).unwrap();
    }
}
