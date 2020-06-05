use std::ffi::CString;
use std::ptr;
use std::str;

use anyhow::{anyhow, Result};
use gl::types::*;

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
