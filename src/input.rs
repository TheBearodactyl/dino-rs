use {
    crate::{
        config::Cfg,
        sound::SoundPlayer,
        types::{Dino, DinoState},
    },
    crossterm::event::{self, Event, KeyCode},
    std::time::Duration,
};

pub struct InputHandler;

impl InputHandler {
    pub fn handle_input(dino: &mut Dino, player: &SoundPlayer) -> color_eyre::Result<bool> {
        let cfg = Cfg::load()?;
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        return Ok(false);
                    }
                    KeyCode::Char(' ') | KeyCode::Up => {
                        if Self::jump(dino, cfg.clone())
                            && let Err(e) = player.play_jump_sound()
                        {
                            eprintln!("Failed to play jump sound: {}", e);
                        }
                    }
                    KeyCode::Down => {
                        Self::start_crouch(dino);
                    }
                    _ => {}
                }
            }
        }

        if matches!(dino.state, DinoState::Crouching) && !Self::is_down_pressed()? {
            Self::stop_crouch(dino);
        }

        Ok(true)
    }

    pub fn wait_for_key() -> color_eyre::Result<Option<KeyCode>> {
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key_event) = event::read()?
        {
            return Ok(Some(key_event.code));
        }
        Ok(None)
    }

    fn jump(dino: &mut Dino, cfg: Cfg) -> bool {
        if matches!(dino.state, DinoState::Running | DinoState::Crouching) {
            dino.state = DinoState::Jumping;
            dino.velocity_y = cfg.physics.jump_velocity;
            true
        } else {
            false
        }
    }

    fn start_crouch(dino: &mut Dino) {
        dino.state = DinoState::Crouching;
    }

    fn stop_crouch(dino: &mut Dino) {
        if matches!(dino.state, DinoState::Crouching) {
            dino.state = DinoState::Running;
        }
    }

    fn is_down_pressed() -> color_eyre::Result<bool> {
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = event::read()?
                && matches!(key_event.code, KeyCode::Down)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
