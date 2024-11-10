mod cube;
mod cuber_drawer;
mod diagonal_drawer;
mod infinite_grid_drawer;
mod trajectory;
mod trajectory_drawer;
mod vertex;

use core::f32;
use std::{
    i64,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

use chrono::{Local, TimeDelta};
use concurrent_queue::ConcurrentQueue;
use cube::CubeBuilder;
use cuber_drawer::CubeDrawer;
use diagonal_drawer::DiagonalDrawer;
use egui::{DragValue, ViewportId, Widget};
use glium::{Blend, Surface};
use infinite_grid_drawer::InfiniteGridDrawer;
use nalgebra::{Matrix4, Point3, Quaternion, UnitQuaternion, Vector3, Vector4};
use trajectory::Trajectory;
use trajectory_drawer::TrajectoryDrawer;
use winit::event::{self, ElementState, MouseButton};

fn main() {
    let width = 1600;
    let height = 1200;

    let event_loop = winit::event_loop::EventLoopBuilder::new().build().unwrap();
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Spinning top")
        .with_inner_size(width, height)
        .build(&event_loop);

    let mut egui_glium =
        egui_glium::EguiGlium::new(ViewportId::ROOT, &display, &window, &event_loop);

    let drawing_parameters = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: false,
            ..Default::default()
        },
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        blend: Blend::alpha_blending(),
        ..Default::default()
    };

    let mut perspective = Matrix4::new_perspective(
        width as f32 / height as f32,
        std::f32::consts::PI / 2.0,
        0.1,
        100.0,
    );

    let mut mouse_position = (0.0, 0.0);
    let mut camera_direction = Vector3::new(0.0f32, 0.0, 1.0);
    let mut camera_angle = Vector3::new(0.0f32, 0.0, 0.0);
    let mut camera_up = Vector3::new(0.0f32, 1.0, 0.0);
    let mut camera_distant = 5.0f32;
    let mut view = Matrix4::look_at_rh(
        &Point3::from_slice((-camera_distant * camera_direction).as_slice()),
        &Point3::new(0.0, 0.0, 0.0),
        &camera_up,
    );
    let mut camera_move_button_pressed = false;

    let infinite_grid_drawer = InfiniteGridDrawer::new(&display);

    let mut cube = CubeBuilder::default()
        .size(1.0)
        .density(1.0)
        .base_rotation(
            UnitQuaternion::from_euler_angles(-(2.0 / 3.0f32).sqrt().acos(), 0.0, 0.0)
                * UnitQuaternion::from_euler_angles(0.0, 0.0, f32::consts::PI / 4.0),
        )
        .rotation(UnitQuaternion::identity())
        .build()
        .unwrap();

    let cube_drawer = CubeDrawer::new(&display);
    let diagonal_drawer = DiagonalDrawer::new(&display);

    let mut cube_size = cube.size();
    let mut cube_density = 1f32;
    let mut cube_deviation = 0f32;
    let mut angular_velocity = 1f32;
    let mut integration_step = 0.001f32;

    let shared_rotation = Arc::new(Mutex::<UnitQuaternion<f32>>::new(UnitQuaternion::identity()));
    let shared_run = Arc::new(Mutex::new(false));
    let mut simulation_thread = None;

    let trajectory_size = 500000;
    let trajectory_queue = Arc::new(ConcurrentQueue::<Vector3<f32>>::unbounded());
    let mut trajectory = Trajectory::new(trajectory_size, &display);
    let trajectory_drawer = TrajectoryDrawer::new(&display);

    let mut previous_time = Local::now();

    let _ = event_loop.run(move |event, window_target| {
        let mut redraw = || {
            let current_time = Local::now();
            let duration = current_time - previous_time;
            let duration_in_seconds = duration.num_microseconds().unwrap_or(1) as f64 / 1_000_000.0;
            let fps = 1.0 / duration_in_seconds;
            previous_time = current_time;

            egui_glium.run(&window, |egui_ctx| {
                egui::Window::new("panel").show(egui_ctx, |ui| {
                    if ui.button("Start").clicked() && simulation_thread.is_none() {
                        let shared_rotation = shared_rotation.clone();
                        let shared_run = shared_run.clone();
                        *shared_run.lock().unwrap() = true;
                        let moment_of_interia = cube.get_moment_of_interia();
                        let weight = cube.get_weight();
                        let trajectory_queue = trajectory_queue.clone();
                        trajectory.clear();
                        simulation_thread = Some(thread::spawn(move || {
                            let mut previous_time = Local::now();
                            let mut tick = TimeDelta::zero();
                            let step =
                                TimeDelta::microseconds((integration_step * 1_000_000.0) as i64);
                            let h = integration_step;
                            let mut w = Vector3::new(0f32, angular_velocity, 0f32);
                            let moment_of_interia = moment_of_interia;
                            let inversed_moment_of_interia =
                                moment_of_interia.try_inverse().unwrap();
                            let top = Vector3::new(0f32, cube_size * 3f32.sqrt(), 0f32);
                            let center = top / 2.0;
                            let weight = weight;
                            let f = Vector3::new(0f32, -weight * 9.81, 0f32);
                            let mut q = shared_rotation.lock().unwrap();
                            *q = UnitQuaternion::from_euler_angles(cube_deviation, 0f32, 0f32);
                            let mut q_copy = q.clone();
                            drop(q);

                            let mut run = shared_run.lock().unwrap().clone();

                            while run {
                                let current_time = Local::now();
                                let duration = current_time - previous_time;
                                tick += duration;
                                previous_time = current_time;
                                if tick <= step {
                                    sleep(Duration::from_micros(
                                        (step - tick).num_microseconds().unwrap_or(i64::MAX) as u64,
                                    ));
                                    tick = TimeDelta::zero();
                                } else {
                                    tick -= step;
                                }

                                let rotation_matrix = q_copy.to_rotation_matrix();
                                trajectory_queue.push(rotation_matrix * top).unwrap();

                                let mut q = q_copy.quaternion().clone();

                                let k1_w = h
                                    * inversed_moment_of_interia
                                    * (f.cross(
                                        &(-(UnitQuaternion::from_quaternion(q)
                                            .to_rotation_matrix()
                                            * center)),
                                    ) + (moment_of_interia * w).cross(&w));

                                let wq = Quaternion::new(0.0, w.x, w.y, w.z);
                                let k1_q = h * q * wq / 2.0;

                                let k2_w = h
                                    * inversed_moment_of_interia
                                    * (f.cross(
                                        &(-(UnitQuaternion::from_quaternion(q + k1_q / 2.0)
                                            .to_rotation_matrix()
                                            * center)),
                                    ) + (moment_of_interia * (w + k1_w / 2.0))
                                        .cross(&(w + k1_w / 2.0)));

                                let wq = Quaternion::new(
                                    0.0,
                                    (w + k1_w / 2.0).x,
                                    (w + k1_w / 2.0).y,
                                    (w + k1_w / 2.0).z,
                                );
                                let k2_q = h * (q + k1_q / 2.0) * wq / 2.0;

                                let k3_w = h
                                    * inversed_moment_of_interia
                                    * (f.cross(
                                        &(-(UnitQuaternion::from_quaternion(q + k2_q / 2.0)
                                            .to_rotation_matrix()
                                            * center)),
                                    ) + (moment_of_interia * (w + k2_w / 2.0))
                                        .cross(&(w + k2_w / 2.0)));

                                let wq = Quaternion::new(
                                    0.0,
                                    (w + k2_w / 2.0).x,
                                    (w + k2_w / 2.0).y,
                                    (w + k2_w / 2.0).z,
                                );
                                let k3_q = h * (q + k2_q / 2.0) * wq / 2.0;

                                let k4_w = h
                                    * inversed_moment_of_interia
                                    * (f.cross(
                                        &(-(UnitQuaternion::from_quaternion(q + k3_q)
                                            .to_rotation_matrix()
                                            * center)),
                                    ) + (moment_of_interia * (w + k3_w)).cross(&(w + k3_w)));

                                let wq =
                                    Quaternion::new(0.0, (w + k3_w).x, (w + k3_w).y, (w + k3_w).z);
                                let k4_q = h * (q + k3_q) * wq / 2.0;

                                let dw = (k1_w + 2.0 * k2_w + 2.0 * k3_w + k4_w) / 6.0;
                                w += dw;

                                let dq = (k1_q + 2.0 * k2_q + 2.0 * k3_q + k4_q) / 6.0;
                                q += dq;

                                q_copy = UnitQuaternion::from_quaternion(q);

                                let mut q = shared_rotation.lock().unwrap();
                                *q = q_copy.clone();
                                drop(q);

                                run = shared_run.lock().unwrap().clone();
                            }
                        }));
                    }

                    if ui.button("Stop").clicked() && simulation_thread.is_some() {
                        let mut run = shared_run.lock().unwrap();
                        *run = false;
                        drop(run);

                        let st = simulation_thread.take();

                        st.unwrap().join().unwrap();
                    }

                    ui.horizontal(|ui| {
                        if DragValue::new(&mut cube_size)
                            .clamp_range(0.1..=10.0)
                            .speed(0.1)
                            .ui(ui)
                            .changed()
                            && simulation_thread.is_none()
                        {
                            cube.set_size(cube_size);
                        }

                        ui.label("cube size");
                    });

                    ui.horizontal(|ui| {
                        if DragValue::new(&mut cube_density)
                            .clamp_range(0.01..=10.0)
                            .speed(0.01)
                            .ui(ui)
                            .changed()
                            && simulation_thread.is_none()
                        {
                            cube.set_density(cube_density);
                        }

                        ui.label("cube density");
                    });

                    ui.horizontal(|ui| {
                        if DragValue::new(&mut cube_deviation)
                            .clamp_range(
                                (-std::f32::consts::PI / 3.0)..=(std::f32::consts::PI / 3.0),
                            )
                            .speed(0.01)
                            .ui(ui)
                            .changed()
                            && simulation_thread.is_none()
                        {
                            let mut q = shared_rotation.lock().unwrap();
                            *q = UnitQuaternion::from_euler_angles(cube_deviation, 0f32, 0f32);
                        }

                        ui.label("cube deviation");
                    });

                    ui.horizontal(|ui| {
                        DragValue::new(&mut angular_velocity)
                            .clamp_range(0.0..=60.0)
                            .speed(0.01)
                            .ui(ui);

                        ui.label("angular velocity");
                    });

                    ui.horizontal(|ui| {
                        DragValue::new(&mut integration_step)
                            .clamp_range(0.0001..=0.1)
                            .speed(0.0001)
                            .ui(ui);

                        ui.label("integration step");
                    });

                    ui.label(format!("FPS: {:.1}", fps));
                });
            });

            window.request_redraw();

            cube.set_rotation(shared_rotation.lock().unwrap().clone());

            trajectory.add_points(trajectory_queue.clone());

            let mut target = display.draw();

            target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

            cube_drawer.draw(&mut target, &perspective, &view, &cube, &drawing_parameters);
            diagonal_drawer.draw(&mut target, &perspective, &view, &cube, &drawing_parameters);

            if !trajectory.points().is_empty() {
                trajectory_drawer.draw(
                    &mut target,
                    &perspective,
                    &view,
                    &trajectory,
                    &drawing_parameters,
                );
            }

            infinite_grid_drawer.draw(&mut target, &perspective, &view, &drawing_parameters);

            egui_glium.paint(&display, &mut target);

            target.finish().unwrap();
        };

        match event {
            event::Event::WindowEvent { event, .. } => {
                use event::WindowEvent;
                match &event {
                    WindowEvent::RedrawRequested => redraw(),
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                        window_target.exit();
                    }
                    WindowEvent::Resized(new_size) => {
                        display.resize((*new_size).into());
                        perspective = Matrix4::new_perspective(
                            new_size.width as f32 / new_size.height as f32,
                            std::f32::consts::PI / 2.0,
                            0.1,
                            100.0,
                        );
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let delta = (position.x - mouse_position.0, position.y - mouse_position.1);
                        mouse_position = (position.x, position.y);
                        if camera_move_button_pressed {
                            camera_angle.x += delta.1 as f32 * 0.01;
                            camera_angle.y += delta.0 as f32
                                * 0.01
                                * if camera_angle.x.cos() < 0.0 {
                                    -1.0
                                } else {
                                    1.0
                                };
                            camera_direction =
                                (Matrix4::from_euler_angles(camera_angle.x, camera_angle.y, 0.0)
                                    * Vector4::new(0.0, 0.0, 1.0, 0.0))
                                .xyz();
                            camera_up =
                                (Matrix4::from_euler_angles(camera_angle.x, camera_angle.y, 0.0)
                                    * Vector4::new(0.0, 1.0, 0.0, 0.0))
                                .xyz();
                            view = Matrix4::look_at_rh(
                                &Point3::from_slice(
                                    (-camera_distant * camera_direction).as_slice(),
                                ),
                                &Point3::new(0.0, 0.0, 0.0),
                                &camera_up,
                            );
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if *button == MouseButton::Middle {
                            camera_move_button_pressed = *state == ElementState::Pressed;
                        }
                    }
                    WindowEvent::KeyboardInput {
                        device_id: _,
                        event,
                        is_synthetic: _,
                    } => {
                        if event.logical_key == "c" && event.state.is_pressed() && !event.repeat {
                            camera_move_button_pressed = !camera_move_button_pressed;
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => match delta {
                        event::MouseScrollDelta::LineDelta(_x, y) => {
                            camera_distant += -y * 0.1;
                            view = Matrix4::look_at_rh(
                                &Point3::from_slice(
                                    (-camera_distant * camera_direction).as_slice(),
                                ),
                                &Point3::new(0.0, 0.0, 0.0),
                                &camera_up,
                            );
                        }
                        _ => {}
                    },
                    WindowEvent::TouchpadMagnify { delta, .. } => {
                        camera_distant -= *delta as f32 * 3.0;
                        view = Matrix4::look_at_rh(
                            &Point3::from_slice((-camera_distant * camera_direction).as_slice()),
                            &Point3::new(0.0, 0.0, 0.0),
                            &camera_up,
                        );
                    }
                    _ => {}
                }

                let event_response = egui_glium.on_event(&window, &event);

                if event_response.repaint {
                    window.request_redraw();
                }
            }
            event::Event::NewEvents(event::StartCause::ResumeTimeReached { .. }) => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
