use gl::types::{GLfloat, GLint, GLsizei, GLsizeiptr, GLuint};
use std::{mem, os::raw::c_void, ptr};

use crate::gls::{Shader, Texture};

pub struct Floor {
    shader: Shader,
    texture: Texture,
    vao: GLuint,
    bo: GLuint,
    mvp_ref: GLint,
}

impl Floor {
    pub fn new() -> Floor {
        let shader = Shader::create(
            include_str!("../../assets/floor/floor.vert"),
            include_str!("../../assets/floor/floor.frag"),
        );
        let refs = shader.get_refs(&["Diffuse", "MVP"]);

        let texture = Texture::load_texture(include_bytes!("../../assets/floor/floor2.dds"));

        // #[rustfmt::skip]
		// let floor_vertices: [GLfloat; 30] = [
		// 	 750.0f32, 0.0f32,  750.0f32, 0.0f32, 1.0f32,
		// 	 750.0f32, 0.0f32, -750.0f32, 0.0f32, 0.0f32,
		// 	-750.0f32, 0.0f32,  750.0f32, 1.0f32, 1.0f32,
		// 	 750.0f32, 0.0f32, -750.0f32, 0.0f32, 0.0f32,
		// 	-750.0f32, 0.0f32, -750.0f32, 1.0f32, 0.0f32,
		// 	-750.0f32, 0.0f32,  750.0f32, 1.0f32, 1.0f32,
		// ];

        #[rustfmt::skip]
        let floor_width = 1024.0;  // 匹配纹理宽度
        let floor_height = 1024.0; // 匹配纹理高度
        let half_width = floor_width / 2.0;
        let half_height = floor_height / 2.0;

        let floor_vertices: [GLfloat; 30] = [
            // X     Y    Z       U    V
            half_width, 0.0,  half_height,  1.0, 0.0,
            -half_width, 0.0,  half_height,  0.0, 0.0,
            half_width, 0.0, -half_height,  1.0, 1.0,

            -half_width, 0.0, -half_height,  0.0, 1.0,
            half_width, 0.0, -half_height,  1.0, 1.0,
            -half_width, 0.0,  half_height,  0.0, 0.0,
        ];
        // #[rustfmt::skip]
        //      let floor_size = 2048.0f32; // 扩展到 2000x2000，可根据需求调整
        // let uv_scale = 1.0f32;      // 纹理重复次数，可以控制贴图拉伸/平铺
        //
        // let floor_vertices: [GLfloat; 30] = [
        //     // 第一个三角形（翻转顺序）
        //      floor_size, 0.0,  floor_size,  1.0, 0.0, // 原来的第三个顶点
        //     -floor_size, 0.0,  floor_size,  0.0, 0.0, // 原来的第一个顶点
        //      floor_size, 0.0, -floor_size,  1.0, 1.0, // 原来的第二个顶点
        //
        //     // 第二个三角形（翻转顺序）
        //     -floor_size, 0.0, -floor_size,  0.0, 1.0, // 原来的第六个顶点
        //      floor_size, 0.0, -floor_size,  1.0, 1.0, // 原来的第四个顶点
        //     -floor_size, 0.0,  floor_size,  0.0, 0.0, // 原来的第五个顶点
        // ];
        unsafe {

            shader.enable();
            gl::Uniform1i(refs[0], 0);

            let mut vao: GLuint = 0;
            let mut bo: GLuint = 0;

            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut bo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, bo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (floor_vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                floor_vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            let stride = (5 * mem::size_of::<GLfloat>()) as GLsizei;
            let offset = (3 * mem::size_of::<GLfloat>()) as *const c_void;

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, offset);

            gl::BindVertexArray(0);

            Floor {
                shader,
                texture,
                vao,
                bo,
                mvp_ref: refs[1],
            }
        }
    }

    pub fn render(&self, projection_view_matrix: &glam::Mat4) {
        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);

            self.shader.enable();
            gl::UniformMatrix4fv(
                self.mvp_ref,
                1,
                gl::FALSE,
                projection_view_matrix.as_ref() as *const GLfloat,
            );

            gl::ActiveTexture(gl::TEXTURE0);
            self.texture.bind();

            gl::BindVertexArray(self.vao);

            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            gl::BindVertexArray(0);
        }
    }
}

impl Drop for Floor {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.bo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
