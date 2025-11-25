#![allow(unused)]

use {
    color_eyre::eyre::{Context, ContextCompat, Result},
    rodio::{Decoder, OutputStream, Sink, Source},
    rust_embed::Embed,
    std::{
        io::Cursor,
        sync::{Arc, Mutex},
        thread,
    },
};

#[derive(Embed)]
#[folder = "assets"]
pub struct Asset;

#[derive(Clone)]
pub struct SoundPlayer {
    stream: Arc<OutputStream>,
    music_sink: Arc<Mutex<Option<Sink>>>,
}

impl Default for SoundPlayer {
    fn default() -> Self {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
            .expect("Failed to create audio output stream");

        Self {
            stream: Arc::new(stream_handle),
            music_sink: Arc::new(Mutex::new(None)),
        }
    }
}

impl SoundPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn play_bg_music(&self) -> Result<()> {
        self.stop_music();

        let file_data = Asset::get("bgmusic.mp3")
            .context("Failed to load background music from embedded assets")?;

        let cursor = Cursor::new(file_data.data);
        let source = Decoder::new(cursor)
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

        let file_data = Asset::get("death_screen.mp3")
            .context("Failed to load death screen music from embedded assets")?;

        let cursor = Cursor::new(file_data.data);
        let source = Decoder::new(cursor)
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
        self.play_sound_effect_concurrent("jump.ogg", 0.4)
    }

    pub fn play_death_sound(&self) -> Result<()> {
        self.play_sound_effect_concurrent("die.ogg", 0.5)
    }

    pub fn play_high_score_sound(&self) -> Result<()> {
        self.play_sound_effect_concurrent("new_high_score.ogg", 0.5)
    }

    fn play_sound_effect_concurrent(&self, filename: &str, volume: f32) -> Result<()> {
        let stream = Arc::clone(&self.stream);
        let filename = filename.to_string();

        thread::spawn(move || {
            if let Err(e) = Self::play_sound_in_thread(stream, &filename, volume) {
                eprintln!("Failed to play sound effect {}: {}", filename, e);
            }
        });

        Ok(())
    }

    fn play_sound_in_thread(stream: Arc<OutputStream>, filename: &str, volume: f32) -> Result<()> {
        let file_data = Asset::get(filename).with_context(|| {
            format!(
                "Failed to load sound file from embedded assets: {}",
                filename
            )
        })?;

        let cursor = Cursor::new(file_data.data);
        let source = Decoder::new(cursor)
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
