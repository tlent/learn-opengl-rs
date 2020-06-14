mod camera;
mod model;
mod shader_program;
mod texture;

use std::time::Instant;

use glutin::{Api, ContextBuilder, GlProfile, GlRequest};
use nalgebra_glm as glm;
use winit::{
    event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

use camera::{Camera, CameraMotion};
use model::Model;
use shader_program::ShaderProgram;

const VERTEX_SHADER: &str = include_str!("shaders/default.vert");
const FRAGMENT_SHADER: &str = include_str!("shaders/default.frag");
const LIGHT_SOURCE_FRAGMENT_SHADER: &str = include_str!("shaders/light-source.frag");

const MULTISAMPLING_SAMPLES: u16 = 4;

fn main() {
    let event_loop = EventLoop::new();
    let monitor = event_loop.primary_monitor();
    let monitor_size = monitor.size();
    let window_builder = WindowBuilder::new()
        .with_title("Learn OpenGL")
        .with_fullscreen(Some(Fullscreen::Borderless(monitor)));
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

    gl::load_with(|s| context.get_proc_address(s));
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::MULTISAMPLE);
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        context.swap_buffers().unwrap();
    }

    let light_color = glm::vec3(1.0, 1.0, 1.0);
    let point_light_position = glm::vec3(2.0, 2.0, 2.0);
    let light_source_shader =
        ShaderProgram::new(VERTEX_SHADER, LIGHT_SOURCE_FRAGMENT_SHADER).unwrap();

    let default_shader = ShaderProgram::new(VERTEX_SHADER, FRAGMENT_SHADER).unwrap();
    let diffuse_color = light_color * 0.5;
    let ambient_color = diffuse_color * 0.2;
    unsafe {
        default_shader.use_program();
        default_shader.set_uniform_mat4f("model", glm::Mat4::identity());

        default_shader.set_uniform_vec3f("directionalLight.direction", glm::vec3(-0.2, -1.0, -0.3));
        default_shader.set_uniform_vec3f("directionalLight.ambient", ambient_color);
        default_shader.set_uniform_vec3f("directionalLight.diffuse", diffuse_color);
        default_shader.set_uniform_vec3f("directionalLight.specular", light_color);

        default_shader.set_uniform_vec3f("pointLight.position", point_light_position);
        default_shader.set_uniform_float("pointLight.constant", 1.0);
        default_shader.set_uniform_float("pointLight.linear", 0.09);
        default_shader.set_uniform_float("pointLight.quadratic", 0.032);
        default_shader.set_uniform_vec3f("pointLight.ambient", ambient_color);
        default_shader.set_uniform_vec3f("pointLight.diffuse", diffuse_color);
        default_shader.set_uniform_vec3f("pointLight.specular", light_color);

        // default_shader.set_uniform_float("spotLight.innerCutoff", 12.5f32.to_radians().cos());
        // default_shader.set_uniform_float("spotLight.outerCutoff", 17.5f32.to_radians().cos());
        // default_shader.set_uniform_float("spotLight.linear", 0.09);
        // default_shader.set_uniform_float("spotLight.quadratic", 0.032);
        // default_shader.set_uniform_vec3f("spotLight.ambient", ambient_color);
        // default_shader.set_uniform_vec3f("spotLight.ambient", ambient_color);
        // default_shader.set_uniform_vec3f("spotLight.diffuse", diffuse_color);
        // default_shader.set_uniform_vec3f("spotLight.specular", light_color);

        default_shader.set_uniform_float("material.shininess", 32.0);
    }

    let cube = unsafe { Model::load("models/cube/cube.obj").unwrap() };
    let backpack = unsafe { Model::load("models/backpack/backpack.obj").unwrap() };

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
    let mut aspect_ratio = monitor_size.width as f64 / monitor_size.height as f64;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(window_size) => {
                    aspect_ratio = window_size.width as f64 / window_size.height as f64;
                    unsafe {
                        gl::Viewport(0, 0, window_size.width as i32, window_size.height as i32);
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

                unsafe {
                    gl::ClearColor(0.1, 0.1, 0.1, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                    let view = camera.view_matrix();
                    let projection = glm::perspective(
                        aspect_ratio as f32,
                        camera.fov().to_radians(),
                        0.1,
                        100.0,
                    );

                    light_source_shader.use_program();
                    light_source_shader.set_uniform_mat4f("view", view);
                    light_source_shader.set_uniform_mat4f("projection", projection);
                    let mut model = glm::translate(&glm::Mat4::identity(), &point_light_position);
                    model = glm::scale(&model, &glm::vec3(0.2, 0.2, 0.2));
                    light_source_shader.set_uniform_mat4f("model", model);
                    light_source_shader.set_uniform_vec3f("color", light_color);
                    cube.draw(&light_source_shader);

                    default_shader.use_program();
                    default_shader.set_uniform_vec3f("viewPos", camera.position());
                    default_shader.set_uniform_mat4f("view", view);
                    default_shader.set_uniform_mat4f("projection", projection);
                    // default_shader.set_uniform_vec3f("spotLight.position", camera.position());
                    // default_shader.set_uniform_vec3f("spotLight.direction", camera.front());

                    backpack.draw(&default_shader);
                }

                context.swap_buffers().unwrap();
            }
            _ => {}
        }
    });
}
