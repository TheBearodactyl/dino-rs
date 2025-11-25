use crate::config::Cfg;
use crate::spawner::ObstacleSpawner;
use crate::types::{Cloud, Dino, DinoState, Obstacle, ObstacleType};
use rand::Rng;

pub struct PhysicsEngine {
    pub cfg: Cfg,
    pub dino: Dino,
    pub obstacles: Vec<Obstacle>,
    pub clouds: Vec<Cloud>,
    pub speed: f32,
    pub score_accumulator: f32,
}

impl PhysicsEngine {
    pub fn new(cfg: Cfg, width: usize, height: usize) -> Self {
        let mut clouds = Vec::new();
        for _ in 0..cfg.game.initial_cloud_count {
            clouds.push(Cloud::new(
                rand::rng().random_range(0.0..width as f32),
                rand::rng()
                    .random_range(5..height.saturating_sub(cfg.clone().physics.ground_height + 10)),
            ));
        }

        Self {
            cfg: cfg.clone(),
            dino: Dino::new(),
            obstacles: vec![Obstacle::new(
                width as f32 + 50.0,
                ObstacleType::SmallCactus,
            )],
            clouds,
            speed: cfg.physics.initial_speed,
            score_accumulator: 0.0,
        }
    }

    pub fn reset(&mut self, width: usize, height: usize) {
        self.dino.reset();
        self.obstacles.clear();
        self.obstacles.push(Obstacle::new(
            width as f32 + 50.0,
            ObstacleType::SmallCactus,
        ));

        self.clouds.clear();
        for _ in 0..self.cfg.game.initial_cloud_count {
            self.clouds.push(Cloud::new(
                rand::rng().random_range(0.0..width as f32),
                rand::rng()
                    .random_range(5..height.saturating_sub(self.cfg.physics.ground_height + 10)),
            ));
        }

        self.speed = self.cfg.physics.initial_speed;
        self.score_accumulator = 0.0;
    }

    pub fn update(
        &mut self,
        spawner: &ObstacleSpawner,
        screen_width: usize,
        screen_height: usize,
        score: &mut usize,
    ) -> bool {
        self.update_dino();
        self.update_obstacles(spawner, screen_width);
        self.update_clouds(screen_width, screen_height);
        self.update_score(score);
        self.increase_speed();

        !self.check_collision(screen_height)
    }

    fn update_dino(&mut self) {
        match self.dino.state {
            DinoState::Jumping => {
                self.dino.velocity_y += self.cfg.physics.gravity;
                self.dino.y -= self.dino.velocity_y;

                if self.dino.y <= 0.0 {
                    self.dino.y = 0.0;
                    self.dino.velocity_y = 0.0;
                    self.dino.state = DinoState::Running;
                }
            }
            DinoState::Running | DinoState::Crouching => {
                self.dino.y = 0.0;
                self.dino.velocity_y = 0.0;
            }
        }
    }

    fn update_obstacles(&mut self, spawner: &ObstacleSpawner, screen_width: usize) {
        for obs in &mut self.obstacles {
            obs.x -= self.speed;
        }

        if let Some(last_obs) = self.obstacles.last()
            && spawner.should_spawn(last_obs.x, screen_width as f32)
        {
            self.obstacles.push(spawner.spawn_next(last_obs.x));
        }

        self.obstacles.retain(|o| o.x > -20.0);
    }

    fn update_clouds(&mut self, width: usize, height: usize) {
        for cloud in &mut self.clouds {
            cloud.x -= self.speed / self.cfg.game.cloud_speed_divisor;
            if cloud.x < -10.0 {
                cloud.x = width as f32 + 10.0;
                cloud.y = rand::rng()
                    .random_range(5..height.saturating_sub(self.cfg.physics.ground_height + 10));
            }
        }
    }

    fn update_score(&mut self, score: &mut usize) {
        self.score_accumulator += 1.0;
        if self.score_accumulator >= self.cfg.game.score_update_threshold {
            *score += 1;
            self.score_accumulator = 0.0;
        }
    }

    fn increase_speed(&mut self) {
        self.speed += self.cfg.physics.speed_increment;
    }

    fn check_collision(&self, screen_height: usize) -> bool {
        let dino_x = 10;
        let ground_y = screen_height.saturating_sub(self.cfg.physics.ground_height);
        let dino_ground_y = ground_y.saturating_sub(6);
        let dino_y = dino_ground_y.saturating_sub(self.dino.y as usize);

        let (dino_width, dino_height, hitbox_y_offset) =
            if matches!(self.dino.state, DinoState::Crouching) {
                (6, 4, 2)
            } else {
                (8, 6, 0)
            };

        let dino_hitbox_y = dino_y + hitbox_y_offset;

        for obs in &self.obstacles {
            let obs_x = obs.x as usize;

            let collision = match obs.obstacle_type {
                ObstacleType::SmallCactus => {
                    let obs_y = ground_y.saturating_sub(2);
                    let dino_box = Box::new(dino_x, dino_hitbox_y, dino_width, dino_height);
                    dino_box.check_collision(Box::new(obs_x, obs_y, 3, 2))
                }
                ObstacleType::MediumCactus => {
                    let obs_y = ground_y.saturating_sub(3);
                    let dino_box = Box::new(dino_x, dino_hitbox_y, dino_width, dino_height);
                    let obs_box = Box::new(obs_x, obs_y, 3, 3);

                    dino_box.check_collision(obs_box)
                }
                ObstacleType::TallCactus => {
                    let obs_y = ground_y.saturating_sub(4);
                    let dino_box = Box::new(dino_x, dino_hitbox_y, dino_width, dino_height);
                    let obs_box = Box::new(obs_x, obs_y, 5, 4);

                    dino_box.check_collision(obs_box)
                }
                ObstacleType::WideCactus => {
                    let obs_y = ground_y.saturating_sub(3);
                    let dino_box = Box::new(dino_x, dino_hitbox_y, dino_width, dino_height);
                    let obs_box = Box::new(obs_x, obs_y, 7, 3);

                    dino_box.check_collision(obs_box)
                }
                ObstacleType::PterodactylLow => {
                    let ptero_y = ground_y.saturating_sub(8);
                    let dino_box = Box::new(dino_x, dino_hitbox_y, dino_width, dino_height);
                    let obs_box = Box::new(obs_x, ptero_y, 7, 3);

                    dino_box.check_collision(obs_box)
                }
                ObstacleType::PterodactylMid => {
                    let ptero_y = ground_y.saturating_sub(12);
                    let dino_box = Box::new(dino_x, dino_hitbox_y, dino_width, dino_height);
                    let obs_box = Box::new(obs_x, ptero_y, 7, 3);

                    dino_box.check_collision(obs_box)
                }
                ObstacleType::PterodactylHigh => {
                    let ptero_y = ground_y.saturating_sub(16);
                    let dino_box = Box::new(dino_x, dino_hitbox_y, dino_width, dino_height);
                    let obs_box = Box::new(obs_x, ptero_y, 7, 3);

                    dino_box.check_collision(obs_box)
                }
            };

            if collision {
                return true;
            }
        }

        false
    }
}

struct Box {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

impl Box {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self { x, y, w, h }
    }

    pub fn check_collision(&self, other: Self) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }
}
