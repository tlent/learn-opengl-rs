mod camera;
mod model;
mod shader_program;
mod texture;

use std::ffi::c_void;
use std::mem;
use std::time::Instant;

use gl::types::*;
use glutin::{Api, ContextBuilder, GlProfile, GlRequest};
use image::GenericImageView;
use memoffset::offset_of;
use nalgebra_glm as glm;
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

const ENVMAP_VERTEX_SHADER: &str = include_str!("shaders/envmap.vert");
const ENVMAP_FRAGMENT_SHADER: &str = include_str!("shaders/envmap.frag");

const SKYBOX_VERTEX_SHADER: &str = include_str!("shaders/skybox.vert");
const SKYBOX_FRAGMENT_SHADER: &str = include_str!("shaders/skybox.frag");

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

    let cube_vertices = cube_normal_vertices();
    let skybox_vertices = skybox_vertices();

    let mut vaos = [0; 2];
    let mut vbos = [0; 2];
    unsafe {
        gl::GenVertexArrays(vaos.len() as GLsizei, vaos.as_mut_ptr());
        gl::GenBuffers(vbos.len() as GLsizei, vbos.as_mut_ptr());

        gl::BindVertexArray(vaos[0]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (cube_vertices.len() * mem::size_of::<NormalVertex>()) as GLsizeiptr,
            cube_vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<NormalVertex>() as GLsizei,
            offset_of!(NormalVertex, position) as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<NormalVertex>() as GLsizei,
            offset_of!(NormalVertex, normal) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        gl::BindVertexArray(vaos[1]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[1]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (skybox_vertices.len() * mem::size_of::<Vertex>()) as GLsizeiptr,
            skybox_vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as GLsizei,
            offset_of!(Vertex, position) as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
    }

    let envmap_shader = ShaderProgram::new(ENVMAP_VERTEX_SHADER, ENVMAP_FRAGMENT_SHADER).unwrap();
    unsafe {
        envmap_shader.use_program();
        envmap_shader.set_uniform_int("skybox", 0);
    }

    let skybox_faces: Vec<_> = ["right", "left", "top", "bottom", "front", "back"]
        .iter()
        .map(|f| format!("resources/textures/skybox/{}.jpg", f))
        .collect();
    let mut skybox_texture = 0;
    unsafe {
        gl::GenTextures(1, &mut skybox_texture);
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, skybox_texture);

        for (i, face) in skybox_faces.iter().enumerate() {
            let image = image::open(face).unwrap();
            let (width, height) = image.dimensions();
            let data = image.into_rgba().into_raw();
            let target = gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as GLenum;
            gl::TexImage2D(
                target,
                0,
                gl::RGBA8 as GLint,
                width as GLsizei,
                height as GLsizei,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const c_void,
            );
        }

        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as GLint,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as GLint,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_R,
            gl::CLAMP_TO_EDGE as GLint,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR as GLint,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR as GLint,
        );
    }
    let skybox_shader = ShaderProgram::new(SKYBOX_VERTEX_SHADER, SKYBOX_FRAGMENT_SHADER).unwrap();
    unsafe {
        skybox_shader.use_program();
        skybox_shader.set_uniform_int("skybox", 0);
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

                unsafe {
                    gl::ClearColor(0.1, 0.1, 0.1, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                    let view = camera.view_matrix();
                    let projection = glm::perspective(
                        window_size.width as f32 / window_size.height as f32,
                        camera.fov().to_radians(),
                        0.1,
                        100.0,
                    );

                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_CUBE_MAP, skybox_texture);

                    envmap_shader.use_program();
                    envmap_shader.set_uniform_mat4f("view", view);
                    envmap_shader.set_uniform_mat4f("projection", projection);
                    envmap_shader.set_uniform_vec3f("viewPos", camera.position());

                    gl::Enable(gl::CULL_FACE);
                    gl::BindVertexArray(vaos[0]);
                    let model = glm::Mat4::identity();
                    envmap_shader.set_uniform_mat4f("model", model);
                    gl::DrawArrays(gl::TRIANGLES, 0, cube_vertices.len() as GLint);
                    gl::Disable(gl::CULL_FACE);

                    let skybox_view = glm::mat3_to_mat4(&glm::mat4_to_mat3(&view));

                    skybox_shader.use_program();
                    skybox_shader.set_uniform_mat4f("view", skybox_view);
                    skybox_shader.set_uniform_mat4f("projection", projection);

                    gl::DepthFunc(gl::LEQUAL);
                    gl::BindVertexArray(vaos[1]);
                    gl::DrawArrays(gl::TRIANGLES, 0, skybox_vertices.len() as GLsizei);

                    gl::DepthFunc(gl::LESS);
                    gl::BindVertexArray(0);
                    gl::BindTexture(gl::TEXTURE_2D, 0);
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
    position: glm::Vec3,
}

fn skybox_vertices() -> Vec<Vertex> {
    vec![
        Vertex {
            position: glm::vec3(-1.0, 1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, -1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(1.0, -1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(1.0, -1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(1.0, 1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, 1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, -1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, -1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, 1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, 1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, 1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, -1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(1.0, -1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(1.0, -1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(1.0, 1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(1.0, 1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(1.0, 1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(1.0, -1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, -1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, 1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(1.0, 1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(1.0, 1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(1.0, -1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, -1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, 1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(1.0, 1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(1.0, 1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(1.0, 1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, 1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, 1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, -1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, -1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(1.0, -1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(1.0, -1.0, -1.0),
        },
        Vertex {
            position: glm::vec3(-1.0, -1.0, 1.0),
        },
        Vertex {
            position: glm::vec3(1.0, -1.0, 1.0),
        },
    ]
}

#[repr(C, packed)]
struct NormalVertex {
    position: glm::Vec3,
    normal: glm::Vec3,
}

fn cube_normal_vertices() -> Vec<NormalVertex> {
    vec![
        // back
        NormalVertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            normal: glm::vec3(0.0, 0.0, -1.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, 0.5, -0.5),
            normal: glm::vec3(0.0, 0.0, -1.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, -0.5, -0.5),
            normal: glm::vec3(0.0, 0.0, -1.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, 0.5, -0.5),
            normal: glm::vec3(0.0, 0.0, -1.0),
        },
        NormalVertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            normal: glm::vec3(0.0, 0.0, -1.0),
        },
        NormalVertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            normal: glm::vec3(0.0, 0.0, -1.0),
        },
        // front
        NormalVertex {
            position: glm::vec3(-0.5, -0.5, 0.5),
            normal: glm::vec3(0.0, 0.0, 1.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, -0.5, 0.5),
            normal: glm::vec3(0.0, 0.0, 1.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            normal: glm::vec3(0.0, 0.0, 1.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            normal: glm::vec3(0.0, 0.0, 1.0),
        },
        NormalVertex {
            position: glm::vec3(-0.5, 0.5, 0.5),
            normal: glm::vec3(0.0, 0.0, 1.0),
        },
        NormalVertex {
            position: glm::vec3(-0.5, -0.5, 0.5),
            normal: glm::vec3(0.0, 0.0, 1.0),
        },
        // left
        NormalVertex {
            position: glm::vec3(-0.5, 0.5, 0.5),
            normal: glm::vec3(-1.0, 0.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            normal: glm::vec3(-1.0, 0.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            normal: glm::vec3(-1.0, 0.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            normal: glm::vec3(-1.0, 0.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(-0.5, -0.5, 0.5),
            normal: glm::vec3(-1.0, 0.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(-0.5, 0.5, 0.5),
            normal: glm::vec3(-1.0, 0.0, 0.0),
        },
        // right
        NormalVertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            normal: glm::vec3(1.0, 0.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, -0.5, -0.5),
            normal: glm::vec3(1.0, 0.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, 0.5, -0.5),
            normal: glm::vec3(1.0, 0.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, -0.5, -0.5),
            normal: glm::vec3(1.0, 0.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            normal: glm::vec3(1.0, 0.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, -0.5, 0.5),
            normal: glm::vec3(1.0, 0.0, 0.0),
        },
        // bottom
        NormalVertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            normal: glm::vec3(0.0, -1.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, -0.5, -0.5),
            normal: glm::vec3(0.0, -1.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, -0.5, 0.5),
            normal: glm::vec3(0.0, -1.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, -0.5, 0.5),
            normal: glm::vec3(0.0, -1.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(-0.5, -0.5, 0.5),
            normal: glm::vec3(0.0, -1.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            normal: glm::vec3(0.0, -1.0, 0.0),
        },
        // top
        NormalVertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            normal: glm::vec3(0.0, 1.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            normal: glm::vec3(0.0, 1.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, 0.5, -0.5),
            normal: glm::vec3(0.0, 1.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            normal: glm::vec3(0.0, 1.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            normal: glm::vec3(0.0, 1.0, 0.0),
        },
        NormalVertex {
            position: glm::vec3(-0.5, 0.5, 0.5),
            normal: glm::vec3(0.0, 1.0, 0.0),
        },
    ]
}

#[repr(C, packed)]
struct TexVertex {
    position: glm::Vec3,
    tex_coord: glm::Vec2,
}

fn cube_vertices() -> Vec<TexVertex> {
    vec![
        // back
        TexVertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(0.5, 0.5, -0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(0.5, -0.5, -0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(0.5, 0.5, -0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        // front
        TexVertex {
            position: glm::vec3(-0.5, -0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(0.5, -0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(-0.5, 0.5, 0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(-0.5, -0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        // left
        TexVertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(-0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(-0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(-0.5, -0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        // right
        TexVertex {
            position: glm::vec3(0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(0.5, 0.5, -0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(0.5, -0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        // bottom
        TexVertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(0.5, -0.5, -0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(0.5, -0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(0.5, -0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(-0.5, -0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        // top
        TexVertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(0.5, 0.5, -0.5),
            tex_coord: glm::vec2(1.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(-0.5, 0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
    ]
}

fn plane_vertices() -> Vec<TexVertex> {
    vec![
        TexVertex {
            position: glm::vec3(5.0, -0.5, 5.0),
            tex_coord: glm::vec2(2.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(-5.0, -0.5, 5.0),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(-5.0, -0.5, -5.0),
            tex_coord: glm::vec2(0.0, 2.0),
        },
        TexVertex {
            position: glm::vec3(5.0, -0.5, 5.0),
            tex_coord: glm::vec2(2.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(-5.0, -0.5, -5.0),
            tex_coord: glm::vec2(0.0, 2.0),
        },
        TexVertex {
            position: glm::vec3(5.0, -0.5, -5.0),
            tex_coord: glm::vec2(2.0, 2.0),
        },
    ]
}

fn quad_vertices(x_size: f32, y_size: f32) -> Vec<TexVertex> {
    vec![
        TexVertex {
            position: glm::vec3(-x_size, -y_size, 0.0),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(x_size, -y_size, 0.0),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        TexVertex {
            position: glm::vec3(-x_size, y_size, 0.0),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        TexVertex {
            position: glm::vec3(x_size, y_size, 0.0),
            tex_coord: glm::vec2(1.0, 1.0),
        },
    ]
}
