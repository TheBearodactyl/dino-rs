use {
    crate::{
        config::Cfg,
        display::DisplaySettings,
        input::InputHandler,
        physics::PhysicsEngine,
        rendering::{DrawArgs, Renderer},
        sound::SoundPlayer,
        spawner::ObstacleSpawner,
    },
    color_eyre::Result,
    crossterm::event::KeyCode,
    std::{
        fs,
        time::{Duration, Instant},
    },
};

pub struct Game {
    cfg: Cfg,
    physics: PhysicsEngine,
    renderer: Renderer,
    spawner: ObstacleSpawner,
    display: DisplaySettings,
    score: usize,
    highscore: usize,
    physics_accumulator: f32,
    player: SoundPlayer,
    last_highscore: usize,
}

impl Game {
    pub fn new() -> Result<Self> {
        let cfg = Cfg::load()?;
        let highscore = load_highscore();
        let display = DisplaySettings::detect()?;
        let physics = PhysicsEngine::new(cfg.clone(), display.width, display.height);
        let renderer = Renderer::new(cfg.clone(), display.width, display.height);
        let spawner = ObstacleSpawner::new(cfg.clone());
        let player = SoundPlayer::new();

        Ok(Self {
            cfg,
            physics,
            renderer,
            spawner,
            display,
            score: 0,
            highscore,
            physics_accumulator: 0.0,
            player,
            last_highscore: highscore,
        })
    }

    pub fn show_countdown(&self) -> Result<()> {
        self.renderer
            .show_countdown(self.display.render_frame_duration)
    }

    pub fn run(&mut self) -> Result<bool> {
        let mut last_frame = Instant::now();

        if let Err(e) = self.player.play_bg_music() {
            eprintln!("Failed to play background music: {}", e);
        }

        let mut played_new_highscore = false;

        loop {
            if !InputHandler::handle_input(&mut self.physics.dino, &self.player)? {
                self.player.stop_music();
                return Ok(false);
            }

            let current_time = Instant::now();
            let frame_time = current_time.duration_since(last_frame).as_secs_f32();
            last_frame = current_time;

            self.physics_accumulator += frame_time;
            self.physics_accumulator = self.physics_accumulator.min(0.25);

            while self.physics_accumulator >= self.cfg.physics.fixed_timestep {
                self.spawner.update_difficulty(self.score);

                if !self.physics.update(
                    &self.spawner,
                    self.display.width,
                    self.display.height,
                    &mut self.score,
                ) {
                    self.handle_game_over()?;
                    return Ok(true);
                }

                if self.score > self.highscore {
                    let was_old_highscore = self.highscore == self.last_highscore;
                    self.highscore = self.score;

                    if was_old_highscore {
                        if !played_new_highscore && let Err(e) = self.player.play_high_score_sound()
                        {
                            eprintln!("Failed to play high score sound: {}", e);
                        }

                        played_new_highscore = true;

                        self.last_highscore = self.highscore;
                    }
                }

                self.physics_accumulator -= self.cfg.physics.fixed_timestep;
            }

            self.renderer.draw(DrawArgs::new(
                self.physics.dino.state,
                self.physics.dino.y,
                &self.physics.obstacles,
                &self.physics.clouds,
                self.score,
                self.highscore,
                self.physics.speed,
            ))?;

            std::thread::sleep(Duration::from_millis(1));
        }
    }

    pub fn wait_for_restart(&mut self) -> Result<bool> {
        loop {
            if let Some(key) = InputHandler::wait_for_key()? {
                match key {
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        self.reset()?;
                        return Ok(true);
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        return Ok(false);
                    }
                    _ => {}
                }
            }
        }
    }

    fn reset(&mut self) -> Result<()> {
        self.display.update_size()?;
        self.physics.reset(self.display.width, self.display.height);
        self.renderer
            .update_dimensions(self.display.width, self.display.height);
        self.score = 0;
        self.physics_accumulator = 0.0;
        self.spawner = ObstacleSpawner::new(self.cfg.clone());
        Ok(())
    }

    fn handle_game_over(&self) -> Result<()> {
        if let Err(e) = self.player.play_death_sound() {
            eprintln!("Failed to play death sound: {}", e);
        }

        self.player.stop_music();
        if let Err(e) = self.player.play_death_screen_music() {
            eprintln!("Failed to play death screen music: {}", e);
        }

        self.renderer.show_game_over(self.score, self.highscore)?;
        save_highscore(self.highscore);
        Ok(())
    }
}

fn load_highscore() -> usize {
    let cfg = Cfg::load().expect("Failed to load config");
    fs::read_to_string(cfg.highscore_file)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn save_highscore(highscore: usize) {
    let cfg = Cfg::load().expect("Failed to load config");
    fs::write(cfg.highscore_file, highscore.to_string()).ok();
}
