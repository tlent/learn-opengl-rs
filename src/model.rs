use std::ffi::c_void;
use std::iter;
use std::mem;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::Result;
use memoffset::offset_of;
use nalgebra_glm as glm;

use crate::shader_program::ShaderProgram;
use crate::texture::Texture;

pub struct Model {
    meshes: Vec<Mesh>,
}

impl Model {
    pub unsafe fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let mut texture_loader = TextureLoader::new();
        let (models, materials) = tobj::load_obj(path.as_ref(), true)?;
        let meshes = models
            .into_iter()
            .map(|model| {
                let mesh = model.mesh;
                let positions = mesh.positions.chunks_exact(3);
                let normals = mesh
                    .normals
                    .chunks_exact(3)
                    .map(|n| Some(n))
                    .chain(iter::repeat(None))
                    .take(positions.len());
                let texture_coords = mesh
                    .texcoords
                    .chunks_exact(2)
                    .map(|c| Some(c))
                    .chain(iter::repeat(None))
                    .take(positions.len());
                let vertices: Vec<_> = positions
                    .zip(normals.zip(texture_coords))
                    .map(|(p, (n, t))| {
                        let position = glm::vec3(p[0], p[1], p[2]);
                        let normal = n
                            .map(|n| glm::vec3(n[0], n[1], n[2]))
                            .expect("No normals in .obj");
                        let texture_coordinate = t
                            .map(|t| glm::vec2(t[0], t[1]))
                            .unwrap_or(glm::vec2(0.0, 0.0));
                        Vertex {
                            position,
                            normal,
                            texture_coordinate,
                        }
                    })
                    .collect();
                let indices = mesh.indices;
                let mut diffuse_textures = vec![];
                let mut specular_textures = vec![];
                if let Some(id) = mesh.material_id {
                    let material = &materials[id];
                    let base_path = path.as_ref().parent().unwrap_or("/".as_ref());
                    if !material.diffuse_texture.is_empty() {
                        let mut path = PathBuf::from(&material.diffuse_texture);
                        if path.is_relative() {
                            path = base_path.join(path);
                        }
                        diffuse_textures.push(texture_loader.load(path)?);
                    };
                    if !material.specular_texture.is_empty() {
                        let mut path = PathBuf::from(&material.specular_texture);
                        if path.is_relative() {
                            path = base_path.join(path);
                        }
                        specular_textures.push(texture_loader.load(path)?);
                    };
                }
                Ok(Mesh::new(
                    vertices,
                    indices,
                    diffuse_textures,
                    specular_textures,
                ))
            })
            .collect::<Result<_>>()?;
        Ok(Self { meshes })
    }

    pub unsafe fn draw(&self, shader: &ShaderProgram) {
        for mesh in self.meshes.iter() {
            mesh.draw(shader);
        }
    }
}

#[derive(Debug)]
struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    diffuse_textures: Vec<Rc<Texture>>,
    specular_textures: Vec<Rc<Texture>>,
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl Mesh {
    unsafe fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        diffuse_textures: Vec<Rc<Texture>>,
        specular_textures: Vec<Rc<Texture>>,
    ) -> Self {
        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<Vertex>()) as isize,
            vertices.as_ptr() as *const c_void,
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
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as i32,
            offset_of!(Vertex, normal) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<Vertex>() as i32,
            offset_of!(Vertex, texture_coordinate) as *const c_void,
        );
        gl::EnableVertexAttribArray(2);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * mem::size_of::<u32>()) as isize,
            indices.as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::BindVertexArray(0);
        Self {
            vertices,
            indices,
            diffuse_textures,
            specular_textures,
            vao,
            vbo,
            ebo,
        }
    }

    unsafe fn draw(&self, shader: &ShaderProgram) {
        let mut texture_num = 0;
        let diffuse_textures = self.diffuse_textures.iter().enumerate();
        for (diffuse_num, texture) in diffuse_textures {
            let name = &format!("material.texture_diffuse[{}]", diffuse_num);
            shader.set_uniform_int(name, texture_num as i32);
            gl::ActiveTexture(gl::TEXTURE0 + texture_num as u32);
            texture.bind();
            texture_num += 1;
        }
        let specular_textures = self.specular_textures.iter().enumerate();
        for (specular_num, texture) in specular_textures {
            let name = &format!("material.texture_specular[{}]", specular_num);
            shader.set_uniform_int(name, texture_num as i32);
            gl::ActiveTexture(gl::TEXTURE0 + texture_num as u32);
            texture.bind();
            texture_num += 1;
        }
        gl::ActiveTexture(gl::TEXTURE0);

        gl::BindVertexArray(self.vao);
        gl::DrawElements(
            gl::TRIANGLES,
            self.indices.len() as i32,
            gl::UNSIGNED_INT,
            0 as *const c_void,
        );
        gl::BindVertexArray(0);
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(2, [self.vbo, self.ebo].as_ptr());
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
    position: glm::Vec3,
    normal: glm::Vec3,
    texture_coordinate: glm::Vec2,
}

struct TextureLoader {
    cache: Vec<(PathBuf, Rc<Texture>)>,
}

impl TextureLoader {
    fn new() -> Self {
        Self { cache: vec![] }
    }

    unsafe fn load<P>(&mut self, path: P) -> Result<Rc<Texture>>
    where
        P: Into<PathBuf>,
    {
        let path = path.into();
        match self.cache.iter().find(|(p, _)| *p == path) {
            Some((_, texture)) => Ok(Rc::clone(&texture)),
            None => {
                let texture = Rc::new(Texture::load(&path)?);
                self.cache.push((path, Rc::clone(&texture)));
                Ok(texture)
            }
        }
    }
}
