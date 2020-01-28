use gl::types::*;
use std::ffi::CString;
use std::ptr;
use std::str;

fn check_error(id: u32) {
    unsafe {
        let mut len = 0;
        gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
        gl::GetShaderInfoLog(id, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        panic!(
            "{}",
            str::from_utf8(&buf)
                .ok()
                .expect("ShaderInfoLog not valid utf8")
        );
    }
}

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            check_error(shader);
        }
    }
    shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            check_error(program);
        }
        program
    }
}

pub fn create_shader(vert_src: &str, frag_src: &str) -> u32 {
    let vs = compile_shader(vert_src, gl::VERTEX_SHADER);
    let fs = compile_shader(frag_src, gl::FRAGMENT_SHADER);
    let program = link_program(vs, fs);

    unsafe {
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);
    }

    program
}

use std::fs::File;
use std::io::Read;

pub fn load_shader(name: &'static str) -> u32 {
    let vert_path = format!("shaders/{}.vert", name);
    let mut vert_file = File::open(vert_path).expect("FS: vert file not found");
    let mut vert_src = String::new();
    vert_file
        .read_to_string(&mut vert_src)
        .expect("FS: vert file not read");

    let frag_path = format!("shaders/{}.frag", name);
    let mut frag_file = File::open(frag_path).expect("FS: frag file not found");
    let mut frag_src = String::new();
    frag_file
        .read_to_string(&mut frag_src)
        .expect("FS: frag file not read");

    create_shader(&vert_src, &frag_src)
}
