use crate::config::Cfg;
use display_info::DisplayInfo;
use std::time::Duration;

pub struct DisplaySettings {
    pub width: usize,
    pub height: usize,
    pub render_frame_duration: Duration,
}

impl DisplaySettings {
    pub fn detect() -> color_eyre::Result<Self> {
        let (cols, rows) = crossterm::terminal::size()?;
        let refresh_rate = Self::detect_refresh_rate();

        Ok(Self {
            width: cols as usize,
            height: rows as usize,
            render_frame_duration: Duration::from_secs_f32(1.0 / refresh_rate),
        })
    }

    fn detect_refresh_rate() -> f32 {
        let cfg = Cfg::load().expect("Failed to load config");
        DisplayInfo::all()
            .ok()
            .map(|displays| displays[0].clone())
            .filter(|display| display.frequency > 0.0)
            .map(|display| display.frequency)
            .unwrap_or(cfg.physics.default_fps)
    }

    pub fn update_size(&mut self) -> color_eyre::Result<()> {
        let (cols, rows) = crossterm::terminal::size()?;
        self.width = cols as usize;
        self.height = rows as usize;
        Ok(())
    }
}
