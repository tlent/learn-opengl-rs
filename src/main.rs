mod camera;
mod model;
mod shader_program;
mod texture;

use std::ffi::c_void;
use std::mem;
use std::time::Instant;

use glutin::{Api, ContextBuilder, GlProfile, GlRequest};
use memoffset::offset_of;
use nalgebra_glm as glm;
use ordered_float::NotNan;
use winit::{
    event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

use camera::{Camera, CameraMotion};
use shader_program::ShaderProgram;
use texture::Texture;

const VERTEX_SHADER: &str = include_str!("shaders/basic.vert");
const FRAGMENT_SHADER: &str = include_str!("shaders/basic.frag");

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
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        context.swap_buffers().unwrap();
    }

    let cube_vertices = cube_vertices();
    let plane_vertices = plane_vertices();
    let quad_vertices = quad_vertices();

    let mut vaos = [0; 3];
    let mut vbos = [0; 3];
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
            offset_of!(Vertex, position) as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as i32,
            offset_of!(Vertex, tex_coord) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        gl::BindVertexArray(vaos[1]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[1]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (plane_vertices.len() * mem::size_of::<Vertex>()) as isize,
            plane_vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as i32,
            offset_of!(Vertex, position) as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as i32,
            offset_of!(Vertex, tex_coord) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        gl::BindVertexArray(vaos[2]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[2]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (quad_vertices.len() * mem::size_of::<Vertex>()) as isize,
            quad_vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as i32,
            offset_of!(Vertex, position) as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as i32,
            offset_of!(Vertex, tex_coord) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        gl::BindVertexArray(0);
    }

    let basic_shader = ShaderProgram::new(VERTEX_SHADER, FRAGMENT_SHADER).unwrap();
    unsafe {
        basic_shader.use_program();
        basic_shader.set_uniform_int("tex", 0);
    }
    let cube_texture = unsafe { Texture::load("resources/textures/marble.jpg").unwrap() };
    let plane_texture = unsafe { Texture::load("resources/textures/metal.png").unwrap() };
    let window_texture = unsafe { Texture::load("resources/textures/window.png").unwrap() };
    unsafe {
        window_texture.set_wrap(gl::CLAMP_TO_EDGE, gl::CLAMP_TO_EDGE);
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
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);

                    let view = camera.view_matrix();
                    let projection = glm::perspective(
                        aspect_ratio as f32,
                        camera.fov().to_radians(),
                        0.1,
                        100.0,
                    );
                    basic_shader.use_program();
                    basic_shader.set_uniform_mat4f("view", view);
                    basic_shader.set_uniform_mat4f("projection", projection);

                    gl::ActiveTexture(gl::TEXTURE0);

                    gl::BindVertexArray(vaos[0]);
                    gl::BindTexture(gl::TEXTURE_2D, cube_texture.id());
                    let cube_positions = [glm::vec3(-1.0, 0.0, -1.0), glm::vec3(2.0, 0.0, 0.0)];
                    for p in cube_positions.iter() {
                        let model = glm::translate(&glm::Mat4::identity(), p);
                        basic_shader.set_uniform_mat4f("model", model);
                        gl::DrawArrays(gl::TRIANGLES, 0, cube_vertices.len() as i32);
                    }

                    gl::BindVertexArray(vaos[1]);
                    gl::BindTexture(gl::TEXTURE_2D, plane_texture.id());
                    basic_shader.set_uniform_mat4f("model", glm::Mat4::identity());
                    gl::DrawArrays(gl::TRIANGLES, 0, plane_vertices.len() as i32);

                    gl::BindVertexArray(vaos[2]);
                    gl::BindTexture(gl::TEXTURE_2D, window_texture.id());
                    let mut positions = [
                        glm::vec3(-1.5, 0.0, -0.48),
                        glm::vec3(1.5, 0.0, 0.51),
                        glm::vec3(0.0, 0.0, 0.7),
                        glm::vec3(-0.3, 0.0, -2.3),
                        glm::vec3(0.5, 0.0, -0.6),
                    ];
                    positions.sort_by_key(|p| {
                        let distance = glm::length(&(p - camera.position()));
                        NotNan::new(distance).unwrap()
                    });
                    for p in positions.iter().rev() {
                        let model = glm::translate(&glm::Mat4::identity(), p);
                        basic_shader.set_uniform_mat4f("model", model);
                        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, quad_vertices.len() as i32);
                    }

                    gl::BindVertexArray(0);
                    gl::BindTexture(gl::TEXTURE_2D, 0);
                }
                context.swap_buffers().unwrap();
            }
            _ => {}
        }
    });
}

#[repr(C, packed)]
struct Vertex {
    position: glm::Vec3,
    tex_coord: glm::Vec2,
}

fn cube_vertices() -> Vec<Vertex> {
    vec![
        Vertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(0.5, -0.5, -0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        Vertex {
            position: glm::vec3(0.5, 0.5, -0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(0.5, 0.5, -0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-0.5, -0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(0.5, -0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        Vertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-0.5, 0.5, 0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-0.5, -0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-0.5, -0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        Vertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        Vertex {
            position: glm::vec3(0.5, 0.5, -0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        Vertex {
            position: glm::vec3(0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        Vertex {
            position: glm::vec3(0.5, -0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        Vertex {
            position: glm::vec3(0.5, -0.5, -0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(0.5, -0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        Vertex {
            position: glm::vec3(0.5, -0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-0.5, -0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        Vertex {
            position: glm::vec3(0.5, 0.5, -0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        Vertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-0.5, 0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
    ]
}

fn plane_vertices() -> Vec<Vertex> {
    vec![
        Vertex {
            position: glm::vec3(5.0, -0.5, 5.0),
            tex_coord: glm::vec2(2.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-5.0, -0.5, 5.0),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-5.0, -0.5, -5.0),
            tex_coord: glm::vec2(0.0, 2.0),
        },
        Vertex {
            position: glm::vec3(5.0, -0.5, 5.0),
            tex_coord: glm::vec2(2.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-5.0, -0.5, -5.0),
            tex_coord: glm::vec2(0.0, 2.0),
        },
        Vertex {
            position: glm::vec3(5.0, -0.5, -5.0),
            tex_coord: glm::vec2(2.0, 2.0),
        },
    ]
}

fn quad_vertices() -> Vec<Vertex> {
    vec![
        Vertex {
            position: glm::vec3(-0.5, -0.5, 0.0),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(0.5, -0.5, 0.0),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-0.5, 0.5, 0.0),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        Vertex {
            position: glm::vec3(0.5, 0.5, 0.0),
            tex_coord: glm::vec2(1.0, 1.0),
        },
    ]
}
