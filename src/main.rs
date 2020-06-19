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
    event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

use camera::{Camera, CameraMotion};
use shader_program::ShaderProgram;

const VERTEX_SHADER: &str = include_str!("shaders/basic.vert");
const FRAGMENT_SHADER: &str = include_str!("shaders/basic.frag");
const GEOMETRY_SHADER: &str = include_str!("shaders/basic.geom");

const MULTISAMPLING_SAMPLES: u16 = 4;

fn main() {
    let event_loop = EventLoop::new();
    let monitor = event_loop.primary_monitor();
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
    let mut window_size = context.window().inner_size();

    gl::load_with(|s| context.get_proc_address(s));
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::MULTISAMPLE);
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        context.swap_buffers().unwrap();
    }

    let points = [
        Vertex {
            position: glm::vec2(0.5, 0.5),
            color: glm::vec3(1.0, 0.0, 0.0),
        },
        Vertex {
            position: glm::vec2(0.5, -0.5),
            color: glm::vec3(0.0, 1.0, 0.0),
        },
        Vertex {
            position: glm::vec2(-0.5, -0.5),
            color: glm::vec3(0.0, 0.0, 1.0),
        },
        Vertex {
            position: glm::vec2(-0.5, 0.5),
            color: glm::vec3(1.0, 1.0, 0.0),
        },
    ];

    let mut vaos = [0; 1];
    let mut vbos = [0; 1];
    unsafe {
        gl::GenVertexArrays(vaos.len() as GLsizei, vaos.as_mut_ptr());
        gl::GenBuffers(vbos.len() as GLsizei, vbos.as_mut_ptr());

        gl::BindVertexArray(vaos[0]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (points.len() * mem::size_of::<Vertex>()) as GLsizeiptr,
            points.as_ptr() as *const c_void,
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
    }

    let shader_program =
        ShaderProgram::new(VERTEX_SHADER, FRAGMENT_SHADER, Some(GEOMETRY_SHADER)).unwrap();

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

                    shader_program.use_program();
                    gl::BindVertexArray(vaos[0]);
                    gl::DrawArrays(gl::POINTS, 0, points.len() as GLsizei);

                    gl::BindVertexArray(0);
                }
                context.swap_buffers().unwrap();
            }
            Event::LoopDestroyed => unsafe {
                gl::DeleteVertexArrays(vaos.len() as GLint, vaos.as_ptr());
                gl::DeleteBuffers(vbos.len() as GLint, vbos.as_ptr());
            },
            _ => {}
        }
    });
}

#[repr(C, packed)]
struct Vertex {
    position: glm::Vec2,
    color: glm::Vec3,
}
