use std::ffi::c_void;
use std::mem;
use std::ptr;
use std::time::Instant;

use glutin::{Api, ContextBuilder, GlProfile, GlRequest};
use nalgebra_glm as glm;
use winit::{
    event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

mod camera;
mod shader_program;

use camera::{Camera, CameraMotion};
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
    }

    let cube_vertices = [
        Vertex::new(glm::vec3(-0.5, -0.5, -0.5), glm::vec3(0.0, 0.0, -1.0)),
        Vertex::new(glm::vec3(0.5, -0.5, -0.5), glm::vec3(0.0, 0.0, -1.0)),
        Vertex::new(glm::vec3(0.5, 0.5, -0.5), glm::vec3(0.0, 0.0, -1.0)),
        Vertex::new(glm::vec3(0.5, 0.5, -0.5), glm::vec3(0.0, 0.0, -1.0)),
        Vertex::new(glm::vec3(-0.5, 0.5, -0.5), glm::vec3(0.0, 0.0, -1.0)),
        Vertex::new(glm::vec3(-0.5, -0.5, -0.5), glm::vec3(0.0, 0.0, -1.0)),
        Vertex::new(glm::vec3(-0.5, -0.5, 0.5), glm::vec3(0.0, 0.0, 1.0)),
        Vertex::new(glm::vec3(0.5, -0.5, 0.5), glm::vec3(0.0, 0.0, 1.0)),
        Vertex::new(glm::vec3(0.5, 0.5, 0.5), glm::vec3(0.0, 0.0, 1.0)),
        Vertex::new(glm::vec3(0.5, 0.5, 0.5), glm::vec3(0.0, 0.0, 1.0)),
        Vertex::new(glm::vec3(-0.5, 0.5, 0.5), glm::vec3(0.0, 0.0, 1.0)),
        Vertex::new(glm::vec3(-0.5, -0.5, 0.5), glm::vec3(0.0, 0.0, 1.0)),
        Vertex::new(glm::vec3(-0.5, 0.5, 0.5), glm::vec3(-1.0, 0.0, 0.0)),
        Vertex::new(glm::vec3(-0.5, 0.5, -0.5), glm::vec3(-1.0, 0.0, 0.0)),
        Vertex::new(glm::vec3(-0.5, -0.5, -0.5), glm::vec3(-1.0, 0.0, 0.0)),
        Vertex::new(glm::vec3(-0.5, -0.5, -0.5), glm::vec3(-1.0, 0.0, 0.0)),
        Vertex::new(glm::vec3(-0.5, -0.5, 0.5), glm::vec3(-1.0, 0.0, 0.0)),
        Vertex::new(glm::vec3(-0.5, 0.5, 0.5), glm::vec3(-1.0, 0.0, 0.0)),
        Vertex::new(glm::vec3(0.5, 0.5, 0.5), glm::vec3(1.0, 0.0, 0.0)),
        Vertex::new(glm::vec3(0.5, 0.5, -0.5), glm::vec3(1.0, 0.0, 0.0)),
        Vertex::new(glm::vec3(0.5, -0.5, -0.5), glm::vec3(1.0, 0.0, 0.0)),
        Vertex::new(glm::vec3(0.5, -0.5, -0.5), glm::vec3(1.0, 0.0, 0.0)),
        Vertex::new(glm::vec3(0.5, -0.5, 0.5), glm::vec3(1.0, 0.0, 0.0)),
        Vertex::new(glm::vec3(0.5, 0.5, 0.5), glm::vec3(1.0, 0.0, 0.0)),
        Vertex::new(glm::vec3(-0.5, -0.5, -0.5), glm::vec3(0.0, -1.0, 0.0)),
        Vertex::new(glm::vec3(0.5, -0.5, -0.5), glm::vec3(0.0, -1.0, 0.0)),
        Vertex::new(glm::vec3(0.5, -0.5, 0.5), glm::vec3(0.0, -1.0, 0.0)),
        Vertex::new(glm::vec3(0.5, -0.5, 0.5), glm::vec3(0.0, -1.0, 0.0)),
        Vertex::new(glm::vec3(-0.5, -0.5, 0.5), glm::vec3(0.0, -1.0, 0.0)),
        Vertex::new(glm::vec3(-0.5, -0.5, -0.5), glm::vec3(0.0, -1.0, 0.0)),
        Vertex::new(glm::vec3(-0.5, 0.5, -0.5), glm::vec3(0.0, 1.0, 0.0)),
        Vertex::new(glm::vec3(0.5, 0.5, -0.5), glm::vec3(0.0, 1.0, 0.0)),
        Vertex::new(glm::vec3(0.5, 0.5, 0.5), glm::vec3(0.0, 1.0, 0.0)),
        Vertex::new(glm::vec3(0.5, 0.5, 0.5), glm::vec3(0.0, 1.0, 0.0)),
        Vertex::new(glm::vec3(-0.5, 0.5, 0.5), glm::vec3(0.0, 1.0, 0.0)),
        Vertex::new(glm::vec3(-0.5, 0.5, -0.5), glm::vec3(0.0, 1.0, 0.0)),
    ];

    let mut vaos = [0; 2];
    let mut vbos = [0; 1];
    unsafe {
        gl::GenVertexArrays(vaos.len() as i32, vaos.as_mut_ptr());
        gl::GenBuffers(vbos.len() as i32, vbos.as_mut_ptr());
        gl::BindVertexArray(vaos[0]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (cube_vertices.len() * mem::size_of::<Vertex>()) as isize,
            cube_vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as i32,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::BindVertexArray(vaos[1]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[0]);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as i32,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as i32,
            (3 * mem::size_of::<f32>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);
    }

    let light_source_position = glm::vec3(1.2, 1.0, 2.0);
    let light_source_shader =
        ShaderProgram::new(VERTEX_SHADER, LIGHT_SOURCE_FRAGMENT_SHADER).unwrap();

    let default_shader = ShaderProgram::new(VERTEX_SHADER, FRAGMENT_SHADER).unwrap();
    unsafe {
        default_shader.use_program();
        default_shader.set_uniform_vec3f("light.position", light_source_position);
        default_shader.set_uniform_vec3f("light.specular", glm::vec3(1.0, 1.0, 1.0));
        default_shader.set_uniform_vec3f("material.ambient", glm::vec3(1.0, 0.5, 0.31));
        default_shader.set_uniform_vec3f("material.diffuse", glm::vec3(1.0, 0.5, 0.31));
        default_shader.set_uniform_vec3f("material.specular", glm::vec3(0.5, 0.5, 0.5));
        default_shader.set_uniform_float("material.shininess", 32.0);
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
                    gl::BindVertexArray(vaos[0]);
                    let mut model = glm::translate(&glm::Mat4::identity(), &light_source_position);
                    model = glm::scale(&model, &glm::vec3(0.2, 0.2, 0.2));
                    for &(name, val) in
                        &[("model", model), ("view", view), ("projection", projection)]
                    {
                        light_source_shader.set_uniform_mat4f(name, val);
                    }
                    gl::DrawArrays(gl::TRIANGLES, 0, 36);

                    default_shader.use_program();
                    let light_color =
                        glm::vec3((2.0 * time).sin(), (0.7 * time).sin(), (1.3 * time).sin());
                    let diffuse_color = light_color * 0.5;
                    let ambient_color = diffuse_color * 0.2;
                    default_shader.set_uniform_vec3f("light.ambient", ambient_color);
                    default_shader.set_uniform_vec3f("light.diffuse", diffuse_color);
                    default_shader.set_uniform_vec3f("viewPos", camera.position());
                    gl::BindVertexArray(vaos[1]);
                    let model = glm::Mat4::identity();
                    for &(name, val) in
                        &[("model", model), ("view", view), ("projection", projection)]
                    {
                        default_shader.set_uniform_mat4f(name, val);
                    }

                    gl::DrawArrays(gl::TRIANGLES, 0, 36);
                }

                context.swap_buffers().unwrap();
            }
            Event::LoopDestroyed => unsafe {
                gl::DeleteVertexArrays(vaos.len() as i32, vaos.as_ptr());
                gl::DeleteBuffers(vbos.len() as i32, vbos.as_ptr());
            },
            _ => {}
        }
    });
}

#[repr(C)]
struct Vertex {
    position: glm::Vec3,
    normal: glm::Vec3,
}

impl Vertex {
    fn new(position: glm::Vec3, normal: glm::Vec3) -> Self {
        Self { position, normal }
    }
}
