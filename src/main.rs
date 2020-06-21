mod camera;
mod model;
mod shader_program;
mod texture;

use std::ffi::c_void;
use std::mem;
use std::time::Instant;

use gl::types::*;
use glutin::{Api, ContextBuilder, GlProfile, GlRequest};
use memoffset::offset_of;
use nalgebra_glm as glm;
use winit::{
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use camera::{Camera, CameraMotion};
use shader_program::ShaderProgram;

const VERTEX_SHADER: &str = include_str!("shaders/main.vert");
const FRAGMENT_SHADER: &str = include_str!("shaders/main.frag");

const MULTISAMPLING_SAMPLES: u16 = 4;

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

    let size = 0.05;
    let quad_vertices = [
        Vertex {
            position: glm::vec2(-size, size),
            color: glm::vec3(1.0, 0.0, 0.0),
        },
        Vertex {
            position: glm::vec2(-size, -size),
            color: glm::vec3(0.0, 0.0, 1.0),
        },
        Vertex {
            position: glm::vec2(size, size),
            color: glm::vec3(0.0, 1.0, 1.0),
        },
        Vertex {
            position: glm::vec2(size, -size),
            color: glm::vec3(0.0, 1.0, 0.0),
        },
    ];
    let mut offsets = [glm::vec2(0.0, 0.0); 100];
    let offset = 0.1;
    for (i, v) in offsets.iter_mut().enumerate() {
        let x = i % 10;
        let y = i / 10;
        let x = (x as f32 - 5.0) / 5.0 + offset;
        let y = (y as f32 - 5.0) / 5.0 + offset;
        *v = glm::vec2(x, y);
    }

    let mut vao = 0;
    let mut vbos = [0; 2];
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(vbos.len() as GLsizei, vbos.as_mut_ptr());

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (quad_vertices.len() * mem::size_of::<Vertex>()) as GLsizeiptr,
            quad_vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as GLsizei,
            offset_of!(Vertex, position) as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as GLsizei,
            offset_of!(Vertex, color) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[1]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (offsets.len() * mem::size_of::<glm::Vec2>()) as GLsizeiptr,
            offsets.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<glm::Vec2>() as GLsizei,
            0 as *const c_void,
        );
        gl::VertexAttribDivisor(2, 1);
        gl::EnableVertexAttribArray(2);
    }

    let main_shader = ShaderProgram::new(VERTEX_SHADER, FRAGMENT_SHADER, None).unwrap();

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

                unsafe {
                    gl::ClearColor(0.1, 0.1, 0.1, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                    main_shader.use_program();
                    gl::BindVertexArray(vao);
                    gl::DrawArraysInstanced(
                        gl::TRIANGLE_STRIP,
                        0,
                        quad_vertices.len() as GLsizei,
                        offsets.len() as GLsizei,
                    );
                }
                context.swap_buffers().unwrap();
            }
            Event::LoopDestroyed => {}
            _ => {}
        }
    });
}

#[repr(C, packed)]
struct Vertex {
    position: glm::Vec2,
    color: glm::Vec3,
}
