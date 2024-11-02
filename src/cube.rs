use derive_builder::Builder;
use derive_getters::Getters;
use derive_new::new;
use nalgebra::{Matrix4, UnitQuaternion};

#[derive(Debug, Clone, Getters, new, Builder)]
pub struct Cube {
    size: f32,
    rotation: UnitQuaternion<f32>,
    base_rotation: UnitQuaternion<f32>,
}

impl Cube {
    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        self.rotation.to_rotation_matrix().to_homogeneous()
            * self.base_rotation.to_rotation_matrix().to_homogeneous()
            * Matrix4::new_scaling(self.size)
    }

    pub fn get_scale_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_scaling(self.size)
    }
}
