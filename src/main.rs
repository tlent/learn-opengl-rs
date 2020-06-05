use std::ffi::{c_void, CString};
use std::mem;
use std::ptr;
use std::time::Instant;

use gl::types::*;
use glutin::{Api, ContextBuilder, GlProfile, GlRequest};
use image::GenericImageView;
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

const VERTEX_SHADER: &str = include_str!("default.vert");
const FRAGMENT_SHADER: &str = include_str!("default.frag");

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
        .build_windowed(window_builder, &event_loop)
        .unwrap();
    let context = unsafe { context.make_current().unwrap() };
    gl::load_with(|s| context.get_proc_address(s));

    let (width, height) = (1920, 1080);
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
    context.window().set_cursor_grab(true).unwrap();
    context.window().set_cursor_visible(false);

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

    let mut last_frame = Instant::now();
    let mut delta_time = 0.0f32;
    let mut pressed_keys = Vec::with_capacity(10);
    let mut mouse_delta = (0.0, 0.0);
    let mut scroll_delta = 0.0;

    let mut camera = Camera::new(
        glm::vec3(0.0, 0.0, 3.0),
        glm::vec3(0.0, 1.0, 0.0),
        -90.0,
        0.0,
    );
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
            } => match input.state {
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
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta: (dx, dy) },
                ..
            } => {
                let (x, y) = mouse_delta;
                mouse_delta = (x + dx as f32, y - dy as f32);
            }
            Event::DeviceEvent {
                event:
                    DeviceEvent::MouseWheel {
                        delta: MouseScrollDelta::LineDelta(_, dy),
                    },
                ..
            } => {
                scroll_delta += dy / 15.0;
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
                let now = Instant::now();
                let time = (now - start).as_secs_f32();
                delta_time = (now - last_frame).as_secs_f32();
                last_frame = now;

                for key in pressed_keys.iter() {
                    match key {
                        VirtualKeyCode::W => camera.move_(CameraMotion::Forward, delta_time),
                        VirtualKeyCode::S => camera.move_(CameraMotion::Backward, delta_time),
                        VirtualKeyCode::A => camera.move_(CameraMotion::Left, delta_time),
                        VirtualKeyCode::D => camera.move_(CameraMotion::Right, delta_time),
                        _ => {}
                    }
                }
                camera.look(mouse_delta);
                mouse_delta = (0.0, 0.0);
                camera.zoom(scroll_delta);
                scroll_delta = 0.0;

                let projection =
                    glm::perspective(16.0 / 9.0, camera.fov().to_radians(), 0.1, 100.0);

                unsafe {
                    gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    shader_program.use_program();
                    gl::BindVertexArray(vaos[0]);

                    let view = camera.view_matrix();
                    for &(name, val) in &[("view", view), ("projection", projection)] {
                        let loc = gl::GetUniformLocation(
                            shader_program.id(),
                            CString::new(name).unwrap().as_ptr(),
                        );
                        gl::UniformMatrix4fv(loc, 1, gl::FALSE, glm::value_ptr(&val).as_ptr());
                    }

                    for (i, p) in cube_positions.iter().enumerate() {
                        let mut model = glm::translate(&glm::Mat4::identity(), p);
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
