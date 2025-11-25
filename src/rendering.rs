use {
    crate::{
        config::Cfg,
        types::{Cloud, DinoState, Obstacle, ObstacleType},
    },
    crossterm::{cursor, execute},
    std::io::{Write, stdout},
};

pub struct Renderer {
    cfg: Cfg,
    width: usize,
    height: usize,
}

pub struct DrawArgs<'a> {
    dino_state: DinoState,
    dino_y: f32,
    obstacles: &'a [Obstacle],
    clouds: &'a [Cloud],
    score: usize,
    highscore: usize,
    speed: f32,
}

impl<'a> DrawArgs<'a> {
    pub fn new(
        dino_state: DinoState,
        dino_y: f32,
        obstacles: &'a [Obstacle],
        clouds: &'a [Cloud],
        score: usize,
        highscore: usize,
        speed: f32,
    ) -> Self {
        Self {
            dino_state,
            dino_y,
            obstacles,
            clouds,
            score,
            highscore,
            speed,
        }
    }
}

impl Renderer {
    pub fn new(cfg: Cfg, width: usize, height: usize) -> Self {
        Self { cfg, width, height }
    }

    pub fn update_dimensions(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    pub fn draw(&self, draw_args: DrawArgs) -> color_eyre::Result<()> {
        execute!(stdout(), cursor::MoveTo(0, 0))?;

        let mut screen = vec![vec![' '; self.width]; self.height];

        self.draw_clouds(&mut screen, draw_args.clouds);
        self.draw_ground(&mut screen);
        self.draw_dino(&mut screen, draw_args.dino_state, draw_args.dino_y);
        self.draw_obstacles(&mut screen, draw_args.obstacles);

        self.render_screen(&screen)?;
        self.render_status(draw_args.score, draw_args.highscore, draw_args.speed)?;

        Ok(())
    }

    fn draw_clouds(&self, screen: &mut [Vec<char>], clouds: &[Cloud]) {
        let cloud_art = ["  .--.  ", " (    ) ", "(_.____)"];

        for cloud in clouds {
            let x = cloud.x as usize;
            if x < self.width.saturating_sub(6) && cloud.y < self.height.saturating_sub(2) {
                for (dy, line) in cloud_art.iter().enumerate() {
                    if cloud.y + dy < self.height {
                        for (dx, ch) in line.chars().enumerate() {
                            if x + dx < self.width {
                                screen[cloud.y + dy][x + dx] = ch;
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw_ground(&self, screen: &mut [Vec<char>]) {
        let ground_y = self.height.saturating_sub(self.cfg.physics.ground_height);
        (ground_y..self.height).for_each(|y| {
            (0..self.width).for_each(|x| {
                if y == ground_y {
                    screen[y][x] = '─';
                } else if y == ground_y + 1 {
                    screen[y][x] = if x % 3 == 0 { '.' } else { ' ' };
                }
            });
        });
    }

    fn draw_dino(&self, screen: &mut [Vec<char>], state: DinoState, dino_y: f32) {
        let dino_x = 10;
        let ground_y = self.height.saturating_sub(self.cfg.physics.ground_height);
        let dino_ground_y = ground_y.saturating_sub(6);
        let dino_screen_y = dino_ground_y.saturating_sub(dino_y as usize);

        let dino_art = if matches!(state, DinoState::Crouching) {
            vec![
                "               ",
                "               ",
                "⢠⣀⢀⠀⣀⣀⣀⣀⡀⣀⡠⣄⣤⣠⡀",
                "⠀⠹⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⡿⡇",
                "⠀⠀⠈⢹⠻⣿⠟⠛⡏⠉⠙⠋⠒⠂⠀",
                "⠀⠀⠀⠈⠐⠇⠀⠀⠁⠁⠀⠀⠀⠀ ",
            ]
        } else {
            vec![
                "          ",
                "     ⣾⣽⣿⣿⡇",
                "⢀   ⢀⣿⣿⠯  ",
                "⢸⣦⣤⣾⣿⣿⣿    ",
                " ⠙⢿⣿⣿⡿⠃   ",
                "  ⠸ ⠈⠇    ",
            ]
        };

        for (dy, line) in dino_art.iter().enumerate() {
            let y = dino_screen_y + dy;
            if y < ground_y && y < self.height {
                for (dx, ch) in line.chars().enumerate() {
                    let x = dino_x + dx;
                    if x < self.width && ch != ' ' {
                        screen[y][x] = ch;
                    }
                }
            }
        }
    }

    fn draw_obstacles(&self, screen: &mut [Vec<char>], obstacles: &[Obstacle]) {
        let ground_y = self.height.saturating_sub(self.cfg.physics.ground_height);

        for obs in obstacles {
            let obs_x = obs.x as usize;
            if obs_x >= self.width {
                continue;
            }

            match obs.obstacle_type {
                ObstacleType::SmallCactus => {
                    let art = vec!["|||", "|||"];
                    self.draw_obstacle_art(screen, &art, obs_x, ground_y);
                }
                ObstacleType::MediumCactus => {
                    let art = vec![" | ", "/|\\", " | "];
                    self.draw_obstacle_art(screen, &art, obs_x, ground_y);
                }
                ObstacleType::TallCactus => {
                    let art = vec!["  |  ", " \\|/ ", "  |  ", " /|\\ "];
                    self.draw_obstacle_art(screen, &art, obs_x, ground_y);
                }
                ObstacleType::WideCactus => {
                    let art = vec![" | | ", "/|\\|/\\", " | | "];
                    self.draw_obstacle_art(screen, &art, obs_x, ground_y);
                }
                ObstacleType::PterodactylLow => {
                    let art = vec![" ^   ^ ", "<(o.o)>", "  \\_/  "];
                    let fly_y = ground_y.saturating_sub(8);
                    self.draw_ptero_art(screen, &art, obs_x, fly_y);
                }
                ObstacleType::PterodactylMid => {
                    let art = vec![" ^   ^ ", "<(o.o)>", "  \\_/  "];
                    let fly_y = ground_y.saturating_sub(12);
                    self.draw_ptero_art(screen, &art, obs_x, fly_y);
                }
                ObstacleType::PterodactylHigh => {
                    let art = vec![" ^   ^ ", "<(o.o)>", "  \\_/  "];
                    let fly_y = ground_y.saturating_sub(16);
                    self.draw_ptero_art(screen, &art, obs_x, fly_y);
                }
            }
        }
    }

    fn draw_obstacle_art(&self, screen: &mut [Vec<char>], art: &[&str], x: usize, ground_y: usize) {
        let start_y = ground_y.saturating_sub(art.len());
        for (dy, line) in art.iter().enumerate() {
            let y = start_y + dy;
            if y < ground_y && y < screen.len() {
                for (dx, ch) in line.chars().enumerate() {
                    let px = x + dx;
                    if px < screen[0].len() && ch != ' ' {
                        screen[y][px] = ch;
                    }
                }
            }
        }
    }

    fn draw_ptero_art(&self, screen: &mut [Vec<char>], art: &[&str], x: usize, y: usize) {
        for (dy, line) in art.iter().enumerate() {
            let py = y + dy;
            if py < screen.len() {
                for (dx, ch) in line.chars().enumerate() {
                    let px = x + dx;
                    if px < screen[0].len() && ch != ' ' {
                        screen[py][px] = ch;
                    }
                }
            }
        }
    }

    fn render_screen(&self, screen: &[Vec<char>]) -> color_eyre::Result<()> {
        for row in screen {
            for &ch in row {
                print!("{}", ch);
            }
        }
        Ok(())
    }

    fn render_status(&self, score: usize, highscore: usize, speed: f32) -> color_eyre::Result<()> {
        print!(
            "\rScore: {} | High: {} | Speed: {:.1}x",
            score,
            highscore,
            speed / self.cfg.physics.initial_speed
        );
        stdout().flush()?;
        Ok(())
    }

    pub fn show_game_over(&self, score: usize, highscore: usize) -> color_eyre::Result<()> {
        use crossterm::terminal::ClearType;
        execute!(
            stdout(),
            crossterm::terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        println!("\n  ╔═══════════════════════════════╗");
        println!("  ║       GAME OVER!              ║");
        println!("  ╠═══════════════════════════════╣");
        println!("  ║  Final Score: {:<14}  ║", score);
        println!("  ║  High Score:  {:<14}  ║", highscore);
        println!("  ╚═══════════════════════════════╝");
        println!("\n  Press 'R' to restart or 'Q' to quit");
        stdout().flush()?;
        Ok(())
    }

    pub fn show_countdown(&self, frame_duration: std::time::Duration) -> color_eyre::Result<()> {
        use crossterm::terminal::ClearType;

        for i in (1..=3).rev() {
            execute!(
                stdout(),
                crossterm::terminal::Clear(ClearType::All),
                cursor::MoveTo(0, 0)
            )?;
            println!("Starting in {}...", i);
            println!("\nControls:");
            println!("  SPACE / UP - Jump");
            println!("  DOWN - Crouch (hold)");
            println!("  Q - Quit");
            println!(
                "\nPhysics: {} FPS | Rendering: {} FPS",
                self.cfg.physics.physics_fps,
                (1.0 / frame_duration.as_secs_f32()).round()
            );
            stdout().flush()?;
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        execute!(
            stdout(),
            crossterm::terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        Ok(())
    }
}
