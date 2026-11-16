use crate::data::{FishConfig, FrameInfo, GameData, InitPayload, StageDef};
use crate::rng::Rng;
use serde::Serialize;

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameState {
    Title,
    Playing,
    Paused,
    StageClear,
    GameOver,
    GameWon,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FishKind {
    Normal,
    Chaser,
    Barracuda,
    Shark,
    Puffer,
    Jellyfish,
    Mine,
    Oyster,
    Poison,
    Mermaid,
    GoldenMinnow,
    BrineShrimp,
    Bonus(BonusKind),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BonusKind {
    DoublePoints,
    FeedingFury,
    FreeLife,
    Shield,
    Speed,
    Star,
    Stun,
}

impl FishKind {
    fn from_class(cls: &str) -> Self {
        match cls {
            "ChaserFish" => Self::Chaser,
            "Barracuda" => Self::Barracuda,
            "Shark" => Self::Shark,
            "PufferFish" => Self::Puffer,
            "BasicJellyFish" => Self::Jellyfish,
            "Mine" => Self::Mine,
            "Oyster" => Self::Oyster,
            "PoisonMinnow" | "SuicideFish" => Self::Poison,
            "Mermaid" => Self::Mermaid,
            "GoldenMinnow" => Self::GoldenMinnow,
            "BrineShrimp" => Self::BrineShrimp,
            "BonusBubble2X" => Self::Bonus(BonusKind::DoublePoints),
            "BonusBubbleFeedingFury" => Self::Bonus(BonusKind::FeedingFury),
            "BonusBubbleFreeLife" => Self::Bonus(BonusKind::FreeLife),
            "BonusBubbleShield" => Self::Bonus(BonusKind::Shield),
            "BonusBubbleSpeed" => Self::Bonus(BonusKind::Speed),
            "BonusBubbleStar" => Self::Bonus(BonusKind::Star),
            "BonusBubbleStun" => Self::Bonus(BonusKind::Stun),
            _ => Self::Normal,
        }
    }

    fn is_hazard(&self) -> bool {
        matches!(self, Self::Mine | Self::Jellyfish | Self::Poison)
    }
}

#[derive(Clone, Copy)]
struct Rect {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl Rect {
    fn intersects(&self, o: &Rect) -> bool {
        self.x1 < o.x2 && self.x2 > o.x1 && self.y1 < o.y2 && self.y2 > o.y1
    }
}

#[derive(Clone)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub facing_right: bool,
    pub turn_timer: f32,
    pub eat_timer: f32,
    pub dash_timer: f32,
    pub dash_cooldown: f32,
    pub invuln_timer: f32,
    pub stun_timer: f32,
    pub poison_timer: f32,
    pub shield: bool,
    pub speed_boost_timer: f32,
    pub double_pts_timer: f32,
    pub suck_timer: f32,
    pub size: f32,
    pub scale: f32,
    pub tier_idx: usize,
    pub food_progress: f32,
    pub anim_time: f32,
    pub config_key: String,
    pub is_john_dory: bool,
    pub lives: i32,
    pub hue_shift: f32,
}

impl Player {
    fn new_at(x: f32, y: f32, size: f32, scale: f32, config_key: String, lives: i32, hue_shift: f32) -> Self {
        let is_jd = config_key.contains("johndory");
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            facing_right: true,
            turn_timer: 0.0,
            eat_timer: 0.0,
            dash_timer: 0.0,
            dash_cooldown: 0.0,
            invuln_timer: 2.2,
            stun_timer: 0.0,
            poison_timer: 0.0,
            shield: false,
            speed_boost_timer: 0.0,
            double_pts_timer: 0.0,
            suck_timer: 0.0,
            size,
            scale,
            tier_idx: 0,
            food_progress: 0.0,
            anim_time: 0.0,
            config_key,
            is_john_dory: is_jd,
            lives,
            hue_shift,
        }
    }
}

pub struct Enemy {
    pub alive: bool,
    pub template_idx: usize,
    pub kind: FishKind,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub facing_right: bool,
    pub anim_time: f32,
    pub state_timer: f32,
    pub move_timer: f32,
    pub target_vx: f32,
    pub target_vy: f32,
    pub inflated: bool,
    pub pearl_open: bool,
    pub pearl_taken: bool,
    pub stunned_timer: f32,
    pub being_eaten_timer: f32,
    pub eaten_by_idx: usize,
    pub school_id: u32,
    pub tail_cooldown: f32,
}

pub struct Spawner {
    pub timer: f32,
    pub warned: bool,
}

#[derive(Serialize, Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub r: f32,
    pub life: f32,
    pub max_life: f32,
    pub kind: u8,
}

#[derive(Serialize, Clone)]
pub struct Popup {
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub text: String,
    pub color: String,
    pub life: f32,
    pub max_life: f32,
    pub scale: f32,
}

#[derive(Serialize, Clone)]
pub struct WarningMarker {
    pub x: f32,
    pub y: f32,
    pub right_side: bool,
}

#[derive(Serialize, Clone)]
pub struct EdibleIcon {
    pub image: String,
    pub hue: f32,
    pub name: String,
}

#[derive(Serialize, Clone)]
pub struct PlayerTag {
    pub x: f32,
    pub y: f32,
    pub label: String,
    pub color: String,
    pub alive: bool,
}

#[derive(Serialize)]
pub struct BuffStatus {
    pub shield: bool,
    pub speed: f32,
    pub double_pts: f32,
    pub poison: f32,
    pub stunned: f32,
}

#[derive(Serialize)]
pub struct HudPayload {
    pub state: GameState,
    pub stage_idx: usize,
    pub stage_count: usize,
    pub stage_name: String,
    pub character: String,
    pub stage_width: f32,
    pub stage_height: f32,
    pub score: u32,
    pub high_score: u32,
    pub lives: i32,
    pub growth_progress: f32,
    pub tier_idx: usize,
    pub tier_count: usize,
    pub tier_marks: Vec<f32>,
    pub frenzy: f32,
    pub frenzy_mult: u32,
    pub time_remaining: f32,
    pub timed_mode: bool,
    pub buffs: BuffStatus,
    pub fish_eaten: u32,
    pub player_is_jd: bool,
    pub coop_count: usize,
    pub player_tags: Vec<PlayerTag>,
    pub particles: Vec<Particle>,
    pub popups: Vec<Popup>,
    pub warnings: Vec<WarningMarker>,
    pub sounds: Vec<String>,
    pub music: String,
    pub edible_icons: Vec<EdibleIcon>,
}

#[derive(Clone, Default)]
pub struct InputState {
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub mouse_active: bool,
    pub key_mask: u32,
    pub dash_just_pressed: bool,
    pub special_just_pressed: bool,
}

pub struct Engine {
    pub data: GameData,
    pub rng: Rng,
    pub state: GameState,
    pub stage_idx: usize,
    pub timed_mode: bool,
    pub is_bonus_stage: bool,
    pub score: u32,
    pub high_score: u32,
    pub fish_eaten: u32,
    pub frenzy: f32,
    pub frenzy_mult: u32,
    pub time_remaining: f32,
    pub bonus_spawn_timer: f32,
    pub ambient_bubble_timer: f32,
    pub stage_clear_timer: f32,
    pub next_school_id: u32,
    pub team_food_progress: f32,
    pub coop_fish_picks: [Option<usize>; 4],

    pub players: Vec<Player>,
    pub inputs: Vec<InputState>,
    pub enemies: Vec<Enemy>,
    pub spawners: Vec<Spawner>,
    pub particles: Vec<Particle>,
    pub popups: Vec<Popup>,
    pub warnings: Vec<WarningMarker>,
    pub sounds: Vec<String>,
    pub current_music: String,

    pub frame_buf: Vec<f32>,
    pub hud_json: String,
    pub init_json: String,
}

impl Engine {
    pub fn new(seed: u32) -> Self {
        let data = GameData::load();
        let init_payload = InitPayload {
            manifest: &data.manifest,
            images: &data.image_paths,
        };
        let init_json = serde_json::to_string(&init_payload).unwrap_or_default();

        let mut engine = Self {
            data,
            rng: Rng::new(seed),
            state: GameState::Title,
            stage_idx: 0,
            timed_mode: false,
            is_bonus_stage: false,
            score: 0,
            high_score: 0,
            fish_eaten: 0,
            frenzy: 0.0,
            frenzy_mult: 1,
            time_remaining: 0.0,
            bonus_spawn_timer: 9.0,
            ambient_bubble_timer: 0.25,
            stage_clear_timer: 0.0,
            next_school_id: 1,
            team_food_progress: 0.0,
            coop_fish_picks: [None; 4],
            players: vec![Player::new_at(400.0, 300.0, 1.5, 0.4, String::new(), 3, 0.0)],
            inputs: vec![InputState {
                mouse_x: 400.0,
                mouse_y: 300.0,
                ..Default::default()
            }],
            enemies: Vec::new(),
            spawners: Vec::new(),
            particles: Vec::new(),
            popups: Vec::new(),
            warnings: Vec::new(),
            sounds: Vec::new(),
            current_music: "assets/resources/music/menuMusic.wav".to_string(),
            frame_buf: Vec::with_capacity(1024),
            hud_json: String::new(),
            init_json,
        };
        engine.build_hud();
        engine
    }

    fn current_stage(&self) -> Option<&StageDef> {
        let entry = self.data.manifest.get(self.stage_idx)?;
        self.data.stages.get(&entry.path)
    }

    pub fn start_stage(&mut self, stage_idx: usize, timed_mode: bool, reset_run: bool, player_count: usize) {
        let idx = stage_idx.min(self.data.manifest.len().saturating_sub(1));
        self.stage_idx = idx;
        self.timed_mode = timed_mode;
        let pcount = player_count.clamp(1, 4);

        if reset_run {
            self.score = 0;
            self.fish_eaten = 0;
        }
        self.frenzy = 0.0;
        self.frenzy_mult = 1;
        self.bonus_spawn_timer = 8.0;
        self.stage_clear_timer = 0.0;
        self.team_food_progress = 0.0;
        self.enemies.clear();
        self.spawners.clear();
        self.particles.clear();
        self.popups.clear();
        self.warnings.clear();
        self.sounds.clear();

        let (sw, sh, pcfg_key, first_size, first_scale, time_limit, stage_type, music_path, enemy_count) = {
            let Some(st) = self.current_stage() else {
                return;
            };
            let tier0 = st.player_sizes.first();
            (
                st.stage_width,
                st.stage_height,
                st.player_config.clone(),
                tier0.map(|t| t.size).unwrap_or(1.5),
                tier0.map(|t| t.scale).unwrap_or(0.5),
                if st.time > 0.0 { st.time } else { 90.0 },
                st.stage_type.clone(),
                st.music.clone(),
                st.enemies.len(),
            )
        };

        self.is_bonus_stage = stage_type == "timedBonus";
        self.time_remaining = if timed_mode || self.is_bonus_stage {
            time_limit
        } else {
            0.0
        };

        let hues = [0.0_f32, 125.0, 215.0, 305.0];
        let old_lives: Vec<i32> = self.players.iter().map(|p| p.lives.max(1)).collect();
        self.players.clear();
        self.inputs.clear();

        for i in 0..pcount {
            let px = sw * (0.35 + (i as f32) * 0.12);
            let py = sh * (0.45 + ((i % 2) as f32) * 0.12);
            let lives = if reset_run {
                3
            } else {
                *old_lives.get(i).unwrap_or(&3)
            };
            let cfg_for_p = if pcount > 1 {
                let fish_keys = [
                    "config/fish/angelplayer.xml",
                    "config/fish/lionplayer.xml",
                    "config/fish/anglerplayer.xml",
                    "config/fish/jdplayer.xml",
                    "config/fish/orcaplayer.xml",
                ];
                self.coop_fish_picks
                    .get(i)
                    .and_then(|opt| *opt)
                    .and_then(|fi| fish_keys.get(fi))
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| pcfg_key.clone())
            } else {
                pcfg_key.clone()
            };
            self.players.push(Player::new_at(
                px,
                py,
                first_size,
                first_scale,
                cfg_for_p,
                lives,
                hues[i],
            ));
            self.inputs.push(InputState {
                mouse_x: px,
                mouse_y: py,
                ..Default::default()
            });
        }

        for i in 0..enemy_count {
            self.spawners.push(Spawner {
                timer: 0.15 + (i as f32) * 0.2,
                warned: false,
            });
        }

        self.state = GameState::Playing;
        self.sounds.push("stageIntro.wav".to_string());
        if !music_path.is_empty() {
            self.current_music = format!("assets/{}", music_path);
        }
        self.spawn_initial_fish();
        self.build_frame();
        self.build_hud();
    }

    fn spawn_initial_fish(&mut self) {
        let count = self.spawners.len();
        for i in 0..count {
            self.try_spawn_for_spawner(i, true);
        }
    }

    pub fn set_player_fish(&mut self, player_idx: usize, fish_idx: usize) {
        if player_idx < 4 && fish_idx < 5 {
            self.coop_fish_picks[player_idx] = Some(fish_idx);
        }
    }

    pub fn set_player_input(
        &mut self,
        player_idx: usize,
        mouse_x: f32,
        mouse_y: f32,
        mouse_active: bool,
        key_mask: u32,
        dash_pressed: bool,
        special_pressed: bool,
    ) {
        if let Some(inp) = self.inputs.get_mut(player_idx) {
            inp.mouse_x = mouse_x;
            inp.mouse_y = mouse_y;
            inp.mouse_active = mouse_active;
            inp.key_mask = key_mask;
            if dash_pressed {
                inp.dash_just_pressed = true;
            }
            if special_pressed {
                inp.special_just_pressed = true;
            }
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            GameState::Playing => self.state = GameState::Paused,
            GameState::Paused => self.state = GameState::Playing,
            _ => {}
        }
        self.build_hud();
    }

    pub fn tick(&mut self, dt_raw: f32) {
        self.sounds.clear();
        let dt = dt_raw.clamp(0.0, 0.05);

        match self.state {
            GameState::Playing => self.update_playing(dt),
            GameState::StageClear => {
                self.stage_clear_timer += dt;
                self.update_particles_and_popups(dt);
                for p in &mut self.players {
                    p.anim_time += dt;
                }
            }
            _ => self.update_particles_and_popups(dt),
        }

        for inp in &mut self.inputs {
            inp.dash_just_pressed = false;
            inp.special_just_pressed = false;
        }

        self.build_frame();
        self.build_hud();
    }

    fn update_playing(&mut self, dt: f32) {
        let Some(st) = self.current_stage().cloned() else {
            return;
        };

        if self.timed_mode || self.is_bonus_stage {
            self.time_remaining -= dt;
            if self.time_remaining <= 0.0 {
                self.time_remaining = 0.0;
                if self.is_bonus_stage {
                    self.sounds.push("stageClear.wav".to_string());
                    self.stage_clear_timer = 0.0;
                    self.state = if self.stage_idx + 1 >= self.data.manifest.len() {
                        GameState::GameWon
                    } else {
                        GameState::StageClear
                    };
                } else {
                    self.state = GameState::GameOver;
                    self.sounds.push("GameOver.wav".to_string());
                }
                return;
            }
        }

        if self.frenzy > 0.0 {
            self.frenzy = (self.frenzy - dt * 0.075).max(0.0);
        }
        self.frenzy_mult = if self.frenzy >= 0.85 {
            3
        } else if self.frenzy >= 0.5 {
            2
        } else {
            1
        };

        self.ambient_bubble_timer -= dt;
        if self.ambient_bubble_timer <= 0.0 {
            self.ambient_bubble_timer = self.rng.range(0.18, 0.45);
            let bx = self.rng.range(20.0, st.stage_width - 20.0);
            self.particles.push(Particle {
                x: bx,
                y: st.stage_height + 10.0,
                vx: self.rng.range(-10.0, 10.0),
                vy: self.rng.range(-90.0, -50.0),
                r: self.rng.range(2.5, 6.5),
                life: 0.0,
                max_life: self.rng.range(4.0, 7.0),
                kind: 0,
            });
        }

        for idx in 0..self.players.len() {
            if self.players[idx].lives > 0 {
                self.update_player(idx, dt, &st);
            }
        }
        self.update_spawners(dt, &st);
        self.update_enemies(dt, &st);
        self.check_collisions(&st);
        self.update_particles_and_popups(dt);
    }

    fn update_player(&mut self, pidx: usize, dt: f32, st: &StageDef) {
        let inp = self.inputs.get(pidx).cloned().unwrap_or_default();
        let pcfg = self.data.fish_configs.get(&self.players[pidx].config_key).cloned();
        let base_max_x = pcfg.as_ref().map(|c| c.max_speed_x[1]).unwrap_or(345.0);
        let base_max_y = pcfg.as_ref().map(|c| c.max_speed_y[1]).unwrap_or(345.0);
        let drag = pcfg.as_ref().map(|c| c.drag).unwrap_or(7.0);

        let p = &mut self.players[pidx];
        p.anim_time += dt;
        if p.turn_timer > 0.0 {
            p.turn_timer = (p.turn_timer - dt).max(0.0);
        }
        if p.eat_timer > 0.0 {
            p.eat_timer = (p.eat_timer - dt).max(0.0);
        }
        if p.invuln_timer > 0.0 {
            p.invuln_timer = (p.invuln_timer - dt).max(0.0);
        }
        if p.stun_timer > 0.0 {
            p.stun_timer = (p.stun_timer - dt).max(0.0);
        }
        if p.poison_timer > 0.0 {
            p.poison_timer = (p.poison_timer - dt).max(0.0);
        }
        if p.speed_boost_timer > 0.0 {
            p.speed_boost_timer = (p.speed_boost_timer - dt).max(0.0);
        }
        if p.double_pts_timer > 0.0 {
            p.double_pts_timer = (p.double_pts_timer - dt).max(0.0);
        }
        if p.dash_timer > 0.0 {
            p.dash_timer = (p.dash_timer - dt).max(0.0);
        }
        if p.dash_cooldown > 0.0 {
            p.dash_cooldown = (p.dash_cooldown - dt).max(0.0);
        }
        if p.suck_timer > 0.0 {
            p.suck_timer = (p.suck_timer - dt).max(0.0);
        }

        let bonus_speed_mult = if p.speed_boost_timer > 0.0 { 1.8 } else { 1.0 };
        let mut spawn_dash_bubbles = None;

        if p.stun_timer <= 0.0 {
            if inp.dash_just_pressed && p.dash_cooldown <= 0.0 {
                p.dash_timer = 0.25;
                p.dash_cooldown = 0.4;
                let (dir_x, dir_y) = if p.vx.abs() > 15.0 || p.vy.abs() > 15.0 {
                    let l = (p.vx * p.vx + p.vy * p.vy).sqrt();
                    (p.vx / l, p.vy / l)
                } else if p.facing_right {
                    (1.0, 0.0)
                } else {
                    (-1.0, 0.0)
                };
                p.vx = dir_x * base_max_x * 2.2 * bonus_speed_mult;
                p.vy = dir_y * base_max_y * 2.2 * bonus_speed_mult;
                self.sounds.push("playerDash.wav".to_string());
                spawn_dash_bubbles = Some((p.x, p.y, dir_x));
            }

            if p.is_john_dory && inp.special_just_pressed && p.suck_timer <= 0.0 {
                p.suck_timer = 1.4;
                self.sounds.push("jdSuckSound.wav".to_string());
            }

            let poison_sign = if p.poison_timer > 0.0 { -1.0 } else { 1.0 };
            let key_dir_x: f32 = ((if inp.key_mask & 8 != 0 { 1.0 } else { 0.0 })
                - (if inp.key_mask & 4 != 0 { 1.0 } else { 0.0 }))
                * poison_sign;
            let key_dir_y: f32 = ((if inp.key_mask & 2 != 0 { 1.0 } else { 0.0 })
                - (if inp.key_mask & 1 != 0 { 1.0 } else { 0.0 }))
                * poison_sign;

            if key_dir_x != 0.0 || key_dir_y != 0.0 {
                let len = (key_dir_x * key_dir_x + key_dir_y * key_dir_y).sqrt().max(1.0);
                let target_vx = (key_dir_x / len) * base_max_x * bonus_speed_mult;
                let target_vy = (key_dir_y / len) * base_max_y * bonus_speed_mult;
                let blend = (dt * 14.0).min(1.0);
                p.vx += (target_vx - p.vx) * blend;
                p.vy += (target_vy - p.vy) * blend;
            } else if inp.mouse_active {
                let dx = (inp.mouse_x - p.x) * poison_sign;
                let dy = (inp.mouse_y - p.y) * poison_sign;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > 2.0 {
                    let speed_factor = (dist / 26.0).clamp(0.0, 1.0);
                    let target_vx = (dx / dist) * base_max_x * bonus_speed_mult * speed_factor;
                    let target_vy = (dy / dist) * base_max_y * bonus_speed_mult * speed_factor;
                    let blend = (dt * 18.0).min(1.0);
                    p.vx += (target_vx - p.vx) * blend;
                    p.vy += (target_vy - p.vy) * blend;
                } else {
                    p.vx *= (-14.0 * dt).exp();
                    p.vy *= (-14.0 * dt).exp();
                }
            } else {
                let damping = (-drag * dt).exp();
                p.vx *= damping;
                p.vy *= damping;
            }
        } else {
            let damping = (-drag * dt).exp();
            p.vx *= damping;
            p.vy *= damping;
        }

        let dash_cap = if p.dash_timer > 0.0 { 3.0 } else { 1.0 };
        p.vx = p.vx.clamp(
            -base_max_x * bonus_speed_mult * dash_cap,
            base_max_x * bonus_speed_mult * dash_cap,
        );
        p.vy = p.vy.clamp(
            -base_max_y * bonus_speed_mult * dash_cap,
            base_max_y * bonus_speed_mult * dash_cap,
        );

        p.x += p.vx * dt;
        p.y += p.vy * dt;

        if p.vx > 22.0 && !p.facing_right {
            p.facing_right = true;
            p.turn_timer = 0.22;
        } else if p.vx < -22.0 && p.facing_right {
            p.facing_right = false;
            p.turn_timer = 0.22;
        }

        let margin_x = 26.0 * p.scale;
        let margin_y = 22.0 * p.scale;
        p.x = p.x.clamp(margin_x, st.stage_width - margin_x);
        p.y = p.y.clamp(margin_y + 106.0, st.stage_height - margin_y);

        if let Some((bx, by, dir_x)) = spawn_dash_bubbles {
            for _ in 0..6 {
                self.particles.push(Particle {
                    x: bx - dir_x * 18.0,
                    y: by + self.rng.range(-10.0, 10.0),
                    vx: -dir_x * self.rng.range(60.0, 140.0),
                    vy: self.rng.range(-25.0, 25.0),
                    r: self.rng.range(2.5, 5.5),
                    life: 0.0,
                    max_life: 0.55,
                    kind: 0,
                });
            }
        }
    }

    fn max_player_size(&self) -> f32 {
        self.players
            .iter()
            .filter(|p| p.lives > 0)
            .map(|p| p.size)
            .fold(1.5_f32, f32::max)
    }

    fn update_spawners(&mut self, dt: f32, st: &StageDef) {
        for i in 0..self.spawners.len() {
            self.spawners[i].timer -= dt;
            if self.spawners[i].timer <= 0.0 {
                self.try_spawn_for_spawner(i, false);
            }
        }

        self.bonus_spawn_timer -= dt;
        if self.bonus_spawn_timer <= 0.0 {
            self.bonus_spawn_timer = self.rng.range(11.0, 17.0);
            let bonus_keys = [
                "config/fish/bonusbubble2x.xml",
                "config/fish/bonusbubblespeed.xml",
                "config/fish/bonusbubbleshield.xml",
                "config/fish/bonusbubblestar.xml",
                "config/fish/bonusbubblestun.xml",
                "config/fish/bonusbubblefeedingfury.xml",
                "config/fish/bonusbubblefreelife.xml",
            ];
            let pick = (self.rng.next_u32() as usize) % bonus_keys.len();
            if let Some(cfg) = self.data.fish_configs.get(bonus_keys[pick]).cloned() {
                self.enemies.push(Enemy {
                    alive: true,
                    template_idx: 1000 + pick,
                    kind: FishKind::from_class(&cfg.class),
                    x: self.rng.range(60.0, st.stage_width - 60.0),
                    y: st.stage_height + 30.0,
                    vx: self.rng.range(-12.0, 12.0),
                    vy: -self.rng.range(55.0, 80.0),
                    facing_right: true,
                    anim_time: 0.0,
                    state_timer: 0.0,
                    move_timer: 1.0,
                    target_vx: 0.0,
                    target_vy: -65.0,
                    inflated: false,
                    pearl_open: false,
                    pearl_taken: false,
                    stunned_timer: 0.0,
                    being_eaten_timer: 0.0,
                    eaten_by_idx: 0,
                    school_id: 0,
                    tail_cooldown: 0.0,
                });
            }
        }
    }

    fn enemy_cfg<'a>(&'a self, template_idx: usize, st: &'a StageDef) -> Option<&'a FishConfig> {
        if template_idx >= 1000 {
            let bonus_keys = [
                "config/fish/bonusbubble2x.xml",
                "config/fish/bonusbubblespeed.xml",
                "config/fish/bonusbubbleshield.xml",
                "config/fish/bonusbubblestar.xml",
                "config/fish/bonusbubblestun.xml",
                "config/fish/bonusbubblefeedingfury.xml",
                "config/fish/bonusbubblefreelife.xml",
            ];
            let key = bonus_keys.get(template_idx - 1000)?;
            self.data.fish_configs.get(*key)
        } else {
            let edef = st.enemies.get(template_idx)?;
            self.data.fish_configs.get(&edef.template)
        }
    }

    fn try_spawn_for_spawner(&mut self, spawner_idx: usize, initial: bool) {
        let Some(st) = self.current_stage().cloned() else {
            return;
        };
        let Some(edef) = st.enemies.get(spawner_idx) else {
            return;
        };
        let Some(cfg) = self.data.fish_configs.get(&edef.template) else {
            return;
        };

        let psize = self.max_player_size();
        let mut active_info = edef.spawn_info.first();
        for si in &edef.spawn_info {
            if psize >= si.player_size {
                active_info = Some(si);
            }
        }
        let Some(info) = active_info else {
            return;
        };

        let alive_count = self
            .enemies
            .iter()
            .filter(|e| e.alive && e.template_idx == spawner_idx)
            .count();

        let freq = self.rng.range(info.respawn_freq[0], info.respawn_freq[1]).max(0.25);
        self.spawners[spawner_idx].timer = freq;

        if alive_count >= info.max_count {
            return;
        }

        let kind = FishKind::from_class(&cfg.class);
        if matches!(kind, FishKind::Barracuda | FishKind::Shark) && !self.spawners[spawner_idx].warned {
            self.spawners[spawner_idx].warned = true;
            self.sounds.push("BarracudaWarn.wav".to_string());
        }

        let is_school = edef.template.contains("school");
        let school_count = if is_school && !initial {
            (info.max_count - alive_count).clamp(1, 5)
        } else {
            1
        };
        let sid = if is_school {
            let id = self.next_school_id;
            self.next_school_id += 1;
            id
        } else {
            0
        };

        let facing_right = self.rng.chance(0.5);
        let speed_x = self
            .rng
            .range(cfg.max_speed_x[0].max(45.0), cfg.max_speed_x[1].max(65.0));
        let base_y = self.rng.range(116.0, st.stage_height - 46.0);

        for k in 0..school_count {
            let (x, y, vx, vy) = match kind {
                FishKind::Mine => (
                    self.rng.range(50.0, st.stage_width - 50.0),
                    -30.0,
                    0.0,
                    self.rng.range(42.0, 68.0),
                ),
                FishKind::Oyster => (
                    self.rng.range(90.0, st.stage_width - 90.0),
                    st.stage_height - 48.0,
                    0.0,
                    0.0,
                ),
                FishKind::Jellyfish => (
                    self.rng.range(50.0, st.stage_width - 50.0),
                    if initial {
                        self.rng.range(80.0, st.stage_height - 80.0)
                    } else {
                        st.stage_height + 35.0
                    },
                    self.rng.range(-15.0, 15.0),
                    -self.rng.range(40.0, 62.0),
                ),
                _ => {
                    let offset_x = (k as f32) * 32.0;
                    let offset_y = ((k % 3) as f32 - 1.0) * 18.0;
                    let sx = if initial {
                        self.rng.range(40.0, st.stage_width - 40.0)
                    } else if facing_right {
                        -60.0 - offset_x
                    } else {
                        st.stage_width + 60.0 + offset_x
                    };
                    let svx = if facing_right { speed_x } else { -speed_x };
                    (
                        sx,
                        (base_y + offset_y).clamp(112.0, st.stage_height - 40.0),
                        svx,
                        self.rng.range(-10.0, 10.0),
                    )
                }
            };

            self.enemies.push(Enemy {
                alive: true,
                template_idx: spawner_idx,
                kind,
                x,
                y,
                vx,
                vy,
                facing_right,
                anim_time: self.rng.range(0.0, 2.0),
                state_timer: self.rng.range(2.2, 5.0),
                move_timer: self.rng.range(1.2, 3.5),
                target_vx: vx,
                target_vy: vy,
                inflated: false,
                pearl_open: false,
                pearl_taken: false,
                stunned_timer: 0.0,
                being_eaten_timer: 0.0,
                eaten_by_idx: 0,
                school_id: sid,
                tail_cooldown: 0.0,
            });
        }
    }

    fn update_enemies(&mut self, dt: f32, st: &StageDef) {
        self.warnings.clear();
        let mut finished_eaten: Vec<(usize, u32)> = Vec::new();

        for idx in 0..self.enemies.len() {
            if !self.enemies[idx].alive {
                continue;
            }

            if self.enemies[idx].being_eaten_timer > 0.0 {
                let target_p = self
                    .players
                    .get(self.enemies[idx].eaten_by_idx)
                    .or_else(|| self.players.first());
                let (px, py) = target_p.map(|p| (p.x, p.y)).unwrap_or((400.0, 300.0));
                self.enemies[idx].being_eaten_timer -= dt;
                self.enemies[idx].x += (px - self.enemies[idx].x) * (dt * 18.0).min(1.0);
                self.enemies[idx].y += (py - self.enemies[idx].y) * (dt * 18.0).min(1.0);
                if self.enemies[idx].being_eaten_timer <= 0.0 {
                    self.enemies[idx].alive = false;
                    finished_eaten.push((self.enemies[idx].eaten_by_idx, self.enemies[idx].school_id));
                }
                continue;
            }

            let template_idx = self.enemies[idx].template_idx;
            let Some(cfg) = self.enemy_cfg(template_idx, st).cloned() else {
                self.enemies[idx].alive = false;
                continue;
            };

            // Find nearest living player
            let mut nearest_dx = 0.0_f32;
            let mut nearest_dy = 0.0_f32;
            let mut nearest_dist = f32::MAX;
            let mut nearest_size = 1.5_f32;
            let mut any_sucking = false;
            for p in &self.players {
                if p.lives <= 0 {
                    continue;
                }
                let dx = p.x - self.enemies[idx].x;
                let dy = p.y - self.enemies[idx].y;
                let d = (dx * dx + dy * dy).sqrt().max(1.0);
                if d < nearest_dist {
                    nearest_dist = d;
                    nearest_dx = dx;
                    nearest_dy = dy;
                    nearest_size = p.size;
                    any_sucking = p.suck_timer > 0.0;
                }
            }

            let e = &mut self.enemies[idx];
            e.anim_time += dt;
            if e.tail_cooldown > 0.0 {
                e.tail_cooldown = (e.tail_cooldown - dt).max(0.0);
            }
            if e.stunned_timer > 0.0 {
                e.stunned_timer = (e.stunned_timer - dt).max(0.0);
                e.x += e.vx * 0.1 * dt;
                e.y += e.vy * 0.1 * dt;
                continue;
            }

            if matches!(e.kind, FishKind::Barracuda | FishKind::Shark)
                && (e.x < 0.0 || e.x > st.stage_width)
            {
                self.warnings.push(WarningMarker {
                    x: if e.x < 0.0 { 26.0 } else { st.stage_width - 26.0 },
                    y: e.y.clamp(112.0, st.stage_height - 40.0),
                    right_side: e.x > st.stage_width,
                });
            }

            match e.kind {
                FishKind::Oyster => {
                    e.state_timer -= dt;
                    if e.state_timer <= 0.0 {
                        e.pearl_open = !e.pearl_open;
                        e.state_timer = if e.pearl_open { 2.8 } else { 3.5 };
                        if !e.pearl_open {
                            e.pearl_taken = false;
                        }
                    }
                }
                FishKind::Mine => {
                    e.y += e.vy * dt;
                    e.x += (e.anim_time * 2.0).sin() * 14.0 * dt;
                    if e.y > st.stage_height + 50.0 {
                        e.alive = false;
                    }
                }
                FishKind::Jellyfish => {
                    e.y += e.vy * dt;
                    e.x += (e.anim_time * 2.5).sin() * 20.0 * dt;
                    if e.y < -50.0 {
                        e.alive = false;
                    }
                }
                FishKind::Bonus(_) => {
                    e.y += e.vy * dt;
                    e.x += (e.anim_time * 3.0).sin() * 18.0 * dt;
                    if e.y < -50.0 {
                        e.alive = false;
                    }
                }
                _ => {
                    if e.kind == FishKind::Puffer {
                        e.state_timer -= dt;
                        if e.state_timer <= 0.0 {
                            e.inflated = !e.inflated;
                            e.state_timer = if e.inflated { 2.6 } else { 4.2 };
                        }
                    }

                    if any_sucking && cfg.size <= nearest_size && nearest_dist < 230.0 {
                        e.vx += (nearest_dx / nearest_dist) * 560.0 * dt;
                        e.vy += (nearest_dy / nearest_dist) * 560.0 * dt;
                    }

                    if nearest_dist < cfg.sight_dist {
                        if cfg.size > nearest_size
                            && matches!(e.kind, FishKind::Chaser | FishKind::Barracuda | FishKind::Shark)
                        {
                            let chase_spd = cfg.max_speed_x[1].max(130.0) * 1.15;
                            e.target_vx = (nearest_dx / nearest_dist) * chase_spd;
                            e.target_vy = (nearest_dy / nearest_dist) * chase_spd * 0.75;
                        } else if cfg.size <= nearest_size && !matches!(e.kind, FishKind::Poison) {
                            let run_spd = cfg.max_speed_x[1].max(95.0) * 1.15;
                            e.target_vx = -(nearest_dx / nearest_dist) * run_spd;
                            e.target_vy = -(nearest_dy / nearest_dist) * run_spd * 0.6;
                        }
                    } else {
                        e.move_timer -= dt;
                        if e.move_timer <= 0.0 {
                            e.move_timer = self.rng.range(1.8, 4.5);
                            e.target_vy = self.rng.range(-30.0, 30.0);
                        }
                    }

                    e.vx += (e.target_vx - e.vx) * (dt * 2.5).min(1.0);
                    e.vy += (e.target_vy - e.vy) * (dt * 2.5).min(1.0);
                    e.x += e.vx * dt;
                    e.y += e.vy * dt;

                    if e.y < 112.0 {
                        e.y = 112.0;
                        e.vy = e.vy.abs();
                        e.target_vy = e.target_vy.abs();
                    } else if e.y > st.stage_height - 32.0 {
                        e.y = st.stage_height - 32.0;
                        e.vy = -e.vy.abs();
                        e.target_vy = -e.target_vy.abs();
                    }

                    if e.vx > 8.0 {
                        e.facing_right = true;
                    } else if e.vx < -8.0 {
                        e.facing_right = false;
                    }

                    if e.x < -160.0 || e.x > st.stage_width + 160.0 {
                        e.alive = false;
                    }
                }
            }
        }

        for &(pidx, sid) in &finished_eaten {
            if sid != 0 && !self.enemies.iter().any(|e| e.alive && e.school_id == sid) {
                self.score += 500;
                if self.score > self.high_score {
                    self.high_score = self.score;
                }
                let (px, py) = self.players.get(pidx).map(|p| (p.x, p.y)).unwrap_or((400.0, 300.0));
                self.sounds.push("schoolBonus.wav".to_string());
                self.popups.push(Popup {
                    x: px,
                    y: py - 42.0,
                    vy: -28.0,
                    text: "SCHOOL BONUS +500!".to_string(),
                    color: "#ffea00".to_string(),
                    life: 0.0,
                    max_life: 1.4,
                    scale: 1.35,
                });
            }
        }

        self.enemies.retain(|e| e.alive);
    }

    fn sprite_dims(&self, anim_path: &str, state: &str) -> (f32, f32) {
        let key = anim_path.to_lowercase();
        if let Some(states) = self.data.sprites.get(&key) {
            if let Some(frames) = states.get(state).or_else(|| states.values().next()) {
                if let Some(f) = frames.first() {
                    return (f.w, f.h);
                }
            }
        }
        (80.0, 60.0)
    }

    fn hotspot_rect(x: f32, y: f32, facing_right: bool, scale: f32, hs: [f32; 4]) -> Rect {
        let (lx, rx) = if facing_right {
            (hs[0] * scale, hs[2] * scale)
        } else {
            (-hs[2] * scale, -hs[0] * scale)
        };
        let (ty, by) = (hs[1] * scale, hs[3] * scale);
        Rect {
            x1: x + lx.min(rx),
            y1: y + ty.min(by),
            x2: x + lx.max(rx),
            y2: y + ty.max(by),
        }
    }

    fn collision_rect(x: f32, y: f32, w: f32, h: f32, col: [f32; 4]) -> Rect {
        let left = x - w * 0.5;
        let top = y - h * 0.5;
        Rect {
            x1: left + w * (col[0] / 100.0),
            y1: top + h * (col[1] / 100.0),
            x2: left + w * (col[2] / 100.0),
            y2: top + h * (col[3] / 100.0),
        }
    }

    fn check_collisions(&mut self, st: &StageDef) {
        let mut stage_cleared = false;

        for pidx in 0..self.players.len() {
            if self.players[pidx].lives <= 0 {
                continue;
            }
            let pcfg = self
                .data
                .fish_configs
                .get(&self.players[pidx].config_key)
                .cloned();
            let p_anim = pcfg.as_ref().map(|c| c.anim_path.as_str()).unwrap_or("");
            let (pw_raw, ph_raw) = self.sprite_dims(p_anim, "swim");
            let p_scale = self.players[pidx].scale;
            let p_size = self.players[pidx].size;
            let px = self.players[pidx].x;
            let py = self.players[pidx].y;
            let p_facing = self.players[pidx].facing_right;
            let pw = pw_raw * p_scale;
            let ph = ph_raw * p_scale;
            let p_hs = pcfg.as_ref().map(|c| c.hot_spot).unwrap_or([-15.0, -24.0, 30.0, 25.0]);
            let p_col = pcfg.as_ref().map(|c| c.collision).unwrap_or([5.0, 19.0, 95.0, 90.0]);

            let p_mouth = Self::hotspot_rect(px, py, p_facing, p_scale, p_hs);
            let p_body = Self::collision_rect(px, py, pw, ph, p_col);

            let mut grow_to: Option<(usize, f32, f32)> = None;

            for i in 0..self.enemies.len() {
                if !self.enemies[i].alive || self.enemies[i].being_eaten_timer > 0.0 {
                    continue;
                }
                let Some(cfg) = self.enemy_cfg(self.enemies[i].template_idx, st).cloned() else {
                    continue;
                };
                let (ew_raw, eh_raw) = self.sprite_dims(&cfg.anim_path, "swim");
                let e_scale = if self.enemies[i].kind == FishKind::Puffer && self.enemies[i].inflated {
                    cfg.scale * 1.3
                } else {
                    cfg.scale
                };
                let ew = ew_raw * e_scale;
                let eh = eh_raw * e_scale;
                let e_body = Self::collision_rect(self.enemies[i].x, self.enemies[i].y, ew, eh, cfg.collision);
                let e_mouth = Self::hotspot_rect(
                    self.enemies[i].x,
                    self.enemies[i].y,
                    self.enemies[i].facing_right,
                    e_scale,
                    cfg.hot_spot,
                );

                match self.enemies[i].kind {
                    FishKind::Bonus(bkind) => {
                        if p_mouth.intersects(&e_body) || p_body.intersects(&e_body) {
                            self.enemies[i].alive = false;
                            self.apply_bonus(pidx, bkind, self.enemies[i].x, self.enemies[i].y);
                        }
                    }
                    FishKind::Oyster => {
                        if p_mouth.intersects(&e_body) || p_body.intersects(&e_body) {
                            if self.enemies[i].pearl_open && !self.enemies[i].pearl_taken {
                                self.enemies[i].pearl_taken = true;
                                let pts = 250 * self.frenzy_mult;
                                self.score += pts;
                                self.sounds.push("oysterPearl.wav".to_string());
                                self.popups.push(Popup {
                                    x: self.enemies[i].x,
                                    y: self.enemies[i].y - 25.0,
                                    vy: -45.0,
                                    text: format!("+{} PEARL!", pts),
                                    color: "#fff3a6".to_string(),
                                    life: 0.0,
                                    max_life: 1.1,
                                    scale: 1.2,
                                });
                            } else if !self.enemies[i].pearl_open && self.players[pidx].invuln_timer <= 0.0 {
                                self.sounds.push("oysterBite.wav".to_string());
                                self.handle_player_hit(pidx);
                            }
                        }
                    }
                    FishKind::Mine => {
                        if p_body.intersects(&e_body) {
                            self.enemies[i].alive = false;
                            self.sounds.push("mineExplode.wav".to_string());
                            self.spawn_burst(self.enemies[i].x, self.enemies[i].y, 2, 16);
                            if self.players[pidx].invuln_timer <= 0.0 {
                                self.handle_player_hit(pidx);
                            }
                        }
                    }
                    FishKind::Jellyfish => {
                        if p_body.intersects(&e_body)
                            && self.players[pidx].invuln_timer <= 0.0
                            && self.players[pidx].stun_timer <= 0.0
                        {
                            if self.players[pidx].shield {
                                self.players[pidx].shield = false;
                                self.players[pidx].invuln_timer = 1.5;
                                self.sounds.push("pufferBounce.wav".to_string());
                            } else {
                                self.players[pidx].stun_timer = 1.5;
                                self.players[pidx].invuln_timer = 1.7;
                                self.sounds.push("playerStunned.wav".to_string());
                                self.popups.push(Popup {
                                    x: self.players[pidx].x,
                                    y: self.players[pidx].y - 28.0,
                                    vy: -35.0,
                                    text: "STUNNED!".to_string(),
                                    color: "#b388ff".to_string(),
                                    life: 0.0,
                                    max_life: 1.0,
                                    scale: 1.1,
                                });
                            }
                        }
                    }
                    FishKind::Puffer if self.enemies[i].inflated => {
                        if p_body.intersects(&e_body) {
                            let dx = self.players[pidx].x - self.enemies[i].x;
                            let dy = self.players[pidx].y - self.enemies[i].y;
                            let d = (dx * dx + dy * dy).sqrt().max(1.0);
                            self.players[pidx].vx = (dx / d) * 440.0;
                            self.players[pidx].vy = (dy / d) * 440.0;
                            self.sounds.push("pufferBounce.wav".to_string());
                        }
                    }
                    FishKind::Poison => {
                        if p_mouth.intersects(&e_body) {
                            self.enemies[i].alive = false;
                            self.players[pidx].poison_timer = 4.0;
                            self.sounds.push("playerPoison.wav".to_string());
                            self.popups.push(Popup {
                                x: self.players[pidx].x,
                                y: self.players[pidx].y - 28.0,
                                vy: -35.0,
                                text: "POISONED!".to_string(),
                                color: "#76ff03".to_string(),
                                life: 0.0,
                                max_life: 1.1,
                                scale: 1.1,
                            });
                        }
                    }
                    _ => {
                        if cfg.size <= p_size {
                            if p_mouth.intersects(&e_body) {
                                self.enemies[i].being_eaten_timer = 0.16;
                                self.enemies[i].eaten_by_idx = pidx;
                                self.players[pidx].eat_timer = 0.24;
                                self.fish_eaten += 1;

                                let double_mult = if self.players[pidx].double_pts_timer > 0.0 { 2 } else { 1 };
                                let pts = (cfg.food_value as u32) * self.frenzy_mult * double_mult;
                                self.score += pts;
                                if self.score > self.high_score {
                                    self.high_score = self.score;
                                }
                                self.players[pidx].food_progress += cfg.food_value;
                                self.team_food_progress += cfg.food_value;

                                let prev_frenzy = self.frenzy;
                                self.frenzy = (self.frenzy + 0.16).min(1.0);
                                if prev_frenzy < 0.5 && self.frenzy >= 0.5 {
                                    self.sounds.push("feedingFrenzy.wav".to_string());
                                    self.popups.push(Popup {
                                        x: st.stage_width * 0.5,
                                        y: 130.0,
                                        vy: -18.0,
                                        text: "FEEDING FRENZY! 2X".to_string(),
                                        color: "#ffea00".to_string(),
                                        life: 0.0,
                                        max_life: 1.5,
                                        scale: 1.5,
                                    });
                                } else if prev_frenzy < 0.85 && self.frenzy >= 0.85 {
                                    self.sounds.push("doubleFrenzy.wav".to_string());
                                    self.popups.push(Popup {
                                        x: st.stage_width * 0.5,
                                        y: 130.0,
                                        vy: -18.0,
                                        text: "DOUBLE FRENZY! 3X".to_string(),
                                        color: "#ff6d00".to_string(),
                                        life: 0.0,
                                        max_life: 1.5,
                                        scale: 1.6,
                                    });
                                }

                                let sound = if self.rng.chance(0.5) {
                                    "playerEat.wav"
                                } else {
                                    "bite1.wav"
                                };
                                self.sounds.push(sound.to_string());
                                self.spawn_burst(self.enemies[i].x, self.enemies[i].y, 2, 5);
                                self.popups.push(Popup {
                                    x: self.enemies[i].x,
                                    y: self.enemies[i].y - 12.0,
                                    vy: -42.0,
                                    text: format!("+{}", pts),
                                    color: "#ffffff".to_string(),
                                    life: 0.0,
                                    max_life: 0.75,
                                    scale: 1.0,
                                });

                                for (ti, tier) in st.player_sizes.iter().enumerate() {
                                    if ti > self.players[pidx].tier_idx
                                        && self.players[pidx].food_progress >= tier.target_score
                                    {
                                        grow_to = Some((ti, tier.size, tier.scale));
                                    }
                                }
                                if !self.is_bonus_stage && self.team_food_progress >= st.next_level_at {
                                    stage_cleared = true;
                                }
                            }
                        } else if e_mouth.intersects(&p_body) && self.players[pidx].invuln_timer <= 0.0 {
                            self.handle_player_hit(pidx);
                        } else if self.enemies[i].kind == FishKind::Shark
                            && self.enemies[i].tail_cooldown <= 0.0
                            && p_mouth.intersects(&e_body)
                        {
                            self.enemies[i].tail_cooldown = 1.2;
                            let pts = 100 * self.frenzy_mult;
                            self.score += pts;
                            self.players[pidx].eat_timer = 0.2;
                            self.sounds.push("bite2.wav".to_string());
                            self.popups.push(Popup {
                                x: self.players[pidx].x,
                                y: self.players[pidx].y - 20.0,
                                vy: -40.0,
                                text: format!("+{} TAIL BITE!", pts),
                                color: "#ffab40".to_string(),
                                life: 0.0,
                                max_life: 0.9,
                                scale: 1.15,
                            });
                        }
                    }
                }
                if self.players[pidx].lives <= 0 || self.state != GameState::Playing {
                    break;
                }
            }

            if let Some((ti, new_size, new_scale)) = grow_to {
                self.players[pidx].tier_idx = ti;
                self.players[pidx].size = new_size;
                self.players[pidx].scale = new_scale;
                self.sounds.push("playerGrow.wav".to_string());
                self.spawn_burst(self.players[pidx].x, self.players[pidx].y, 1, 14);
                self.popups.push(Popup {
                    x: self.players[pidx].x,
                    y: self.players[pidx].y - 36.0,
                    vy: -30.0,
                    text: "GROW!".to_string(),
                    color: "#69f0ae".to_string(),
                    life: 0.0,
                    max_life: 1.2,
                    scale: 1.4,
                });
            }
        }

        // Co-Op: bigger players can eat smaller players!
        if self.players.len() > 1 && self.state == GameState::Playing {
            let mut eaten_pairs: Vec<(usize, usize)> = Vec::new();
            for a in 0..self.players.len() {
                if self.players[a].lives <= 0 {
                    continue;
                }
                let pcfg_a = self.data.fish_configs.get(&self.players[a].config_key);
                let hs_a = pcfg_a.map(|c| c.hot_spot).unwrap_or([-15.0, -24.0, 30.0, 25.0]);
                let mouth_a = Self::hotspot_rect(
                    self.players[a].x,
                    self.players[a].y,
                    self.players[a].facing_right,
                    self.players[a].scale,
                    hs_a,
                );
                for b in 0..self.players.len() {
                    if a == b || self.players[b].lives <= 0 || self.players[b].invuln_timer > 0.0 {
                        continue;
                    }
                    if self.players[a].size <= self.players[b].size {
                        continue;
                    }
                    let pcfg_b = self.data.fish_configs.get(&self.players[b].config_key);
                    let anim_b = pcfg_b.map(|c| c.anim_path.as_str()).unwrap_or("");
                    let (bw_raw, bh_raw) = self.sprite_dims(anim_b, "swim");
                    let col_b = pcfg_b.map(|c| c.collision).unwrap_or([5.0, 19.0, 95.0, 90.0]);
                    let body_b = Self::collision_rect(
                        self.players[b].x,
                        self.players[b].y,
                        bw_raw * self.players[b].scale,
                        bh_raw * self.players[b].scale,
                        col_b,
                    );
                    if mouth_a.intersects(&body_b) {
                        eaten_pairs.push((a, b));
                    }
                }
            }
            for (a, b) in eaten_pairs {
                if self.players[a].lives <= 0 || self.players[b].lives <= 0 || self.players[b].invuln_timer > 0.0 {
                    continue;
                }
                self.players[a].eat_timer = 0.25;
                let pts = 150 * self.frenzy_mult;
                self.score += pts;
                if self.score > self.high_score {
                    self.high_score = self.score;
                }
                self.frenzy = (self.frenzy + 0.2).min(1.0);
                self.sounds.push("playerEat.wav".to_string());
                self.popups.push(Popup {
                    x: self.players[b].x,
                    y: self.players[b].y - 18.0,
                    vy: -42.0,
                    text: format!("P{} ATE P{}! +{}", a + 1, b + 1, pts),
                    color: "#ff80ab".to_string(),
                    life: 0.0,
                    max_life: 1.2,
                    scale: 1.2,
                });
                self.handle_player_hit(b);
            }
        }

        if stage_cleared && self.state == GameState::Playing {
            self.sounds.push("stageClear.wav".to_string());
            self.stage_clear_timer = 0.0;
            self.state = if self.stage_idx + 1 >= self.data.manifest.len() {
                GameState::GameWon
            } else {
                GameState::StageClear
            };
        }
    }

    fn handle_player_hit(&mut self, pidx: usize) {
        if self.players[pidx].shield {
            self.players[pidx].shield = false;
            self.players[pidx].invuln_timer = 2.0;
            self.sounds.push("pufferBounce.wav".to_string());
            self.popups.push(Popup {
                x: self.players[pidx].x,
                y: self.players[pidx].y - 25.0,
                vy: -35.0,
                text: "SHIELD SAVED!".to_string(),
                color: "#40c4ff".to_string(),
                life: 0.0,
                max_life: 1.0,
                scale: 1.2,
            });
            return;
        }

        self.players[pidx].lives -= 1;
        self.sounds.push("playerDie.wav".to_string());
        self.spawn_burst(self.players[pidx].x, self.players[pidx].y, 0, 18);

        let all_dead = self.players.iter().all(|p| p.lives <= 0);
        if all_dead {
            self.state = GameState::GameOver;
            self.sounds.push("GameOver.wav".to_string());
        } else if self.players[pidx].lives > 0 {
            if let Some(st) = self.current_stage().cloned() {
                self.players[pidx].x = st.stage_width * 0.5;
                self.players[pidx].y = st.stage_height * 0.55;
            }
            self.players[pidx].vx = 0.0;
            self.players[pidx].vy = 0.0;
            self.players[pidx].invuln_timer = 2.6;
            self.players[pidx].poison_timer = 0.0;
            self.players[pidx].stun_timer = 0.0;
            self.sounds.push("playerSpawn.wav".to_string());
        }
    }

    fn apply_bonus(&mut self, pidx: usize, kind: BonusKind, x: f32, y: f32) {
        let label = match kind {
            BonusKind::DoublePoints => {
                self.players[pidx].double_pts_timer = 5.0;
                self.sounds.push("starPickup.wav".to_string());
                "2X POINTS!"
            }
            BonusKind::FeedingFury => {
                self.frenzy = 1.0;
                self.sounds.push("furyStart.wav".to_string());
                "FEEDING FURY!"
            }
            BonusKind::FreeLife => {
                self.players[pidx].lives += 1;
                self.sounds.push("freeLife.wav".to_string());
                "1-UP!"
            }
            BonusKind::Shield => {
                self.players[pidx].shield = true;
                self.sounds.push("starPickup.wav".to_string());
                "SHIELD!"
            }
            BonusKind::Speed => {
                self.players[pidx].speed_boost_timer = 4.0;
                self.sounds.push("speedStart.wav".to_string());
                "SPEED BOOST!"
            }
            BonusKind::Star => {
                self.score += 300;
                self.sounds.push("starPickup.wav".to_string());
                "+300 STAR!"
            }
            BonusKind::Stun => {
                for e in &mut self.enemies {
                    if e.alive {
                        e.stunned_timer = 6.0;
                    }
                }
                self.sounds.push("stunBubble.wav".to_string());
                "FREEZE!"
            }
        };
        self.spawn_burst(x, y, 1, 12);
        self.popups.push(Popup {
            x,
            y: y - 16.0,
            vy: -40.0,
            text: label.to_string(),
            color: "#80d8ff".to_string(),
            life: 0.0,
            max_life: 1.2,
            scale: 1.25,
        });
    }

    fn spawn_burst(&mut self, x: f32, y: f32, kind: u8, count: usize) {
        for _ in 0..count {
            let ang = self.rng.range(0.0, std::f32::consts::TAU);
            let spd = self.rng.range(35.0, 140.0);
            self.particles.push(Particle {
                x,
                y,
                vx: ang.cos() * spd,
                vy: ang.sin() * spd,
                r: self.rng.range(2.5, 6.0),
                life: 0.0,
                max_life: self.rng.range(0.4, 0.85),
                kind,
            });
        }
    }

    fn update_particles_and_popups(&mut self, dt: f32) {
        for pt in &mut self.particles {
            pt.life += dt;
            pt.x += pt.vx * dt;
            pt.y += pt.vy * dt;
        }
        self.particles.retain(|pt| pt.life < pt.max_life);

        for pp in &mut self.popups {
            pp.life += dt;
            pp.y += pp.vy * dt;
        }
        self.popups.retain(|pp| pp.life < pp.max_life);
    }

    fn pick_frame<'a>(
        &'a self,
        anim_path: &str,
        preferred_states: &[&str],
        anim_time: f32,
        fps: f32,
        looping: bool,
    ) -> Option<&'a FrameInfo> {
        let key = anim_path.to_lowercase();
        let states = self.data.sprites.get(&key)?;
        let mut frames = None;
        for st in preferred_states {
            if let Some(f) = states.get(*st) {
                if !f.is_empty() {
                    frames = Some(f);
                    break;
                }
            }
        }
        let frames = frames.or_else(|| states.values().find(|f| !f.is_empty()))?;
        let idx = if looping {
            ((anim_time * fps) as usize) % frames.len()
        } else {
            ((anim_time * fps) as usize).min(frames.len() - 1)
        };
        frames.get(idx)
    }

    fn push_cmd(
        &mut self,
        image_id: usize,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        flip_x: f32,
        rot: f32,
        alpha: f32,
        hue_deg: f32,
    ) {
        self.frame_buf.extend_from_slice(&[
            image_id as f32,
            x,
            y,
            w,
            h,
            flip_x,
            rot,
            alpha,
            hue_deg,
        ]);
    }

    fn build_frame(&mut self) {
        self.frame_buf.clear();
        let Some(st) = self.current_stage().cloned() else {
            return;
        };

        for img in st.bg_images.iter().chain(st.mg_images.iter()) {
            if let Some(id) = self.data.image_id(&img.file) {
                let w = img.w * img.scale_x.abs();
                let h = img.h * img.scale_y.abs();
                let flip = if img.scale_x < 0.0 { -1.0 } else { 1.0 };
                let cx = img.x + w * 0.5;
                let cy = img.y + h * 0.5;
                self.push_cmd(id, cx, cy, w, h, flip, 0.0, 1.0, 0.0);
            }
        }

        for idx in 0..self.enemies.len() {
            if !self.enemies[idx].alive {
                continue;
            }
            let template_idx = self.enemies[idx].template_idx;
            let kind = self.enemies[idx].kind;
            let inflated = self.enemies[idx].inflated;
            let pearl_open = self.enemies[idx].pearl_open;
            let anim_time = self.enemies[idx].anim_time;
            let facing_right = self.enemies[idx].facing_right;
            let ex = self.enemies[idx].x;
            let ey = self.enemies[idx].y;
            let eat_t = self.enemies[idx].being_eaten_timer;

            let Some(cfg) = self.enemy_cfg(template_idx, &st).cloned() else {
                continue;
            };
            let pref: &[&str] = match kind {
                FishKind::Puffer => {
                    if inflated {
                        &["inflate", "swim", "idle"]
                    } else {
                        &["swim", "idle"]
                    }
                }
                FishKind::Oyster => {
                    if pearl_open {
                        &["open", "idle", "swim"]
                    } else {
                        &["close", "idle", "swim"]
                    }
                }
                _ => &["swim", "idle"],
            };
            if let Some(fr) = self.pick_frame(&cfg.anim_path, pref, anim_time, 18.0, true).cloned() {
                if let Some(id) = self.data.image_id(&fr.path) {
                    let flip = if facing_right { -1.0 } else { 1.0 };
                    let shrink = if eat_t > 0.0 { (eat_t / 0.16).clamp(0.2, 1.0) } else { 1.0 };
                    let puf = if kind == FishKind::Puffer && inflated { 1.3 } else { 1.0 };
                    let w = fr.w * cfg.scale * puf * shrink;
                    let h = fr.h * cfg.scale * puf * shrink;
                    self.push_cmd(id, ex, ey, w, h, flip, 0.0, 1.0, cfg.hsv_offset[0]);
                }
            }
        }

        if self.state != GameState::Title {
            for pidx in 0..self.players.len() {
                let p = self.players[pidx].clone();
                if p.lives <= 0 {
                    continue;
                }
                if let Some(pcfg) = self.data.fish_configs.get(&p.config_key).cloned() {
                    let speed = (p.vx * p.vx + p.vy * p.vy).sqrt();
                    let pref: &[&str] = if p.eat_timer > 0.0 {
                        &["eat", "swim", "idle"]
                    } else if p.turn_timer > 0.0 {
                        &["turn", "swim", "idle"]
                    } else if speed > 28.0 {
                        &["swim", "idle"]
                    } else {
                        &["idle", "swim"]
                    };
                    if let Some(fr) = self
                        .pick_frame(&pcfg.anim_path, pref, p.anim_time, 20.0, true)
                        .cloned()
                    {
                        if let Some(id) = self.data.image_id(&fr.path) {
                            let flip = if p.facing_right { -1.0 } else { 1.0 };
                            let rot_sign = if p.facing_right { 1.0 } else { -1.0 };
                            let w = fr.w * p.scale;
                            let h = fr.h * p.scale;
                            let alpha = if p.invuln_timer > 0.0
                                && ((p.invuln_timer * 14.0) as i32) % 2 == 0
                            {
                                0.45
                            } else {
                                1.0
                            };
                            let hue = if p.poison_timer > 0.0 {
                                95.0
                            } else {
                                pcfg.hsv_offset[0] + p.hue_shift
                            };
                            self.push_cmd(
                                id,
                                p.x,
                                p.y,
                                w,
                                h,
                                flip,
                                (p.vy * 0.0007) * rot_sign,
                                alpha,
                                hue,
                            );
                        }
                    }
                }
            }
        }

        for img in &st.fg_images {
            if let Some(id) = self.data.image_id(&img.file) {
                let w = img.w * img.scale_x.abs();
                let h = img.h * img.scale_y.abs();
                let flip = if img.scale_x < 0.0 { -1.0 } else { 1.0 };
                let cx = img.x + w * 0.5;
                let cy = img.y + h * 0.5;
                self.push_cmd(id, cx, cy, w, h, flip, 0.0, 0.92, 0.0);
            }
        }
    }

    fn build_hud(&mut self) {
        let entry = self.data.manifest.get(self.stage_idx);
        let st = self.current_stage();
        let next_lvl = st.map(|s| s.next_level_at).unwrap_or(1000.0).max(1.0);
        let tier_marks = st
            .map(|s| {
                s.player_sizes
                    .iter()
                    .skip(1)
                    .filter(|t| t.target_score < next_lvl)
                    .map(|t| (t.target_score / next_lvl).clamp(0.0, 1.0))
                    .collect()
            })
            .unwrap_or_default();

        let p0 = self.players.first();
        let max_sz = self.max_player_size();

        let edible_icons = st
            .map(|s| {
                let mut seen = Vec::new();
                let mut out = Vec::new();
                for e in &s.enemies {
                    if let Some(cfg) = self.data.fish_configs.get(&e.template) {
                        if cfg.size <= max_sz
                            && !FishKind::from_class(&cfg.class).is_hazard()
                            && !seen.contains(&cfg.anim_path)
                        {
                            seen.push(cfg.anim_path.clone());
                            if let Some(fr) = self.pick_frame(&cfg.anim_path, &["idle", "swim"], 0.0, 1.0, true) {
                                out.push(EdibleIcon {
                                    image: fr.path.clone(),
                                    hue: cfg.hsv_offset[0],
                                    name: cfg.name.clone(),
                                });
                            }
                        }
                    }
                }
                out
            })
            .unwrap_or_default();

        let tag_colors = ["#ffeb3b", "#69f0ae", "#40c4ff", "#ff80ab"];
        let player_tags = if self.players.len() > 1 {
            self.players
                .iter()
                .enumerate()
                .map(|(i, p)| PlayerTag {
                    x: p.x,
                    y: p.y - 34.0 * p.scale - 10.0,
                    label: format!("P{}", i + 1),
                    color: tag_colors[i % tag_colors.len()].to_string(),
                    alive: p.lives > 0,
                })
                .collect()
        } else {
            Vec::new()
        };

        let payload = HudPayload {
            state: self.state,
            stage_idx: self.stage_idx,
            stage_count: self.data.manifest.len(),
            stage_name: st
                .map(|s| s.friendly_name.clone())
                .unwrap_or_else(|| "Feeding Frenzy".to_string()),
            character: entry.map(|e| e.character.clone()).unwrap_or_default(),
            stage_width: st.map(|s| s.stage_width).unwrap_or(800.0),
            stage_height: st.map(|s| s.stage_height).unwrap_or(600.0),
            score: self.score,
            high_score: self.high_score,
            lives: self.players.iter().map(|p| p.lives.max(0)).sum(),
            growth_progress: (self.team_food_progress / next_lvl).clamp(0.0, 1.0),
            tier_idx: p0.map(|p| p.tier_idx).unwrap_or(0),
            tier_count: st.map(|s| s.player_sizes.len()).unwrap_or(1),
            tier_marks,
            frenzy: self.frenzy,
            frenzy_mult: self.frenzy_mult,
            time_remaining: self.time_remaining,
            timed_mode: self.timed_mode || self.is_bonus_stage,
            buffs: BuffStatus {
                shield: p0.map(|p| p.shield).unwrap_or(false),
                speed: p0.map(|p| p.speed_boost_timer).unwrap_or(0.0),
                double_pts: p0.map(|p| p.double_pts_timer).unwrap_or(0.0),
                poison: p0.map(|p| p.poison_timer).unwrap_or(0.0),
                stunned: p0.map(|p| p.stun_timer).unwrap_or(0.0),
            },
            fish_eaten: self.fish_eaten,
            player_is_jd: p0.map(|p| p.is_john_dory).unwrap_or(false),
            coop_count: self.players.len(),
            player_tags,
            particles: self.particles.clone(),
            popups: self.popups.clone(),
            warnings: self.warnings.clone(),
            sounds: self.sounds.clone(),
            music: self.current_music.clone(),
            edible_icons,
        };
        self.hud_json = serde_json::to_string(&payload).unwrap_or_default();
    }
}
