use std::ffi::{c_void, CString};
use std::mem;
use std::ptr;
use std::str;

use anyhow::{anyhow, Result};
use gl::types::*;
use glutin::{Api, ContextBuilder, GlProfile, GlRequest};
use image::GenericImageView;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

const VERTEX_SHADER: &str = include_str!("default.vert");
const FRAGMENT_SHADER: &str = include_str!("default.frag");

fn main() {
    let event_loop = EventLoop::new();
    let monitor = event_loop.primary_monitor();
    let PhysicalSize { width, height } = monitor.size();
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
    unsafe {
        gl::Viewport(0, 0, width as i32, height as i32);
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
    context.window().set_visible(true);

    let vertices: [GLfloat; 32] = [
        -0.5, 0.5, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, -0.5,
        -0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0,
    ];
    let indices: [GLint; 6] = [0, 1, 2, 1, 2, 3];

    let mut vaos = [0; 1];
    let mut vbos = [0; 2];
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
            8 * mem::size_of::<GLfloat>() as i32,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            8 * mem::size_of::<GLfloat>() as i32,
            (3 * mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            8 * mem::size_of::<GLfloat>() as i32,
            (6 * mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(2);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbos[1]);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * mem::size_of::<GLint>()) as isize,
            indices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
    }

    let shader_program = ShaderProgram::new(VERTEX_SHADER, FRAGMENT_SHADER).unwrap();

    let img = image::open("container.jpg").unwrap();
    let (width, height) = img.dimensions();
    let data = img.flipv().into_rgb().into_raw();

    let mut textures = [0; 2];
    unsafe {
        gl::GenTextures(2, textures.as_mut_ptr());
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, textures[0]);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
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
                event: WindowEvent::Resized(PhysicalSize { width, height }),
                ..
            } => unsafe {
                gl::Viewport(0, 0, width as i32, height as i32);
            },
            Event::MainEventsCleared => {
                unsafe {
                    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    shader_program.use_program();
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, texture);
                    gl::BindVertexArray(vaos[0]);
                    gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
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
