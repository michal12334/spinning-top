use derive_builder::Builder;
use derive_getters::Getters;
use derive_new::new;
use nalgebra::{Matrix3, Matrix4, UnitQuaternion};

#[derive(Debug, Clone, Getters, new, Builder)]
pub struct Cube {
    #[getter(copy)]
    size: f32,
    #[getter(copy)]
    weight: f32,
    #[getter(copy)]
    rotation: UnitQuaternion<f32>,
    #[getter(copy)]
    base_rotation: UnitQuaternion<f32>,
}

impl Cube {
    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        self.rotation.to_rotation_matrix().to_homogeneous()
            * self.base_rotation.to_rotation_matrix().to_homogeneous()
            * Matrix4::new_scaling(self.size)
    }

    pub fn get_diagonal_model_matrix(&self) -> Matrix4<f32> {
        self.rotation.to_rotation_matrix().to_homogeneous() * Matrix4::new_scaling(self.size)
    }

    pub fn update_rotation(&mut self, rotation: UnitQuaternion<f32>) {
        self.rotation = rotation;
    }

    pub fn update_size(&mut self, size: f32) {
        self.size = size;
    }

    pub fn get_moment_of_interia(&self) -> Matrix3<f32> {
        Matrix3::new(
            self.weight * self.size * self.size / 6.0,
            0.0,
            0.0,
            0.0,
            5.0 * self.weight * self.size * self.size / 12.0,
            0.0,
            0.0,
            0.0,
            self.weight * self.size * self.size / 6.0,
        )
    }
}
