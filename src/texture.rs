use std::ffi::c_void;
use std::path::Path;

use anyhow::Result;
use image::GenericImageView;

#[derive(Debug, PartialEq, Eq)]
pub struct Texture {
    id: u32,
}

impl Texture {
    pub unsafe fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let image = image::open(path)?;
        let (width, height) = image.dimensions();
        let raw = image.into_rgba().into_raw();

        let mut id = 0;
        gl::GenTextures(1, &mut id);
        gl::BindTexture(gl::TEXTURE_2D, id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA8 as i32,
            width as i32,
            height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            raw.as_ptr() as *const c_void,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        gl::BindTexture(gl::TEXTURE_2D, 0);

        Ok(Self { id })
    }

    pub unsafe fn set_wrap(&self, wrap_s: gl::types::GLenum, wrap_t: gl::types::GLenum) {
        gl::BindTexture(gl::TEXTURE_2D, self.id);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_s as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_t as i32);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    pub unsafe fn bind(&self) {
        gl::BindTexture(gl::TEXTURE_2D, self.id);
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
