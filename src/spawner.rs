use crate::config::Cfg;
use crate::types::{Obstacle, ObstacleType};
use rand::Rng;

pub struct ObstacleSpawner {
    cfg: Cfg,
    min_gap: f32,
    max_gap: f32,
    difficulty_score: usize,
}

impl ObstacleSpawner {
    pub fn new(cfg: Cfg) -> Self {
        Self {
            cfg,
            min_gap: 50.0,
            max_gap: 120.0,
            difficulty_score: 0,
        }
    }

    pub fn update_difficulty(&mut self, score: usize) {
        self.difficulty_score = score;
        let difficulty_factor = (score as f32 / 1000.0).min(1.0);
        self.min_gap = 50.0 - (difficulty_factor * 15.0);
        self.max_gap = 120.0 - (difficulty_factor * 30.0);
    }

    pub fn should_spawn(&self, rightmost_obstacle_x: f32, screen_width: f32) -> bool {
        rightmost_obstacle_x < screen_width - self.min_gap
    }

    pub fn spawn_next(&self, last_x: f32) -> Obstacle {
        let gap = self.get_spawn_distance();
        let new_x = last_x + gap;
        let obstacle_type = self.choose_obstacle_type();
        Obstacle::new(new_x, obstacle_type)
    }

    fn get_spawn_distance(&self) -> f32 {
        let mut rng = rand::rng();
        rng.random_range(self.min_gap..=self.max_gap)
    }

    fn choose_obstacle_type(&self) -> ObstacleType {
        let mut rng = rand::rng();
        let can_spawn_ptero = self.difficulty_score > self.cfg.game.ptero_spawn_score;
        let use_hard_obstacles = self.difficulty_score > self.cfg.game.hard_obstacles_score
            && rng.random_range(0..100) < 30;

        if can_spawn_ptero && rng.random_range(0..100) < 25 {
            match rng.random_range(0..3) {
                0 => ObstacleType::PterodactylLow,
                1 => ObstacleType::PterodactylMid,
                _ => ObstacleType::PterodactylHigh,
            }
        } else if use_hard_obstacles {
            match rng.random_range(0..2) {
                0 => ObstacleType::TallCactus,
                _ => ObstacleType::WideCactus,
            }
        } else {
            match rng.random_range(0..3) {
                0 => ObstacleType::SmallCactus,
                1 => ObstacleType::MediumCactus,
                _ => ObstacleType::TallCactus,
            }
        }
    }
}

impl Default for ObstacleSpawner {
    fn default() -> Self {
        let cfg = Cfg::load().expect("Failed");
        Self::new(cfg)
    }
}
