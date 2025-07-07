use gl::types::{GLfloat, GLint, GLsizeiptr, GLuint};
use std::{
    mem,
    os::raw::c_void,
    ptr,
    sync::{Arc, Mutex},
};
use crate::gls::{Shader, Texture};
use crate::config_json::CONFIG_JSON;

use std::sync::atomic::{AtomicUsize, Ordering};

static SKYBOX_INSTANCE_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);


lazy_static::lazy_static! {
    // 使用 Arc + Mutex 管理动态纹理资源
    pub static ref SKYBOX_TEXTURE: Arc<Mutex<Option<Texture>>> = Arc::new(Mutex::new(None));
}

pub struct Skybox {
    id: usize,
    shader: Shader,
    vao: GLuint,
    bo: Vec<GLuint>,
    mvp_ref: GLint,
}

impl Skybox {
    pub fn new() -> Skybox {
        let mut test: gl::types::GLint = 0;
        unsafe {
            gl::GetIntegerv(gl::SAMPLES, &mut test);
            println!("🧪 测试 glGetIntegerv(GL_SAMPLES): {}", test);
        }
        let id = SKYBOX_INSTANCE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        println!("🆕 创建 Skybox 实例 #{}", id);
        let shader = Shader::create(
            include_str!("../../assets/skybox/skybox.vert"),
            include_str!("../../assets/skybox/skybox.frag"),
        );
        
        if shader.id == 0 {
            eprintln!("❌ 着色器创建失败，ID 为 0");
        } else {
            println!("⚡️ 着色器创建成功，ID: {}", shader.id);
        }

        let refs = shader.get_refs(&["MVP"]);
        let mut vao: GLuint = 0;
        let mut bo: Vec<GLuint> = vec![0; 2];
        unsafe {
            // 初始化 VAO 和 VBO（保持原有几何数据不变）

            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(2, bo.as_mut_ptr());

            println!("📦 创建 VAO: {}", vao);

            gl::BindVertexArray(vao);

            // 顶点缓冲
            gl::BindBuffer(gl::ARRAY_BUFFER, bo[0]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (SKYBOX_VERTICES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                SKYBOX_VERTICES.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
                ptr::null(),
            );

            // 索引缓冲
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, bo[1]);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (SKYBOX_INDICES.len() * mem::size_of::<GLint>()) as GLsizeiptr,
                SKYBOX_INDICES.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::BindVertexArray(0);
        }

        Skybox {
            id,
            shader,
            vao,
            bo,
            mvp_ref: refs[0],
        }
    }

    // 更新天空盒纹理
    pub fn update(&mut self) {
        unsafe {
            println!("update更新天空盒纹理");
            // 获取当前配置中的贴图路径
            let config = CONFIG_JSON.lock().unwrap();
            let skybox_path = config.skybox_file.clone();
            drop(config);
            println!(" 当前配置: show_skybox={}", CONFIG_JSON.lock().unwrap().show_skybox);
            println!("🖼️  贴图路径: {}", skybox_path);
            // 如果路径为空或不存在，则不更新
            if skybox_path.is_empty() {
                println!("Skybox path is empty!");
                return;
            }

            if !std::path::Path::new(&skybox_path).exists() {
                println!("Skybox path does not exist: {}", skybox_path);
                return;
            }
            println!("使用贴图skybox_path: {:?}", skybox_path);

            // 加载新贴图
            match Texture::load_cubemap_from_single_dds_file(&skybox_path) {
                Ok(texture) => {
                    // 替换旧贴图
                    if let Some(old_texture) = SKYBOX_TEXTURE.lock().unwrap().take() {
                        old_texture.delete();
                    }
                    *SKYBOX_TEXTURE.lock().unwrap() = Some(texture);
                }
                Err(e) => {
                    eprintln!("Failed to load skybox texture: {}", e);
                }
            }
        }
    }

    pub fn render(&self, view_matrix: &glam::Mat4, projection_matrix: &glam::Mat4) {
        unsafe {
            //println!("👁️ 使用 Skybox 实例 #{}, VAO: {}", self.id, self.vao);
            //println!("✅ 开始绘制天空盒");
            gl::Disable(gl::DEPTH_TEST);

            let texture_guard = match SKYBOX_TEXTURE.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    eprintln!("🔒 无法锁定纹理锁");
                    return;
                }
            };

            let texture = match texture_guard.as_ref() {
                Some(texture) => texture,
                None => {
                    //eprintln!("🖼️ 纹理未加载");
                    return;
                }
            };

            //println!("🔗 使用纹理: {:?}", texture);
            texture.bind();

            self.shader.enable();
            if self.shader.id == 0 {
                eprintln!("❌ 着色器 ID 无效");
                return;
            }

            let rotation = glam::Mat3::from_mat4(*view_matrix);
            let skybox_view_matrix = glam::Mat4::from_mat3(rotation);
            let model = glam::Mat4::IDENTITY;
            let mvp = *projection_matrix * skybox_view_matrix * model;

            gl::UniformMatrix4fv(
                self.mvp_ref,
                1,
                gl::FALSE,
                mvp.as_ref().as_ptr(),
            );

            gl::ActiveTexture(gl::TEXTURE0);

            if self.vao == 0 {
                eprintln!("❌ 无效的 VAO，ID 为 0 (self.vao = {})", self.vao);
                return;
            }
            gl::BindVertexArray(self.vao);
            //println!("👁️ 使用 VAO: {}", self.vao);
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.bo[1]);
            gl::DrawElements(
                gl::TRIANGLES,
                36,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl::DisableVertexAttribArray(0);
            gl::BindVertexArray(0);

            gl::Enable(gl::DEPTH_TEST);
        }
    }
}

// 天空盒几何数据（保持不变）
const SKYBOX_VERTICES: [GLfloat; 24] = [
    -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0,
];

const SKYBOX_INDICES: [GLuint; 36] = [
    1, 2, 6, 6, 5, 1, 0, 4, 7, 7, 3, 0, 4, 5, 6, 6, 7, 4, 0, 3, 2, 2, 1, 0, 0, 1, 5,
    5, 4, 0, 3, 7, 6, 6, 2, 3,
];

impl Drop for Skybox {
    fn drop(&mut self) {
        println!("🧨 Skybox 正在被 Drop，VAO: {}", self.vao);
        unsafe {
            gl::DeleteBuffers(2, self.bo.as_ptr());
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
