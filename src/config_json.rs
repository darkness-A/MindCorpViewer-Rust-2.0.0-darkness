use gl::types::GLsizei;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::File, io::Read, io::Write, path::Path};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::MindModel;
pub fn get_base_path() -> String {
    let exe_path = env::current_exe().unwrap();
    let base_dir = exe_path.parent().unwrap().to_str().unwrap().to_string();
    base_dir
}

impl Default for ConfigJson {
    fn default() -> Self {
        ConfigJson {
            msaa: Some(8),
            vsync: true,
            show_floor: true,
            show_skybox: true,
            synchronized_time: false,
            screen_shot_resolution: [1920, 1080],
            control_sensitivity: ControlSensitivity::default(),
            paths: vec![],
            options: vec![],
            meshes: vec![],
            skybox_file: "./skybox/Default.dds".parse().unwrap(),
        }
    }
}

pub static CONFIG_JSON: Lazy<Mutex<ConfigJson>> = Lazy::new(|| {
    Mutex::new(ConfigJson::default())
});
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PathJson {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Skin", alias = "SKN")]
    pub skin: String,

    #[serde(rename = "Skeleton", alias = "SKL")]
    pub skeleton: String,

    #[serde(rename = "Textures", alias = "DDS")]
    pub textures: String,

    #[serde(rename = "Animations", alias = "ANM")]
    pub animations: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptionsJson {
    #[serde(rename = "Show")]
    pub show: bool,

    #[serde(rename = "ShowWireframe")]
    pub show_wireframe: bool,

    #[serde(rename = "ShowSkeletonNames")]
    pub show_skeleton_names: bool,

    #[serde(rename = "ShowSkeletonBones")]
    pub show_skeleton_bones: bool,

    #[serde(rename = "ShowSkeletonJoints")]
    pub show_skeleton_joints: bool,

    #[serde(rename = "UseAnimation")]
    pub use_animation: bool,

    #[serde(rename = "PlayAnimation")]
    pub play_animation: bool,

    #[serde(rename = "LoopAnimation")]
    pub loop_animation: bool,

    #[serde(rename = "NextAnimation")]
    pub next_animation: bool,

    #[serde(rename = "AnimationTime")]
    pub animation_time: f32,

    #[serde(rename = "AnimationSpeed")]
    pub animation_speed: f32,

    #[serde(rename = "SelectedAnimation")]
    pub selected_animation_path: String,

    #[serde(rename = "PositionOffset")]
    pub position_offset: [f32; 3],
    
    #[serde(rename = "Rotation_angles")]
    pub rotation_angles: [f32; 3],

}

impl OptionsJson {
    pub fn new() -> OptionsJson {
        OptionsJson {
            show: true,
            show_wireframe: false,
            show_skeleton_names: false,
            show_skeleton_bones: false,
            show_skeleton_joints: false,
            use_animation: false,
            play_animation: false,
            loop_animation: true,
            next_animation: false,
            animation_time: 0.0f32,
            animation_speed: 1.0f32,
            selected_animation_path: String::new(),
            position_offset: [0.0, 0.0, 0.0],
            rotation_angles: [0.0, 0.0, 0.0],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeshJson {
    #[serde(rename = "Show")]
    pub show: bool,

    #[serde(flatten)]
    pub name_texture: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ControlSensitivity {
    #[serde(rename = "Zoom", default = "default_zoom")]
    pub zoom: f32,

    #[serde(rename = "Pan", default = "default_pan")]
    pub pan: f32,

    #[serde(rename = "Rotate", default = "default_rotate")]
    pub rotate: f32,
}
impl Default for ControlSensitivity {
    fn default() -> Self {
        ControlSensitivity {
            rotate: 0.5,
            pan: 0.7,
            zoom: 1.0,
        }
    }
}
fn default_zoom() -> f32 { 75.0 }
fn default_pan() -> f32 { 0.20 }
fn default_rotate() -> f32 { 0.03 }



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigJson {
    #[serde(rename = "MSAA")]
    pub msaa: Option<u32>,

    #[serde(rename = "Vsync")]
    pub vsync: bool,

    #[serde(rename = "ShowFloor")]
    pub show_floor: bool,

    #[serde(rename = "ShowSkybox")]
    pub show_skybox: bool,


    #[serde(rename = "SynchronizedTime")]
    pub synchronized_time: bool,

    #[serde(rename = "ScreenShotResolution")]
    pub screen_shot_resolution: [GLsizei; 2],

    #[serde(rename = "ControlSensitivity")]
    pub control_sensitivity: ControlSensitivity,

    #[serde(rename = "PATHS")]
    pub paths: Vec<PathJson>,

    #[serde(rename = "OPTIONS")]
    pub options: Vec<OptionsJson>,

    #[serde(rename = "MESHES")]
    pub meshes: Vec<Vec<MeshJson>>,

    #[serde(rename = "SkyboxFile")]
    pub skybox_file: String,
}

impl ConfigJson {
    pub fn read(path: &Path) -> ConfigJson {
        println!("Reading config file");

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(error) => {
                println!("Could not open config file: {error}");
                return ConfigJson::new();
            }
        };

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => {}
            Err(error) => {
                println!("Could not read config file: {error}");
                return ConfigJson::new();
            }
        }

        let mut config_json = match serde_json::from_str::<ConfigJson>(&contents) {
            Ok(config_json) => config_json,
            Err(error) => {
                println!("Could not deserialize config: {error}");
                ConfigJson::new()
            }
        };

        if config_json.options.len() < config_json.paths.len() {
            let diff = config_json.paths.len() - config_json.options.len();
            let options = vec![OptionsJson::new(); diff];
            config_json.options.extend_from_slice(&options);
        }

        if config_json.meshes.len() < config_json.paths.len() {
            let diff = config_json.paths.len() - config_json.meshes.len();
            let meshes = vec![vec![]; diff];
            config_json.meshes.extend_from_slice(&meshes);
        }

        println!("Finished reading config file");

        config_json
    }

    pub fn write(&self, mind_models: &[MindModel]) {
        println!("Writing to config file");

        let mut config_json = self.clone();

        config_json
            .options
            .iter_mut()
            .enumerate()
            .for_each(|(i, config)| {
                config.selected_animation_path = mind_models[i].animations_file_names
                    [mind_models[i].animation_selected]
                    .to_owned()
            });

        config_json.meshes = Vec::with_capacity(config_json.paths.len());
        for i in 0..config_json.paths.len() {
            let mind_model = &mind_models[i];

            let mut meshes = Vec::with_capacity(mind_model.skin.meshes.len());
            for i in 0..mind_model.skin.meshes.len() {
                let mut name_texture = BTreeMap::new();
                name_texture.insert(
                    mind_model.skin.meshes[i].submesh.name.to_owned(),
                    mind_model.textures_file_names[mind_model.textures_selecteds[i]].to_owned(),
                );
                meshes.push(MeshJson {
                    show: mind_model.show_meshes[i],
                    name_texture,
                });
            }
            config_json.meshes.push(meshes);
        }

        let contents = match pretty_json(&config_json) {
            Ok(contents) => contents,
            Err(error) => {
                return println!("Could not serialize config file: {error}");
            }
        };

        let mut file = match File::create(Path::new("config.json")) {
            Ok(file) => file,
            Err(error) => {
                return println!("Could not create config file: {error}");
            }
        };

        match file.write_all(contents.as_bytes()) {
            Ok(_) => {}
            Err(error) => {
                return println!("Could not write to config file: {error}");
            }
        }

        println!("Finished writing to config file");
    }

    pub fn new() -> ConfigJson {
        ConfigJson {
            msaa: Some(8),
            vsync: true,
            show_floor: true,
            show_skybox: true,
            synchronized_time: false,
            screen_shot_resolution: [1920, 1080],
            control_sensitivity: ControlSensitivity::default(),
            paths: vec![],
            options: vec![],
            meshes: vec![],
            skybox_file: String::new(),

        }
    }
}

fn pretty_json(config_json: &ConfigJson) -> Result<String, serde_json::Error> {
    let mut buffer = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
    let mut serializer = serde_json::Serializer::with_formatter(&mut buffer, formatter);
    config_json.serialize(&mut serializer)?;
    Ok(unsafe { String::from_utf8_unchecked(buffer) })
}

use std::env;

// 在结构体定义之后或 impl 块内添加以下函数
