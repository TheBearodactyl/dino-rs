use {
    color_eyre::eyre::{Context, Result},
    config::Config,
    serde::{Deserialize, Serialize},
    std::{fs, path::Path},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct Cfg {
    #[serde(default)]
    pub highscore_file: String,

    #[serde(default)]
    pub physics: PhysicsConfig,

    #[serde(default)]
    pub game: GameConfig,
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            highscore_file: "highscore.txt".to_string(),
            physics: PhysicsConfig::default(),
            game: GameConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PhysicsConfig {
    #[serde(default)]
    pub ground_height: usize,
    #[serde(default)]
    pub default_fps: f32,
    #[serde(default)]
    pub physics_fps: f32,
    #[serde(default)]
    pub fixed_timestep: f32,
    #[serde(default)]
    pub gravity: f32,
    #[serde(default)]
    pub jump_velocity: f32,
    #[serde(default)]
    pub initial_speed: f32,
    #[serde(default)]
    pub speed_increment: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameConfig {
    #[serde(default)]
    pub score_update_threshold: f32,
    #[serde(default)]
    pub ptero_spawn_score: usize,
    #[serde(default)]
    pub hard_obstacles_score: usize,
    #[serde(default)]
    pub initial_cloud_count: usize,
    #[serde(default)]
    pub cloud_speed_divisor: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        let physics_fps = 60.0;
        Self {
            ground_height: 3,
            default_fps: 60.0,
            physics_fps,
            fixed_timestep: 0.03,
            gravity: 1.5,
            jump_velocity: -7.0,
            initial_speed: 4.0,
            speed_increment: 0.002,
        }
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            score_update_threshold: 6.0,
            ptero_spawn_score: 300,
            hard_obstacles_score: 500,
            initial_cloud_count: 8,
            cloud_speed_divisor: 4.0,
        }
    }
}

fn find_local_config_file() -> Option<String> {
    let extensions = ["toml", "yaml", "yml", "json"];

    for ext in &extensions {
        let filename = format!("dino.{}", ext);
        if Path::new(&filename).exists() {
            return Some(filename);
        }
    }

    None
}

impl Cfg {
    pub fn load() -> Result<Self> {
        let defaults = Self::default();
        let mut builder = Config::builder();

        builder = builder.add_source(
            config::Config::try_from(&defaults).context("Failed to build default config source")?,
        );

        builder = builder.add_source(config::Environment::with_prefix("DINO"));

        if let Some(local_config) = find_local_config_file() {
            let local_config_name = local_config
                .strip_suffix(&format!(
                    ".{}",
                    local_config.split('.').next_back().unwrap_or("toml")
                ))
                .unwrap_or(&local_config);

            builder =
                builder.add_source(config::File::with_name(local_config_name).required(false));
        }

        let settings = builder.build()?;

        let cfg = settings
            .try_deserialize::<Self>()
            .unwrap_or_else(|_| defaults.clone());

        if !Path::new(&cfg.highscore_file).exists()
            && let Err(e) = defaults.init(&cfg.highscore_file)
        {
            eprintln!("Couldn't save default config file: {}", e);
        }

        Ok(cfg)
    }

    pub fn init(&self, path: &str) -> Result<()> {
        let toml_str =
            toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;

        fs::write(path, toml_str)
            .with_context(|| format!("Failed to write configuration to file: {}", path))?;

        Ok(())
    }
}
