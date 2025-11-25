use {
    color_eyre::eyre::{Context, Result},
    rodio::{Decoder, OutputStream, Sink, Source},
    std::{
        fs::File,
        sync::{Arc, Mutex},
        thread,
    },
};

#[derive(Clone)]
pub struct SoundPlayer {
    stream: Arc<OutputStream>,
    music_sink: Arc<Mutex<Option<Sink>>>,
    death_sound: Arc<String>,
    jump_sound: Arc<String>,
    high_score_sound: Arc<String>,
    bg_music: Arc<String>,
    death_screen_music: Arc<String>,
}

impl Default for SoundPlayer {
    fn default() -> Self {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
            .expect("Failed to create audio output stream");

        Self {
            stream: Arc::new(stream_handle),
            music_sink: Arc::new(Mutex::new(None)),
            bg_music: Arc::new("assets/bgmusic.mp3".to_string()),
            death_sound: Arc::new("assets/die.ogg".to_string()),
            death_screen_music: Arc::new("assets/death_screen.mp3".to_string()),
            jump_sound: Arc::new("assets/jump.ogg".to_string()),
            high_score_sound: Arc::new("assets/new_high_score.ogg".to_string()),
        }
    }
}

impl SoundPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn play_bg_music(&self) -> Result<()> {
        self.stop_music();

        let file = File::open(self.bg_music.as_str())
            .with_context(|| format!("Failed to open background music: {}", self.bg_music))?;

        let source = Decoder::try_from(file)
            .context("Failed to decode background music")?
            .repeat_infinite()
            .amplify(0.3);

        let sink = rodio::Sink::connect_new(self.stream.mixer());
        sink.append(source);
        sink.play();

        if let Ok(mut music_sink) = self.music_sink.lock() {
            *music_sink = Some(sink);
        }

        Ok(())
    }

    pub fn play_death_screen_music(&self) -> Result<()> {
        self.stop_music();

        let file = File::open(self.death_screen_music.as_str()).with_context(|| {
            format!(
                "Failed to open death screen music: {}",
                self.death_screen_music
            )
        })?;

        let source = Decoder::try_from(file)
            .context("Failed to decode death screen music")?
            .repeat_infinite()
            .amplify(0.3);

        let sink = rodio::Sink::connect_new(self.stream.mixer());
        sink.append(source);
        sink.play();

        if let Ok(mut music_sink) = self.music_sink.lock() {
            *music_sink = Some(sink);
        }

        Ok(())
    }

    pub fn stop_music(&self) {
        if let Ok(mut music_sink) = self.music_sink.lock()
            && let Some(sink) = music_sink.take()
        {
            sink.stop();
        }
    }

    pub fn pause_music(&self) {
        if let Ok(music_sink) = self.music_sink.lock()
            && let Some(sink) = music_sink.as_ref()
        {
            sink.pause();
        }
    }

    pub fn resume_music(&self) {
        if let Ok(music_sink) = self.music_sink.lock()
            && let Some(sink) = music_sink.as_ref()
        {
            sink.play();
        }
    }

    pub fn play_jump_sound(&self) -> Result<()> {
        self.play_sound_effect_concurrent(self.jump_sound.clone(), 0.4)
    }

    pub fn play_death_sound(&self) -> Result<()> {
        self.play_sound_effect_concurrent(self.death_sound.clone(), 0.5)
    }

    pub fn play_high_score_sound(&self) -> Result<()> {
        self.play_sound_effect_concurrent(self.high_score_sound.clone(), 0.5)
    }

    fn play_sound_effect_concurrent(&self, path: Arc<String>, volume: f32) -> Result<()> {
        let stream = Arc::clone(&self.stream);

        thread::spawn(move || {
            if let Err(e) = Self::play_sound_in_thread(stream, &path, volume) {
                eprintln!("Failed to play sound effect {}: {}", path, e);
            }
        });

        Ok(())
    }

    fn play_sound_in_thread(stream: Arc<OutputStream>, path: &str, volume: f32) -> Result<()> {
        let file =
            File::open(path).with_context(|| format!("Failed to open sound file: {}", path))?;

        let source = Decoder::try_from(file)
            .context("Failed to decode sound")?
            .amplify(volume);

        let sink = rodio::Sink::connect_new(stream.mixer());
        sink.append(source);

        sink.sleep_until_end();

        Ok(())
    }

    pub fn set_music_volume(&self, volume: f32) {
        if let Ok(music_sink) = self.music_sink.lock()
            && let Some(sink) = music_sink.as_ref()
        {
            sink.set_volume(volume.clamp(0.0, 1.0));
        }
    }

    pub fn is_music_playing(&self) -> bool {
        if let Ok(music_sink) = self.music_sink.lock() {
            music_sink.as_ref().map(|s| !s.is_paused()).unwrap_or(false)
        } else {
            false
        }
    }
}
