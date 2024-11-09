use derive_builder::Builder;
use derive_getters::Getters;
use derive_new::new;
use derive_setters::Setters;
use nalgebra::{Matrix3, Matrix4, UnitQuaternion};

#[derive(Debug, Clone, Getters, Setters, new, Builder)]
#[setters(generate = false, prefix = "set_", borrow_self)]
pub struct Cube {
    #[getter(copy)]
    #[setters(generate)]
    size: f32,
    #[getter(copy)]
    #[setters(generate)]
    density: f32,
    #[getter(copy)]
    #[setters(generate)]
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

    pub fn get_weight(&self) -> f32 {
        self.density * self.size * self.size * self.size
    }

    pub fn get_moment_of_interia(&self) -> Matrix3<f32> {
        let weight = self.get_weight();

        Matrix3::new(
            weight * self.size * self.size / 6.0,
            0.0,
            0.0,
            0.0,
            5.0 * weight * self.size * self.size / 12.0,
            0.0,
            0.0,
            0.0,
            weight * self.size * self.size / 6.0,
        )
    }
}
