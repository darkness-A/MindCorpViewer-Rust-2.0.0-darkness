use glfw::Glfw;
use native_dialog::FileDialog;
use std::path::PathBuf;
use imgui::StyleColor;
use crate::{
    config_json::{ConfigJson, OptionsJson},
    export, MindModel,
};

pub fn settings(
    ui: &imgui::Ui,
    glfw: &mut Glfw,
    has_samples: bool,
    use_samples: &mut bool,
    config_json: &mut ConfigJson,
    camera_pos: &mut glam::Vec3,  // 新增相机位置参数
    camera_rotation: &mut glam::Vec2,  // 新增相机旋转参数
) {
    if has_samples && ui.checkbox("开启多重采样抗锯齿(MSAA)", use_samples) {
        match use_samples {
            true => unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                gl::Enable(gl::MULTISAMPLE);
                gl::Enable(gl::SAMPLE_ALPHA_TO_COVERAGE);
            },
            false => unsafe {
                gl::Disable(gl::BLEND);
                gl::Disable(gl::MULTISAMPLE);
                gl::Disable(gl::SAMPLE_ALPHA_TO_COVERAGE);
            },
        }
    }

    if ui.checkbox("开启垂直同步(Enable Vsync)", &mut config_json.vsync) {
        glfw.set_swap_interval(match config_json.vsync {
            true => glfw::SwapInterval::Sync(1),
            false => glfw::SwapInterval::None,
        });
    }

    ui.checkbox("显示地面(Show Floor)", &mut config_json.show_floor);
    ui.checkbox("显示天空(Show Skybox)", &mut config_json.show_skybox);

    ui.checkbox("时间同步(Synchronized Time)", &mut config_json.synchronized_time);
    if ui.is_item_hovered() {
        ui.tooltip(|| {
            ui.text("将所有模型与第一个模型同步(Synchronize all models to first model)");
        });
    }

    ui.separator();
    // 缩放灵敏度
    ui.text("缩放灵敏度(Zoom):");
    ui.same_line();
    ui.slider_config("##zoom_sensitivity", 1.0, 200.0)
        .display_format("%.1f")
        .build(&mut config_json.control_sensitivity.zoom);
    // 平移灵敏度
    ui.text("平移灵敏度(Pan):");
    ui.same_line();
    ui.slider_config("##pan_sensitivity", 0.01, 2.0)
        .display_format("%.2f")
        .build(&mut config_json.control_sensitivity.pan);
    // 旋转灵敏度
    ui.text("旋转灵敏度(Rotate):");
    ui.same_line();
    ui.slider_config("##rotate_sensitivity", 0.01, 0.2)
        .display_format("%.2f")
        .build(&mut config_json.control_sensitivity.rotate);


    let _button_style = ui.push_style_color(imgui::StyleColor::Button, [1.0, 0.3, 0.3, 1.0]);
    if ui.button("重置镜头坐标(Reset)") {
        *camera_pos = glam::Vec3::new(0.0, 0.0, 5.0);
        *camera_rotation = glam::Vec2::new(90.0, 70.0);
    }
    if ui.is_item_hovered() {
        ui.tooltip(|| { ui.text("将相机重置到默认位置和角度"); });
    }
    _button_style.pop();
    // 让下一个控件紧贴前一个控件
    ui.same_line();
    // 第二个按钮 - 重置灵敏度
    let _button_style2 = ui.push_style_color(imgui::StyleColor::Button, [0.3, 0.7, 1.0, 1.0]);
    if ui.button("重置灵敏度(Sensitivity)") {
        config_json.control_sensitivity.zoom = 75.0;
        config_json.control_sensitivity.pan = 0.20;
        config_json.control_sensitivity.rotate = 0.03;
    }
    if ui.is_item_hovered() {
        ui.tooltip(|| { ui.text("重置缩放、平移、旋转灵敏度为默认值"); });
    }
    _button_style2.pop();
}

pub fn model(
    ui: &imgui::Ui,
    options: &mut OptionsJson,
    mind_model: &mut MindModel,
    export_as: &mut u8,
    name: &String,
) {
    let _header_style = ui.push_style_color(StyleColor::Header, [0.2, 0.5, 0.3, 1.0]);      // 深绿
    let _hover_style = ui.push_style_color(StyleColor::HeaderHovered, [0.3, 0.6, 0.4, 1.0]); // 浅绿
    let _active_style = ui.push_style_color(StyleColor::HeaderActive, [0.1, 0.4, 0.2, 1.0]); // 墨绿
    // 可选：配套文字颜色（浅灰提升可读性）
    let _text_style = ui.push_style_color(StyleColor::Text, [0.9, 0.9, 0.9, 1.0]); // 灰字
    ui.checkbox("显示线框(Show Wireframe)", &mut options.show_wireframe);
    ui.checkbox("显示骨骼名称(Show Skeleton Names)", &mut options.show_skeleton_names);
    ui.checkbox("显示骨骼(Show Skeleton Bones)", &mut options.show_skeleton_bones);
    ui.checkbox("显示关节(Show Skeleton Joints)", &mut options.show_skeleton_joints);

    ui.tree_node_config("网格(Meshes)")
        .flags(imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH)
        .framed(true)
        .build(|| {
            for i in 0..mind_model.skin.meshes.len() {
                let _meshes_id = ui.push_id_usize(i);
                ui.checkbox(
                    mind_model.skin.meshes[i].submesh.name.as_str(),
                    &mut mind_model.show_meshes[i],
                );
                if mind_model.show_meshes[i] {
                    ui.combo_simple_string(
                        "##texture",
                        &mut mind_model.textures_selecteds[i],
                        &mind_model.textures_file_names,
                    );
                    imgui::Image::new(
                        imgui::TextureId::new(
                            mind_model.textures[mind_model.textures_selecteds[i]].id as usize,
                        ),
                        [64.0f32, 64.0f32],
                    )
                        .build(ui);
                }
            }
        });
    
    ui.tree_node_config("动画(Animations)")
        .flags(imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH)
        .framed(true)
        .build(|| {
            ui.checkbox("使用动画(Use Animation)", &mut options.use_animation);
            ui.checkbox("播放/停止(Play / Stop)", &mut options.play_animation);
            ui.checkbox("循环动画(Loop Animation)", &mut options.loop_animation);
            ui.checkbox("下一动画(Next Animation)", &mut options.next_animation);

            ui.text("CTRL+点击更改输入(CTRL+Click Change To Input)");

            ui.align_text_to_frame_padding();
            ui.text("速度(Speed):     ");
            ui.same_line();
            ui.slider_config("##speed", 0.00001f32, 10.0f32)
                .display_format("%.5f")
                .flags(imgui::SliderFlags::ALWAYS_CLAMP)
                .build(&mut options.animation_speed);

            ui.align_text_to_frame_padding();
            ui.text("时间(Time):      ");
            ui.same_line();
            ui.slider_config(
                "##time",
                0.0f32,
                mind_model.animations[mind_model.animation_selected].duration,
            )
                .display_format("%.5f")
                .flags(imgui::SliderFlags::ALWAYS_CLAMP)
                .build(&mut options.animation_time);

            ui.align_text_to_frame_padding();
            ui.text("动画列表(Animations):");
            ui.same_line();
            ui.combo_simple_string(
                "##animations",
                &mut mind_model.animation_selected,
                &mind_model.animations_file_names,
            );
        });

    

    ui.tree_node_config("位置(Position)")
        .flags(imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH)
        .framed(true)
        .build(|| {
            ui.text("X轴左右偏移(X Offset):  ");
            ui.same_line();
            ui.slider_config("##x_offset", -700.0f32, 700.0f32)
                .display_format("%.2f")
                .flags(imgui::SliderFlags::ALWAYS_CLAMP)
                .build(&mut options.position_offset[0]);

            ui.text("Y轴上下偏移(Y Offset):  ");
            ui.same_line();
            ui.slider_config("##y_offset", -700.0f32, 700.0f32)
                .display_format("%.2f")
                .flags(imgui::SliderFlags::ALWAYS_CLAMP)
                .build(&mut options.position_offset[1]);

            ui.text("Z轴前后偏移(Z Offset):  ");
            ui.same_line();
            ui.slider_config("##z_offset", -700.0f32, 700.0f32)
                .display_format("%.2f")
                .flags(imgui::SliderFlags::ALWAYS_CLAMP)
                .build(&mut options.position_offset[2]);

            if ui.button("重置位置(Reset Position)") {
                options.position_offset[0] = 0.0;
                options.position_offset[1] = 0.0;
                options.position_offset[2] = 0.0;
            }
        });


    ui.tree_node_config("导出(Export)")
        .flags(imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH)
        .framed(true)
        .build(|| {
            ui.radio_button("导出为gltf(Export as gltf)", export_as, 0);
            ui.radio_button("导出为glb(Export as glb)", export_as, 1);
            if ui.button_with_size("导出模型(Export Model)", [ui.content_region_avail()[0], 0.0f32]) {
                export::export_model(*export_as, name, mind_model);
            }
            if ui.is_item_hovered() {
                ui.tooltip(|| {
                    ui.text("默认保存到软件目录export目录下");
                });
            }
        });
}

pub struct AddModel {
    pub name: String,
    pub skin: String,
    pub skeleton: String,
    pub textures: String,
    pub animations: String,
}

impl AddModel {
	pub fn new() -> Self {
		Self {
			name: String::new(),
			skin: String::new(),
			skeleton: String::new(),
			textures: String::new(),
			animations: String::new(),
		}
	}
}

pub fn add_model<F>(
    ui: &imgui::Ui,
    working_dir: &PathBuf,
    add_model: &mut AddModel,
    mut add_funct: F,
) where
    F: FnMut(&mut AddModel),
{
    ui.tree_node_config("添加模型(Add Model)")
        .flags(imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH)
        .framed(true)
        .build(|| {
            ui.align_text_to_frame_padding();
            ui.text("名称(Name):       ");
            ui.same_line();
            ui.input_text("##name", &mut add_model.name).build();

            ui.align_text_to_frame_padding();
            ui.text("皮肤(Skin):       ");
            ui.same_line();
            ui.input_text("##skin", &mut add_model.skin).build();
            ui.same_line();
            if ui.button("选择(Select)##1") {
                let file_dialog_path = FileDialog::new()
                    .set_location(&working_dir)
                    .add_filter("皮肤(Skin)", &["skn"])
                    .show_open_single_file()
                    .unwrap();

                if let Some(path) = file_dialog_path {
                    // 更新 Skin 字段
                    add_model.skin.clear();
                    add_model.skin.insert_str(0, path.to_str().unwrap());

                    // 解析路径并推导其他文件路径
                    if let Some(parent_dir) = path.parent() {
                        let file_stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("");


                        add_model.name.clear();
                        add_model.name.insert_str(0, file_stem);

                        // 推导 Skeleton 路径
                        let skeleton_path = parent_dir.join(format!("{}.skl", file_stem));
                        if skeleton_path.exists() {
                            add_model.skeleton.clear();
                            add_model.skeleton.insert_str(0, skeleton_path.to_str().unwrap());
                        }

                        // 推导 Textures 路径
                        let textures_path = parent_dir.join("textures"); // 假设纹理在textures目录下
                        if textures_path.exists() {
                            add_model.textures.clear();
                            add_model.textures.insert_str(0, textures_path.to_str().unwrap());
                        } else {
                            add_model.textures.clear(); // 如果路径不存在，则保持为空
                        }

                        // 推导 Animations 路径
                        let animations_path = parent_dir.join("animations");
                        if animations_path.exists() {
                            add_model.animations.clear();
                            add_model.animations.insert_str(0, animations_path.to_str().unwrap());
                        }
                    }
                }
            }


            ui.align_text_to_frame_padding();
            ui.text("骨骼(Skeleton):   ");
            ui.same_line();
            ui.input_text("##skeleton", &mut add_model.skeleton).build();
            ui.same_line();
            if ui.button("选择(Select)##2") {
                let file_dialog_path = FileDialog::new()
                    .set_location(&working_dir)
                    .add_filter("骨骼(Skeleton)", &["skl"])
                    .show_open_single_file()
                    .unwrap();
                if let Some(path) = file_dialog_path {
                    add_model.skeleton.clear();
                    add_model.skeleton.insert_str(0, path.to_str().unwrap());
                }
            }

            ui.align_text_to_frame_padding();
            ui.text("纹理(Textures):   ");
            ui.same_line();
            ui.input_text("##textures", &mut add_model.textures).build();
            ui.same_line();
            if ui.button("选择(Select)##3") {
                let path = FileDialog::new()
                    .set_location(&working_dir)
                    .add_filter("纹理(Textures)", &["dds", "tex"])
                    .show_open_single_dir()
                    .unwrap();
                if let Some(path) = path {
                    add_model.textures.clear();
                    add_model.textures.insert_str(0, path.to_str().unwrap());
                }
            }

            ui.align_text_to_frame_padding();
            ui.text("动画(Animations): ");
            ui.same_line();
            ui.input_text("##animations", &mut add_model.animations)
                .build();
            ui.same_line();
            if ui.button("选择(Select)##4") {
                let file_dialog_path = FileDialog::new()
                    .set_location(&working_dir)
                    .add_filter("动画(Animations)", &["anm"])
                    .show_open_single_dir()
                    .unwrap();
                if let Some(path) = file_dialog_path {
                    add_model.animations.clear();
                    add_model.animations.insert_str(0, path.to_str().unwrap());
                }
            }

            ui.separator();
            if ui.button_with_size("导入JSON配置(Import JSON)", [ui.content_region_avail()[0], 0.0f32]) {
                if let Some(path) = FileDialog::new()
                    .set_location(&working_dir)
                    .add_filter("JSON配置", &["json"])
                    .show_open_single_file()
                    .unwrap()
                {
                    if let Ok(json_str) = std::fs::read_to_string(&path) {
                            if let Ok(config) = serde_json::from_str::<ConfigJson>(&json_str) {
                                if !config.paths.is_empty() {
                                    let path = &config.paths[0];
                                    add_model.name = path.name.clone();
                                    add_model.skin = path.skin.clone();
                                    add_model.skeleton = path.skeleton.clone();
                                    add_model.textures = path.textures.clone();
                                    add_model.animations = path.animations.clone();

                                    // 如果配置中有MESHES数据，保存到临时文件供后续使用
                                    if !config.meshes.is_empty() {
                                        let _ = std::fs::write(
                                            "temp_meshes.json",
                                            serde_json::to_string_pretty(&config.meshes[0]).unwrap()
                                        );
                                    }
                                }
                            }
                    }
                }
            }

            if ui.button_with_size("添加(Add)", [ui.content_region_avail()[0], 0.0f32]) {
                // 添加路径空值检查
                if add_model.skin.is_empty() {
                    ui.open_popup("##missing_skin");
                } else if add_model.skeleton.is_empty() {
                    ui.open_popup("##missing_skeleton");
                } else {
                    add_funct(add_model);
                    add_model.name.clear();
                    add_model.skin.clear();
                    add_model.skeleton.clear();
                    add_model.textures.clear();
                    add_model.animations.clear();
                }
            }
            // 添加错误提示弹窗
            ui.popup("##missing_skin", || {
                ui.text("错误: 必须指定皮肤文件路径!");
                if ui.button("确定") { ui.close_current_popup(); }
            });
            ui.popup("##missing_skeleton", || {
                ui.text("错误: 必须指定骨骼文件路径!");
                if ui.button("确定") { ui.close_current_popup(); }
            });
        });
}

pub fn screenshot(
    ui: &imgui::Ui,
    use_samples: bool,
    take_screenshot: &mut bool,
    screenshot: &mut super::Screenshot,
    config_json: &mut ConfigJson,
) {
    ui.tree_node_config("截图(Screenshot)")
        .flags(imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH)
        .framed(true)
        .build(|| {
            ui.align_text_to_frame_padding();
            ui.text("分辨率(Resolution):");
            ui.same_line();
            ui.input_scalar_n("##resolution", &mut config_json.screen_shot_resolution)
                .build();

            if config_json.screen_shot_resolution[0] == 0 {
                config_json.screen_shot_resolution[0] = 1280;
            }
            if config_json.screen_shot_resolution[1] == 0 {
                config_json.screen_shot_resolution[1] = 720;
            }

            ui.align_text_to_frame_padding();
            ui.text("文件名(File name): ");
            ui.same_line();
            ui.input_text("##file_name", &mut screenshot.file_name)
                .build();

            ui.align_text_to_frame_padding();
            ui.text("格式(Format):    ");
            ui.same_line();
            ui.combo_simple_string("##format", &mut screenshot.format, &FORMATS);

            if ui.button_with_size("保存图片到桌面(Take to Desktop)", [ui.content_region_avail()[0], 0.0f32]) {
                *take_screenshot = true;
                screenshot.use_samples = use_samples;
                screenshot.resolution = config_json.screen_shot_resolution;
            }
        });
}

const FORMATS: [&str; 4] = ["PNG", "JPG", "BMP", "TIFF"];

pub fn confirm_delete_button(ui: &imgui::Ui) -> bool {
    let delete_button = ui.button("\u{F014}");
    if ui.is_item_hovered() {
        ui.tooltip(|| {
            ui.text("删除项目？(Delete item?)");
        });
    }
    if delete_button {
        ui.open_popup("##deletepopup");
    }
    let mut delete = false;
    ui.popup("##deletepopup", || {
        ui.text("是否确认？(Are you sure?)");
        if ui.button("是(Yes)") {
            ui.close_current_popup();
            delete = true;
        }
        ui.same_line();
        if ui.button("否(No)") {
            ui.close_current_popup();
        }
    });
    delete
}

pub fn no_window_hovered() -> bool {
    unsafe { !imgui::sys::igIsWindowHovered(imgui::WindowHoveredFlags::ANY_WINDOW.bits() as i32) }
}
