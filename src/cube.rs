use derive_builder::Builder;
use derive_getters::Getters;
use derive_new::new;
use derive_setters::Setters;
use nalgebra::{Matrix3, Matrix4, UnitQuaternion, Vector3};

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

    pub fn get_gravity_vector_model_matrix(&self) -> Matrix4<f32> {
        let center = Vector3::new(0f32, self.size * 3f32.sqrt() / 2f32, 0f32);
        let rotated_center = self.rotation.to_rotation_matrix() * center;
        Matrix4::new_translation(&rotated_center)
    }

    pub fn get_weight(&self) -> f32 {
        self.density * self.size * self.size * self.size
    }

    pub fn get_moment_of_interia(&self) -> Matrix3<f32> {
        let weight = self.get_weight();

        let base_i = weight * self.size * self.size / 6.0;

        let mx = self.base_rotation.to_rotation_matrix().inverse() * Vector3::x();
        let my = self.base_rotation.to_rotation_matrix().inverse() * Vector3::y();
        let mz = self.base_rotation.to_rotation_matrix().inverse() * Vector3::z();

        Matrix3::from_diagonal(&Vector3::new(
            base_i * mx.dot(&mx),
            base_i * my.dot(&my),
            base_i * mz.dot(&mz),
        )) + Matrix3::new(
            3.0 * weight * self.size * self.size / 4.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            3.0 * weight * self.size * self.size / 4.0,
        )
    }
}
