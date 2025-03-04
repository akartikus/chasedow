use macroquad::audio::*;
use macroquad::prelude::*;

pub const MUSIC_VOLUME: f32 = 0.5;

pub struct GameAudio {
    pub background_music: Sound,
    pub muted: bool,
    // jump_sound: Sound,
    // game_over_sound: Sound,
}

impl GameAudio {
    pub async fn new() -> Self {
        set_pc_assets_folder("assets");
        Self {
            background_music: load_sound("background.ogg").await.expect("Failed to load background music"),
            muted: false,
            // jump_sound: load_sound("jump.ogg").await.expect("Failed to load jump sound"),
            // game_over_sound: load_sound("game_over.ogg").await.expect("Failed to load game over sound"),
        }
    }

    pub fn play_background(&self) {
        // if !is_sound_playing(&self.background_music) {
            play_sound(&self.background_music, PlaySoundParams {
                looped: true,
                volume: self.music_volume(),
            });
        // }
    }

    // fn play_jump(&self) {
    //     play_audio(&self.jump_sound, PlaySoundParams {
    //         looped: false,
    //         volume: 0.8,
    //     });
    // }
    //
    // fn play_game_over(&self) {
    //     stop_audio(&self.background_music);
    //     play_audio(&self.game_over_sound, PlaySoundParams {
    //         looped: false,
    //         volume: 1.0,
    //     });
    // }

    pub fn music_volume(&self) -> f32 {
        if self.muted { 0.0 } else { MUSIC_VOLUME }
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        set_sound_volume(&self.background_music, self.music_volume());
    }

    pub fn stop_all(&self) {
        stop_sound(&self.background_music);
        // stop_audio(&self.jump_sound);
        // stop_audio(&self.game_over_sound);
    }
}
