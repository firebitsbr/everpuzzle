use std::ffi::c_void;

pub struct Texture {
    pub id: u32,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn new(name: &'static str) -> Self {
        let data = std::fs::read(name).expect("Failed to open PNG");
        let data = std::io::Cursor::new(data);
        let decoder = png_pong::FrameDecoder::<_, pix::Rgba8>::new(data);
        let png_pong::Frame { raster, delay: _ } = decoder
            .last()
            .expect("No frames in PNG")
            .expect("PNG parsing error");

        let width = raster.width();
        let height = raster.height();

        let mut id: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::MIRRORED_REPEAT as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::MIRRORED_REPEAT as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                raster.as_u8_slice().as_ptr() as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        Self { id, width, height }
    }
}
