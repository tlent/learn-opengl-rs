use std::ffi::{c_void, CString};
use std::mem;
use std::ptr;
use std::str;
use std::time::Instant;

use anyhow::{anyhow, Result};
use gl::types::*;
use glutin::{Api, ContextBuilder, GlProfile, GlRequest};
use image::GenericImageView;
use nalgebra_glm as glm;
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

const VERTEX_SHADER: &str = include_str!("default.vert");
const FRAGMENT_SHADER: &str = include_str!("default.frag");

fn main() {
    let event_loop = EventLoop::new();
    let monitor = event_loop.primary_monitor();
    let monitor_size = monitor.size();
    let window_builder = WindowBuilder::new()
        .with_visible(false)
        .with_title("Learn OpenGL")
        .with_fullscreen(Some(Fullscreen::Borderless(monitor)));
    let context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core)
        .with_vsync(true)
        .build_windowed(window_builder, &event_loop)
        .unwrap();
    let context = unsafe { context.make_current().unwrap() };
    gl::load_with(|s| context.get_proc_address(s));

    let (width, height) = (800, 600);
    let mut x_offset = (monitor_size.width as i32 / 2) - (width / 2);
    if x_offset < 0 {
        x_offset = 0;
    }
    let mut y_offset = (monitor_size.height as i32 / 2) - (height / 2);
    if y_offset < 0 {
        y_offset = 0;
    }
    unsafe {
        gl::Viewport(x_offset, y_offset, width, height);
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::Enable(gl::DEPTH_TEST);
    }
    let start = Instant::now();
    context.window().set_visible(true);

    let vertices: [GLfloat; 36 * 5] = [
        -0.5, -0.5, -0.5, 0.0, 0.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.5, 0.5, -0.5, 1.0, 1.0, 0.5, 0.5,
        -0.5, 1.0, 1.0, -0.5, 0.5, -0.5, 0.0, 1.0, -0.5, -0.5, -0.5, 0.0, 0.0, -0.5, -0.5, 0.5,
        0.0, 0.0, 0.5, -0.5, 0.5, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 1.0, -0.5,
        0.5, 0.5, 0.0, 1.0, -0.5, -0.5, 0.5, 0.0, 0.0, -0.5, 0.5, 0.5, 1.0, 0.0, -0.5, 0.5, -0.5,
        1.0, 1.0, -0.5, -0.5, -0.5, 0.0, 1.0, -0.5, -0.5, -0.5, 0.0, 1.0, -0.5, -0.5, 0.5, 0.0,
        0.0, -0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, -0.5, 1.0, 1.0, 0.5,
        -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, 0.5, 0.0, 0.0, 0.5, 0.5, 0.5,
        1.0, 0.0, -0.5, -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, -0.5, 1.0, 1.0, 0.5, -0.5, 0.5, 1.0, 0.0,
        0.5, -0.5, 0.5, 1.0, 0.0, -0.5, -0.5, 0.5, 0.0, 0.0, -0.5, -0.5, -0.5, 0.0, 1.0, -0.5, 0.5,
        -0.5, 0.0, 1.0, 0.5, 0.5, -0.5, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0,
        -0.5, 0.5, 0.5, 0.0, 0.0, -0.5, 0.5, -0.5, 0.0, 1.0,
    ];

    let mut vaos = [0; 1];
    let mut vbos = [0; 1];
    unsafe {
        gl::GenVertexArrays(vaos.len() as i32, vaos.as_mut_ptr());
        gl::BindVertexArray(vaos[0]);
        gl::GenBuffers(vbos.len() as i32, vbos.as_mut_ptr());
        gl::BindBuffer(gl::ARRAY_BUFFER, vbos[0]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<GLfloat>()) as isize,
            vertices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            5 * mem::size_of::<GLfloat>() as i32,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            5 * mem::size_of::<GLfloat>() as i32,
            (3 * mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);
    }

    let shader_program = ShaderProgram::new(VERTEX_SHADER, FRAGMENT_SHADER).unwrap();

    let mut textures = [0; 2];
    unsafe {
        gl::GenTextures(2, textures.as_mut_ptr());
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, textures[0]);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        let img = image::open("container.jpg").unwrap();
        let (width, height) = img.dimensions();
        let data = img.flipv().into_rgb().into_raw();
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            width as i32,
            height as i32,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const c_void,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, textures[1]);

        let img = image::open("awesomeface.png").unwrap();
        let (width, height) = img.dimensions();
        let data = img.flipv().into_rgba().into_raw();
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            width as i32,
            height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const c_void,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        shader_program.use_program();
        shader_program.set_uniform_int("textureSampler", 0);
        shader_program.set_uniform_int("textureSampler2", 1);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, textures[0]);
        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, textures[1]);
    }

    let cube_positions = [
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(2.0, 5.0, -15.0),
        glm::vec3(-1.5, -2.2, -2.5),
        glm::vec3(-3.8, -2.0, -12.3),
        glm::vec3(2.4, -0.4, -3.5),
        glm::vec3(-1.7, 3.0, -7.5),
        glm::vec3(1.3, -2.0, -2.5),
        glm::vec3(1.5, 2.0, -2.5),
        glm::vec3(1.5, 0.2, -1.5),
        glm::vec3(-1.3, 1.0, -1.5),
    ];

    let mut model = glm::Mat4::identity();
    let view = glm::translate(&glm::Mat4::identity(), &glm::vec3(0.0, 0.0, -3.0));
    let projection = glm::perspective(4.0 / 3.0, 45.0f32.to_radians(), 0.1, 100.0);

    unsafe {
        for &(name, val) in &[("view", view), ("projection", projection)] {
            let loc =
                gl::GetUniformLocation(shader_program.id(), CString::new(name).unwrap().as_ptr());
            gl::UniformMatrix4fv(loc, 1, gl::FALSE, glm::value_ptr(&val).as_ptr());
        }
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if input.state == ElementState::Pressed {
                    match input.virtual_keycode {
                        Some(VirtualKeyCode::Escape) => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(window_size),
                ..
            } => unsafe {
                let mut x_offset = (window_size.width as i32 / 2) - (width / 2);
                if x_offset < 0 {
                    x_offset = 0;
                }
                let mut y_offset = (window_size.height as i32 / 2) - (height / 2);
                if y_offset < 0 {
                    y_offset = 0;
                }
                gl::Viewport(x_offset, y_offset, width, height);
            },
            Event::MainEventsCleared => {
                let time = (Instant::now() - start).as_secs_f32();
                unsafe {
                    gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    shader_program.use_program();
                    gl::BindVertexArray(vaos[0]);

                    for (i, p) in cube_positions.iter().enumerate() {
                        model = glm::translate(&glm::Mat4::identity(), p);
                        let angle = if i % 3 == 0 {
                            time
                        } else {
                            (20.0 * i as f32).to_radians()
                        };
                        model = glm::rotate(&model, angle, &glm::vec3(1.0, 0.3, 0.5));
                        let loc = gl::GetUniformLocation(
                            shader_program.id(),
                            CString::new("model").unwrap().as_ptr(),
                        );
                        gl::UniformMatrix4fv(loc, 1, gl::FALSE, glm::value_ptr(&model).as_ptr());

                        gl::DrawArrays(gl::TRIANGLES, 0, 36);
                    }
                }

                context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}

pub struct ShaderProgram {
    id: u32,
}

impl ShaderProgram {
    pub fn new(vertex_shader: &str, fragment_shader: &str) -> Result<Self> {
        let mut shaders = vec![];
        for &(s, t) in &[
            (vertex_shader, gl::VERTEX_SHADER),
            (fragment_shader, gl::FRAGMENT_SHADER),
        ] {
            let id = unsafe { gl::CreateShader(t) };
            let source = CString::new(s).unwrap();
            unsafe {
                gl::ShaderSource(id, 1, &source.as_ptr(), ptr::null());
                gl::CompileShader(id);
            }

            let mut success = gl::FALSE as GLint;
            unsafe {
                gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
            }
            if success != gl::TRUE as GLint {
                for shader in shaders {
                    unsafe { gl::DeleteShader(shader) };
                }
                let mut message = vec![0; 512];
                unsafe {
                    gl::GetShaderInfoLog(
                        id,
                        512,
                        ptr::null_mut(),
                        message.as_mut_ptr() as *mut GLchar,
                    );
                    message.set_len(message.iter().position(|&v| v == 0).unwrap_or(512));
                }
                return Err(anyhow!(
                    "ERROR::SHADER::COMPILATION_FAILED:\n{}",
                    str::from_utf8(&message).unwrap()
                ));
            }
            shaders.push(id);
        }

        let id = unsafe { gl::CreateProgram() };
        for &shader in shaders.iter() {
            unsafe { gl::AttachShader(id, shader) };
        }
        unsafe {
            gl::LinkProgram(id);
        }
        for shader in shaders {
            unsafe { gl::DeleteShader(shader) };
        }
        let mut success = gl::FALSE as GLint;
        unsafe {
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
        }
        if success != gl::TRUE as GLint {
            let mut message = vec![0; 512];
            unsafe {
                gl::GetProgramInfoLog(
                    id,
                    512,
                    ptr::null_mut(),
                    message.as_mut_ptr() as *mut GLchar,
                );
                message.set_len(message.iter().position(|&v| v == 0).unwrap_or(512));
            }
            return Err(anyhow!(
                "ERROR::SHADER::PROGRAM::LINKING_FAILED:\n{}",
                str::from_utf8(&message).unwrap()
            ));
        }
        Ok(Self { id })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub unsafe fn use_program(&self) {
        gl::UseProgram(self.id);
    }

    pub unsafe fn set_uniform_bool(&self, name: &str, value: bool) {
        let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
        gl::Uniform1i(location, value as i32);
    }

    pub unsafe fn set_uniform_int(&self, name: &str, value: i32) {
        let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
        gl::Uniform1i(location, value);
    }

    pub unsafe fn set_uniform_float(&self, name: &str, value: f32) {
        let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
        gl::Uniform1f(location, value);
    }
}
