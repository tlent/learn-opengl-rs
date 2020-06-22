mod camera;
mod model;
mod shader_program;
mod texture;

use std::ffi::c_void;
use std::mem;
use std::time::Instant;

use gl::types::*;
use glutin::{Api, ContextBuilder, GlProfile, GlRequest};
use nalgebra_glm as glm;
use rand::prelude::*;
use winit::{
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use camera::{Camera, CameraMotion};
use model::Model;
use shader_program::ShaderProgram;

const VERTEX_SHADER: &str = include_str!("shaders/main.vert");
const FRAGMENT_SHADER: &str = include_str!("shaders/main.frag");

const INSTANCED_VERTEX_SHADER: &str = include_str!("shaders/instanced.vert");

const MULTISAMPLING_SAMPLES: u16 = 4;

const ASTEROID_COUNT: usize = 100000;

fn main() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("Learn OpenGL")
        .with_inner_size(LogicalSize::new(800, 600));
    let context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core)
        .with_vsync(true)
        .with_multisampling(MULTISAMPLING_SAMPLES)
        .build_windowed(window_builder, &event_loop)
        .unwrap();
    let context = unsafe { context.make_current().unwrap() };
    context.window().set_cursor_grab(true).unwrap();
    context.window().set_cursor_visible(false);
    let mut window_size = context.window().inner_size();

    gl::load_with(|s| context.get_proc_address(s));
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::MULTISAMPLE);
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        context.swap_buffers().unwrap();
    }

    let mut rng = rand::thread_rng();
    let radius = 150.0;
    let offset = 25.0;
    let mut asteroid_models = Vec::with_capacity(ASTEROID_COUNT);
    for i in 0..ASTEROID_COUNT {
        let mut model = glm::Mat4::identity();

        let angle = i as f32 / ASTEROID_COUNT as f32 * 360.0;
        let x = angle.sin() * radius + rng.gen_range(-offset, offset);
        let y = rng.gen_range(-offset, offset) * 0.4;
        let z = angle.cos() * radius + rng.gen_range(-offset, offset);
        model = glm::translate(&model, &glm::vec3(x, y, z));

        let scale = rng.gen_range(0.05, 0.25);
        model = glm::scale(&model, &glm::vec3(scale, scale, scale));

        let rotation = rng.gen_range(0, 360) as f32;
        model = glm::rotate(&model, rotation, &glm::vec3(0.4, 0.6, 0.8));

        asteroid_models.push(model);
    }

    let planet = unsafe { Model::load("resources/models/planet/planet.obj").unwrap() };
    let main_shader = ShaderProgram::new(VERTEX_SHADER, FRAGMENT_SHADER, None).unwrap();

    let asteroid = unsafe { Model::load("resources/models/rock/rock.obj").unwrap() };
    let instanced_shader =
        ShaderProgram::new(INSTANCED_VERTEX_SHADER, FRAGMENT_SHADER, None).unwrap();

    let mut buffer = 0;
    unsafe {
        gl::GenBuffers(1, &mut buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (mem::size_of::<glm::Mat4>() * asteroid_models.len()) as GLsizeiptr,
            asteroid_models.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );

        for mesh in asteroid.meshes.iter() {
            gl::BindVertexArray(mesh.vao);
            gl::VertexAttribPointer(
                3,
                4,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<glm::Mat4>() as GLsizei,
                0 as *const c_void,
            );
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribDivisor(3, 1);
            gl::VertexAttribPointer(
                4,
                4,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<glm::Mat4>() as GLsizei,
                mem::size_of::<glm::Vec4>() as *const c_void,
            );
            gl::EnableVertexAttribArray(4);
            gl::VertexAttribDivisor(4, 1);
            gl::VertexAttribPointer(
                5,
                4,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<glm::Mat4>() as GLsizei,
                (2 * mem::size_of::<glm::Vec4>()) as *const c_void,
            );
            gl::EnableVertexAttribArray(5);
            gl::VertexAttribDivisor(5, 1);
            gl::VertexAttribPointer(
                6,
                4,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<glm::Mat4>() as GLsizei,
                (3 * mem::size_of::<glm::Vec4>()) as *const c_void,
            );
            gl::EnableVertexAttribArray(6);
            gl::VertexAttribDivisor(6, 1);
        }
    }

    let mut prev_frame_time = Instant::now();
    let mut delta_time = 0.0f32;
    let mut time = delta_time;
    let mut pressed_keys = Vec::with_capacity(10);
    let mut mouse_delta = (0.0, 0.0);
    let mut scroll_delta = 0.0;
    let mut camera = Camera::new(
        glm::vec3(0.0, 0.0, 3.0),
        glm::vec3(0.0, 1.0, 0.0),
        -90.0,
        0.0,
    );
    let mut window_is_focused = true;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    window_size = size;
                    unsafe {
                        gl::Viewport(0, 0, size.width as GLint, size.height as GLint);
                    }
                }
                WindowEvent::KeyboardInput { input, .. } => match input.state {
                    ElementState::Pressed => match input.virtual_keycode {
                        Some(VirtualKeyCode::Escape) => *control_flow = ControlFlow::Exit,
                        Some(key) => {
                            if !pressed_keys.contains(&key) {
                                pressed_keys.push(key)
                            }
                        }
                        _ => {}
                    },
                    ElementState::Released => match input.virtual_keycode {
                        Some(key) => {
                            if let Some(i) = pressed_keys.iter().position(|&k| k == key) {
                                pressed_keys.swap_remove(i);
                            }
                        }
                        _ => {}
                    },
                },
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(_, dy),
                    ..
                } => {
                    scroll_delta -= dy;
                }
                WindowEvent::Focused(is_focused) => window_is_focused = is_focused,
                _ => {}
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion {
                    delta: (dx, dy), ..
                } => {
                    if !window_is_focused {
                        return;
                    }
                    let (x, y) = mouse_delta;
                    mouse_delta = (x + dx as f32, y - dy as f32);
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                let now = Instant::now();
                delta_time = (now - prev_frame_time).as_secs_f32();
                time += delta_time;
                prev_frame_time = now;

                let camera_directions: Vec<_> = pressed_keys
                    .iter()
                    .filter_map(|key| match key {
                        VirtualKeyCode::W => Some(CameraMotion::Forward),
                        VirtualKeyCode::S => Some(CameraMotion::Backward),
                        VirtualKeyCode::A => Some(CameraMotion::Left),
                        VirtualKeyCode::D => Some(CameraMotion::Right),
                        VirtualKeyCode::Space => Some(CameraMotion::Up),
                        VirtualKeyCode::X => Some(CameraMotion::Down),
                        _ => None,
                    })
                    .collect();
                camera.move_(&camera_directions, delta_time);
                camera.look(mouse_delta);
                mouse_delta = (0.0, 0.0);
                camera.zoom(scroll_delta);
                scroll_delta = 0.0;

                let view = camera.view_matrix();
                let projection = glm::perspective(
                    window_size.width as f32 / window_size.height as f32,
                    (45.0f32).to_radians(),
                    0.1,
                    100.0,
                );

                unsafe {
                    gl::ClearColor(0.1, 0.1, 0.1, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                    main_shader.use_program();
                    main_shader.set_uniform_mat4f("view", view);
                    main_shader.set_uniform_mat4f("projection", projection);

                    let mut model = glm::Mat4::identity();
                    model = glm::translate(&model, &glm::vec3(0.0, -3.0, 0.0));
                    model = glm::scale(&model, &glm::vec3(4.0, 4.0, 4.0));
                    main_shader.set_uniform_mat4f("model", model);
                    planet.draw(&main_shader);

                    instanced_shader.use_program();
                    instanced_shader.set_uniform_mat4f("view", view);
                    instanced_shader.set_uniform_mat4f("projection", projection);
                    for mesh in asteroid.meshes.iter() {
                        mesh.diffuse_textures[0].bind();
                        gl::BindVertexArray(mesh.vao);
                        gl::DrawElementsInstanced(
                            gl::TRIANGLES,
                            mesh.indices.len() as GLsizei,
                            gl::UNSIGNED_INT,
                            0 as *const c_void,
                            ASTEROID_COUNT as GLsizei,
                        )
                    }
                }
                context.swap_buffers().unwrap();
            }
            Event::LoopDestroyed => {}
            _ => {}
        }
    });
}
