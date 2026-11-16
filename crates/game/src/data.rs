use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ManifestEntry {
    pub name: String,
    pub path: String,
    pub character: String,
    #[serde(rename = "mapPos")]
    pub map_pos: [f32; 2],
}

#[derive(Debug, Deserialize, Clone)]
pub struct LayoutImage {
    pub file: String,
    pub x: f32,
    pub y: f32,
    #[serde(rename = "scaleX")]
    pub scale_x: f32,
    #[serde(rename = "scaleY")]
    pub scale_y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlayerSizeTier {
    pub size: f32,
    #[serde(rename = "targetScore")]
    pub target_score: f32,
    pub scale: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SpawnInfo {
    #[serde(rename = "playerSize")]
    pub player_size: f32,
    #[serde(rename = "maxCount")]
    pub max_count: usize,
    #[serde(rename = "respawnFreq")]
    pub respawn_freq: [f32; 2],
}

#[derive(Debug, Deserialize, Clone)]
pub struct EnemySpawnDef {
    pub name: String,
    pub template: String,
    #[serde(rename = "poolSize")]
    pub pool_size: usize,
    #[serde(rename = "spawnInfo")]
    pub spawn_info: Vec<SpawnInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StageDef {
    pub name: String,
    #[serde(rename = "friendlyName")]
    pub friendly_name: String,
    #[serde(rename = "nextLevelAt")]
    pub next_level_at: f32,
    #[serde(rename = "stageWidth")]
    pub stage_width: f32,
    #[serde(rename = "stageHeight")]
    pub stage_height: f32,
    pub time: f32,
    #[serde(rename = "stageType")]
    pub stage_type: String,
    #[serde(rename = "bgImages")]
    pub bg_images: Vec<LayoutImage>,
    #[serde(rename = "mgImages")]
    pub mg_images: Vec<LayoutImage>,
    #[serde(rename = "fgImages")]
    pub fg_images: Vec<LayoutImage>,
    #[serde(rename = "playerConfig")]
    pub player_config: String,
    #[serde(rename = "playerSizes")]
    pub player_sizes: Vec<PlayerSizeTier>,
    pub enemies: Vec<EnemySpawnDef>,
    pub music: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReactDef {
    pub class: String,
    pub reaction: String,
    #[serde(rename = "reactFreq")]
    pub react_freq: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FishConfig {
    pub name: String,
    pub class: String,
    #[serde(rename = "foodValue")]
    pub food_value: f32,
    #[serde(rename = "maxSpeedX")]
    pub max_speed_x: [f32; 2],
    #[serde(rename = "maxSpeedY")]
    pub max_speed_y: [f32; 2],
    #[serde(rename = "minSpeedX")]
    pub min_speed_x: [f32; 2],
    #[serde(rename = "minSpeedY")]
    pub min_speed_y: [f32; 2],
    #[serde(rename = "sightDist")]
    pub sight_dist: f32,
    #[serde(rename = "accelRate")]
    pub accel_rate: f32,
    pub drag: f32,
    #[serde(rename = "fluidDrag")]
    pub fluid_drag: f32,
    pub scale: f32,
    pub size: f32,
    #[serde(rename = "hotSpot")]
    pub hot_spot: [f32; 4],
    pub collision: [f32; 4],
    #[serde(rename = "spawnType")]
    pub spawn_type: String,
    #[serde(rename = "animPath")]
    pub anim_path: String,
    #[serde(rename = "hsvOffset")]
    pub hsv_offset: [f32; 3],
    pub reacts: Vec<ReactDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FrameInfo {
    pub path: String,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Deserialize)]
pub struct RawGameData {
    pub manifest: Vec<ManifestEntry>,
    pub stages: HashMap<String, StageDef>,
    #[serde(rename = "fishConfigs")]
    pub fish_configs: HashMap<String, FishConfig>,
    pub sprites: HashMap<String, HashMap<String, Vec<FrameInfo>>>,
}

#[derive(Debug, Serialize)]
pub struct InitPayload<'a> {
    pub manifest: &'a [ManifestEntry],
    pub images: &'a [String],
}

pub struct GameData {
    pub manifest: Vec<ManifestEntry>,
    pub stages: HashMap<String, StageDef>,
    pub fish_configs: HashMap<String, FishConfig>,
    pub sprites: HashMap<String, HashMap<String, Vec<FrameInfo>>>,
    pub image_paths: Vec<String>,
    pub image_ids: HashMap<String, usize>,
}

impl GameData {
    pub fn load() -> Self {
        let raw: RawGameData =
            serde_json::from_str(include_str!("game_data.json")).expect("valid game_data.json");
        let mut image_paths = Vec::new();
        let mut image_ids = HashMap::new();

        let mut reg = |p: &str| {
            if !p.is_empty() && !image_ids.contains_key(p) {
                let id = image_paths.len();
                image_paths.push(p.to_string());
                image_ids.insert(p.to_string(), id);
            }
        };

        for st in raw.stages.values() {
            for img in st
                .bg_images
                .iter()
                .chain(st.mg_images.iter())
                .chain(st.fg_images.iter())
            {
                reg(&img.file);
            }
        }
        for states in raw.sprites.values() {
            for frames in states.values() {
                for fr in frames {
                    reg(&fr.path);
                }
            }
        }

        Self {
            manifest: raw.manifest,
            stages: raw.stages,
            fish_configs: raw.fish_configs,
            sprites: raw.sprites,
            image_paths,
            image_ids,
        }
    }

    pub fn image_id(&self, path: &str) -> Option<usize> {
        self.image_ids.get(path).copied()
    }
}
