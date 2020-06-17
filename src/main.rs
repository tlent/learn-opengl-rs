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
use ordered_float::NotNan;
use winit::{
    event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

use camera::{Camera, CameraMotion};
use framebuffer::Framebuffer;
use shader_program::ShaderProgram;
use texture::Texture;

const VERTEX_SHADER: &str = include_str!("shaders/basic.vert");
const FRAGMENT_SHADER: &str = include_str!("shaders/basic.frag");

const SCREEN_VERTEX_SHADER: &str = include_str!("shaders/screen.vert");
const SCREEN_FRAGMENT_SHADER: &str = include_str!("shaders/screen.frag");

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
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        context.swap_buffers().unwrap();
    }

    let cube_vertices = cube_vertices();
    let plane_vertices = plane_vertices();
    let grass_quad_vertices = quad_vertices(0.5, 0.5);
    let screen_quad_vertices = quad_vertices(1.0, 1.0);

    let mut vaos = [0; 4];
    let mut vbos = [0; 4];
    unsafe {
        gl::GenVertexArrays(vaos.len() as GLint, vaos.as_mut_ptr());
        gl::GenBuffers(vbos.len() as GLint, vbos.as_mut_ptr());

        gl::BindVertexArray(vaos[0]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (cube_vertices.len() * mem::size_of::<Vertex>()) as GLsizeiptr,
            cube_vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as GLint,
            offset_of!(Vertex, position) as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as GLint,
            offset_of!(Vertex, tex_coord) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        gl::BindVertexArray(vaos[1]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[1]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (plane_vertices.len() * mem::size_of::<Vertex>()) as GLsizeiptr,
            plane_vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as GLint,
            offset_of!(Vertex, position) as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as GLint,
            offset_of!(Vertex, tex_coord) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        gl::BindVertexArray(vaos[2]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[2]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (grass_quad_vertices.len() * mem::size_of::<Vertex>()) as GLsizeiptr,
            grass_quad_vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as GLint,
            offset_of!(Vertex, position) as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as GLint,
            offset_of!(Vertex, tex_coord) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        gl::BindVertexArray(vaos[3]);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[3]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (screen_quad_vertices.len() * mem::size_of::<Vertex>()) as GLsizeiptr,
            screen_quad_vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as GLint,
            offset_of!(Vertex, position) as *const c_void,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as GLint,
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

    let screen_shader = ShaderProgram::new(SCREEN_VERTEX_SHADER, SCREEN_FRAGMENT_SHADER).unwrap();
    unsafe {
        screen_shader.use_program();
        screen_shader.set_uniform_int("tex", 0);
    }

    let cube_texture = unsafe { Texture::load("resources/textures/container.jpg").unwrap() };
    let plane_texture = unsafe { Texture::load("resources/textures/metal.png").unwrap() };
    let window_texture = unsafe { Texture::load("resources/textures/window.png").unwrap() };
    unsafe {
        window_texture.set_wrap(gl::CLAMP_TO_EDGE, gl::CLAMP_TO_EDGE);
    }

    let framebuffer = Framebuffer::new(window_size);

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
                    framebuffer.resize(size);
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
                    framebuffer.bind();
                    gl::ClearColor(0.1, 0.1, 0.1, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    gl::Enable(gl::DEPTH_TEST);

                    let view = camera.view_matrix();
                    let projection = glm::perspective(
                        window_size.width as f32 / window_size.height as f32,
                        camera.fov().to_radians(),
                        0.1,
                        100.0,
                    );
                    basic_shader.use_program();
                    basic_shader.set_uniform_mat4f("view", view);
                    basic_shader.set_uniform_mat4f("projection", projection);

                    gl::ActiveTexture(gl::TEXTURE0);

                    gl::Enable(gl::CULL_FACE);
                    gl::BindVertexArray(vaos[0]);
                    gl::BindTexture(gl::TEXTURE_2D, cube_texture.id());
                    let cube_positions = [glm::vec3(-1.0, 0.0, -1.0), glm::vec3(2.0, 0.0, 0.0)];
                    for p in cube_positions.iter() {
                        let model = glm::translate(&glm::Mat4::identity(), p);
                        basic_shader.set_uniform_mat4f("model", model);
                        gl::DrawArrays(gl::TRIANGLES, 0, cube_vertices.len() as GLint);
                    }
                    gl::Disable(gl::CULL_FACE);

                    gl::BindVertexArray(vaos[1]);
                    gl::BindTexture(gl::TEXTURE_2D, plane_texture.id());
                    basic_shader.set_uniform_mat4f("model", glm::Mat4::identity());
                    gl::DrawArrays(gl::TRIANGLES, 0, plane_vertices.len() as GLint);

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
                        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, grass_quad_vertices.len() as GLint);
                    }

                    gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                    gl::ClearColor(1.0, 1.0, 1.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    gl::Disable(gl::DEPTH_TEST);

                    screen_shader.use_program();
                    gl::BindVertexArray(vaos[3]);
                    gl::BindTexture(gl::TEXTURE_2D, framebuffer.texture());
                    gl::DrawArrays(gl::TRIANGLE_STRIP, 0, screen_quad_vertices.len() as GLint);

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

mod framebuffer {
    use std::ptr;

    use gl::types::*;
    use winit::dpi::PhysicalSize;

    pub struct Framebuffer {
        id: GLuint,
        texture: GLuint,
        render_buffer: GLuint,
    }

    impl Framebuffer {
        pub fn new(size: PhysicalSize<u32>) -> Self {
            let mut s = Self {
                id: 0,
                texture: 0,
                render_buffer: 0,
            };
            unsafe {
                gl::GenFramebuffers(1, &mut s.id);
                gl::GenTextures(1, &mut s.texture);
                gl::GenRenderbuffers(1, &mut s.render_buffer);

                gl::BindFramebuffer(gl::FRAMEBUFFER, s.id);
                gl::BindTexture(gl::TEXTURE_2D, s.texture);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGB8 as GLint,
                    size.width as GLint,
                    size.height as GLint,
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    ptr::null(),
                );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                gl::BindTexture(gl::TEXTURE_2D, 0);
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    s.texture,
                    0,
                );

                gl::BindRenderbuffer(gl::RENDERBUFFER, s.render_buffer);
                gl::RenderbufferStorage(
                    gl::RENDERBUFFER,
                    gl::DEPTH24_STENCIL8,
                    size.width as GLint,
                    size.height as GLint,
                );
                gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
                gl::FramebufferRenderbuffer(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_STENCIL_ATTACHMENT,
                    gl::RENDERBUFFER,
                    s.render_buffer,
                );

                if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                    panic!("Framebuffer initialization failed");
                }
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            }

            s
        }

        pub fn bind(&self) {
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
            }
        }

        pub fn resize(&self, size: PhysicalSize<u32>) {
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, self.texture);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGB8 as GLint,
                    size.width as GLint,
                    size.height as GLint,
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    ptr::null(),
                );
                gl::BindTexture(gl::TEXTURE_2D, 0);

                gl::BindRenderbuffer(gl::RENDERBUFFER, self.render_buffer);
                gl::RenderbufferStorage(
                    gl::RENDERBUFFER,
                    gl::DEPTH24_STENCIL8,
                    size.width as GLint,
                    size.height as GLint,
                );
                gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
            }
        }

        pub fn texture(&self) -> GLuint {
            self.texture
        }
    }

    impl Drop for Framebuffer {
        fn drop(&mut self) {
            unsafe {
                gl::DeleteTextures(1, &self.texture);
                gl::DeleteRenderbuffers(1, &self.render_buffer);
                gl::DeleteFramebuffers(1, &self.id);
            }
        }
    }
}

#[repr(C, packed)]
struct Vertex {
    position: glm::Vec3,
    tex_coord: glm::Vec2,
}

fn cube_vertices() -> Vec<Vertex> {
    vec![
        // back
        Vertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(0.5, 0.5, -0.5),
            tex_coord: glm::vec2(1.0, 1.0),
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
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        // front
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
        // left
        Vertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
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
            position: glm::vec3(-0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-0.5, -0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        // right
        Vertex {
            position: glm::vec3(0.5, -0.5, -0.5),
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
            position: glm::vec3(0.5, -0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(0.5, -0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        // bottom
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
        // top
        Vertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
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
            position: glm::vec3(0.5, 0.5, 0.5),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-0.5, 0.5, -0.5),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        Vertex {
            position: glm::vec3(-0.5, 0.5, 0.5),
            tex_coord: glm::vec2(0.0, 0.0),
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

fn quad_vertices(x_size: f32, y_size: f32) -> Vec<Vertex> {
    vec![
        Vertex {
            position: glm::vec3(-x_size, -y_size, 0.0),
            tex_coord: glm::vec2(0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(x_size, -y_size, 0.0),
            tex_coord: glm::vec2(1.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-x_size, y_size, 0.0),
            tex_coord: glm::vec2(0.0, 1.0),
        },
        Vertex {
            position: glm::vec3(x_size, y_size, 0.0),
            tex_coord: glm::vec2(1.0, 1.0),
        },
    ]
}
