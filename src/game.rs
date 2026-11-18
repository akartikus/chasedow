use macroquad::prelude::*;
use macroquad::rand::*;
use macroquad_platformer::*;

use crate::audio::*;
use crate::coin::*;
use crate::constants::*;
use crate::platform::*;
use crate::player::*;
use crate::shadow::*;

#[cfg(not(target_arch = "wasm32"))]
pub const HIGH_SCORE_FILE: &str = "highscore.txt";

#[cfg(not(target_arch = "wasm32"))]
pub fn load_high_score() -> f32 {
    std::fs::read_to_string(HIGH_SCORE_FILE)
        .ok()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .unwrap_or(0.0)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_high_score(score: f32) {
    // Best effort: a failed save must never crash the game
    let _ = std::fs::write(HIGH_SCORE_FILE, score.to_string());
}

// Persistence is not available on WASM yet (see TASKS.md, task 18)
#[cfg(target_arch = "wasm32")]
pub fn load_high_score() -> f32 {
    0.0
}

#[cfg(target_arch = "wasm32")]
pub fn save_high_score(_score: f32) {}

#[derive(PartialEq)]
pub enum GameScreen {
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

/// Combo count after grabbing a coin: it grows if the previous coin was grabbed
/// less than COMBO_WINDOW ago (`time_left` > 0), otherwise it restarts at 1.
pub fn next_combo(combo: i32, time_left: f32) -> i32 {
    if time_left > 0.0 { combo + 1 } else { 1 }
}

/// Points multiplier for a combo count, between 1 and COMBO_MAX.
pub fn combo_multiplier(combo: i32) -> i32 {
    combo.clamp(1, COMBO_MAX)
}

// Short-lived text that floats up and fades out (e.g. "+10")
pub struct FloatingText {
    pub pos: Vec2,
    pub life: f32,
    pub text: String,
}

pub struct GameState {
    pub world: World,
    pub player: Player,
    pub shadow: Shadow,
    pub platforms: Vec<Platform>,
    pub score: f32,
    pub screen: GameScreen,
    pub high_score: f32,
    pub new_record: bool,
    pub difficulty_timer: f32,
    pub lives: i32,
    pub invulnerable_timer: f32,
    pub is_invulnerable: bool,
    pub audio: GameAudio,
    pub coins: Vec<Coin>,
    pub coin_texture: Texture2D,
    pub coin_spawn_timer: f32,
    pub coin_points: i32,
    pub floating_texts: Vec<FloatingText>,
    pub shadow_freeze: f32,
    pub hit_flash: f32,
    pub combo: i32,
    pub combo_timer: f32,
}

impl GameState {
    pub async fn new() -> Self {
        let mut world = World::new();
        let player = Player::new(&mut world).await;
        let shadow = Shadow::new(SHADOW_FRAMES_DELAY).await;
        let platforms = create_platforms(&mut world).await;
        let audio = GameAudio::new().await;
        let coin_texture = load_texture("player.png").await.expect("Couldn't load coin texture");
        coin_texture.set_filter(FilterMode::Nearest);

        Self {
            world,
            player,
            shadow,
            platforms,
            score: 0.0,
            screen: GameScreen::MainMenu,
            high_score: load_high_score(),
            new_record: false,
            difficulty_timer: SHADOW_DELAY_INTERVAL,
            lives: INITIAL_LIVES,
            invulnerable_timer: 0.0,
            is_invulnerable: false,
            audio,
            coins: Vec::new(),
            coin_texture,
            coin_spawn_timer: 0.0,
            coin_points: 0,
            floating_texts: Vec::new(),
            shadow_freeze: 0.0,
            hit_flash: 0.0,
            combo: 0,
            combo_timer: 0.0,
        }
    }

    pub async fn reset_game(&mut self) {
        // Reset world and game elements
        self.world = World::new();
        self.player = Player::new(&mut self.world).await;
        self.shadow = Shadow::new(SHADOW_FRAMES_DELAY).await;
        self.platforms = create_platforms(&mut self.world).await;
        self.score = 0.0;
        self.new_record = false;
        self.difficulty_timer = SHADOW_DELAY_INTERVAL;
        self.lives = INITIAL_LIVES;
        self.invulnerable_timer = 0.0;
        self.is_invulnerable = false;

        self.coins.clear();
        self.coin_spawn_timer = 0.0;
        self.coin_points = 0;
        self.floating_texts.clear();
        self.shadow_freeze = 0.0;
        self.hit_flash = 0.0;
        self.combo = 0;
        self.combo_timer = 0.0;
    }

    pub fn handle_shadow_collision(&mut self) {
        if self.invulnerable_timer <= 0.0 {
            self.lives -= 1;
            if self.lives <= 0 {
                if self.score > self.high_score {
                    self.high_score = self.score;
                    self.new_record = true;
                    save_high_score(self.high_score);
                }
                self.audio.stop_all();
                self.screen = GameScreen::GameOver;
            } else {
                // Start invulnerability period
                self.invulnerable_timer = INVULNERABILITY_DURATION;
                self.hit_flash = HIT_FLASH_DURATION;
                // Optional: Reset player position after hit
                let new_pos = self.world.actor_pos(self.player.collider);
                // new_pos.y -= 50.0; // Move player up a bit to avoid immediate re-collision
                self.world.set_actor_position(self.player.collider, new_pos);
            }
        }
    }

    pub async fn update(&mut self) {
        // Mute toggle works on every screen
        if is_key_pressed(KeyCode::M) {
            self.audio.toggle_mute();
        }

        match self.screen {
            GameScreen::Playing => self.update_playing(),
            GameScreen::Paused => self.update_paused(),
            GameScreen::MainMenu => self.update_main_menu().await,
            GameScreen::GameOver => self.update_game_over().await,
        }
    }

    /// Adds coin points with the combo multiplier applied and returns the points gained.
    fn add_coin_points(&mut self, base: i32) -> i32 {
        self.combo = next_combo(self.combo, self.combo_timer);
        self.combo_timer = COMBO_WINDOW;
        let gained = base * combo_multiplier(self.combo);
        self.coin_points += gained;
        gained
    }

    pub fn spawn_coin(&mut self) {
        // Random position within window bounds
        let x = gen_range(0.0, WINDOW_WIDTH - COIN_SIZE.x);
        let y = gen_range(100.0, WINDOW_HEIGHT - COIN_SIZE.y - 50.0);  // Keep above ground level

        // One coin out of FREEZE_SPAWN_CHANCE freezes the shadow
        let kind = if gen_range(0, FREEZE_SPAWN_CHANCE) == 0 {
            CoinKind::FreezeShadow
        } else if self.lives < INITIAL_LIVES && gen_range(0, EXTRA_LIFE_SPAWN_CHANCE) == 0 {
            CoinKind::ExtraLife
        } else if gen_range(0, SPECIAL_SPAWN_CHANCE) == 0 {
            CoinKind::Special
        } else {
            CoinKind::Normal
        };

        self.coins.push(Coin::new(vec2(x, y), kind, self.coin_texture.clone()));
    }

    pub fn update_playing(&mut self) {
        // Music is started once when a game begins (see update_main_menu / update_game_over):
        // calling play_background() every frame would restart it constantly
        // (https://github.com/not-fl3/macroquad/issues/440)

        // Update coin spawn timer
        self.coin_spawn_timer -= get_frame_time();
        if self.coin_spawn_timer <= 0.0 {
            self.spawn_coin();
            self.coin_spawn_timer = COIN_SPAWN_INTERVAL;
        }

        // The combo ends when the window runs out
        if self.combo_timer > 0.0 {
            self.combo_timer -= get_frame_time();
            if self.combo_timer <= 0.0 {
                self.combo = 0;
            }
        }

        // Update existing coins
        let player_pos = self.world.actor_pos(self.player.collider);
        let mut i = 0;
        while i < self.coins.len() {
            if !self.coins[i].update() {
                self.coins.remove(i);
            } else if self.coins[i].collides_with_player(player_pos, PLAYER_SIZE) {
                let text = match self.coins[i].kind {
                    CoinKind::Normal => format!("+{}", self.add_coin_points(COIN_POINTS)),
                    CoinKind::Special => format!("+{}", self.add_coin_points(COIN_SPECIAL_POINTS)),
                    CoinKind::ExtraLife => {
                        // Never go above the number of hearts that can be displayed
                        self.lives = (self.lives + 1).min(INITIAL_LIVES);
                        "+1 LIFE".to_string()
                    }
                    CoinKind::FreezeShadow => {
                        self.shadow_freeze = FREEZE_DURATION;
                        "FREEZE!".to_string()
                    }
                };
                self.floating_texts.push(FloatingText {
                    pos: self.coins[i].position,
                    life: FLOATING_TEXT_LIFETIME,
                    text,
                });
                self.coins.remove(i);
            } else {
                i += 1;
            }
        }

        // Update floating texts
        for t in self.floating_texts.iter_mut() {
            t.pos.y -= FLOATING_TEXT_SPEED * get_frame_time();
            t.life -= get_frame_time();
        }
        self.floating_texts.retain(|t| t.life > 0.0);

        self.hit_flash = (self.hit_flash - get_frame_time()).max(0.0);

        // Update invulnerability
        if self.is_invulnerable {
            self.invulnerable_timer -= get_frame_time();
            if self.invulnerable_timer <= 0.0 {
                self.is_invulnerable = false;
                self.invulnerable_timer = 0.0;
            }
        }

        // Check for pause
        if is_key_pressed(KeyCode::Escape) {
            self.screen = GameScreen::Paused;
            return;
        }

        // Update game elements
        let player_pos = self.world.actor_pos(self.player.collider);
        for platform in self.platforms.iter_mut() {
            platform.update(&mut self.world, player_pos);
        }

        self.player.update(&mut self.world);

        // Enforce window boundaries
        let mut player_pos = self.world.actor_pos(self.player.collider);
        if player_pos.x < 0.0 {
            player_pos.x = 0.0;
            self.world.set_actor_position(self.player.collider, player_pos);
            self.player.speed.x = 0.0;
        } else if player_pos.x > WINDOW_WIDTH - PLAYER_SIZE.x {
            player_pos.x = WINDOW_WIDTH - PLAYER_SIZE.x;
            self.world.set_actor_position(self.player.collider, player_pos);
            self.player.speed.x = 0.0;
        }

        // The shadow gets closer as time goes by
        self.difficulty_timer -= get_frame_time();
        if self.difficulty_timer <= 0.0 {
            self.shadow.reduce_delay(SHADOW_DELAY_STEP, SHADOW_DELAY_MIN);
            self.difficulty_timer = SHADOW_DELAY_INTERVAL;
        }

        // A frozen shadow neither moves nor hurts
        let shadow_frozen = self.shadow_freeze > 0.0;
        if shadow_frozen {
            self.shadow_freeze -= get_frame_time();
        } else {
            self.shadow.update(player_pos);
        }

        // Check for collision with shadow
        if !shadow_frozen && self.shadow.collides_with_player(player_pos) {
            self.is_invulnerable = true;
            self.handle_shadow_collision();
        }

        self.score += get_frame_time();
    }

    pub fn update_paused(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = GameScreen::Playing;
        }
    }

    pub async fn update_main_menu(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            self.reset_game().await;
            self.audio.play_background();
            self.screen = GameScreen::Playing;
        }
    }

    pub async fn update_game_over(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            self.reset_game().await;
            self.audio.play_background();
            self.screen = GameScreen::Playing;
        } else if is_key_pressed(KeyCode::Escape) {
            self.screen = GameScreen::MainMenu;
        }
    }

    pub fn draw(&mut self) {
        clear_background(BACKGROUND_COLOR);

        match self.screen {
            GameScreen::Playing => self.draw_playing(),
            GameScreen::Paused => self.draw_paused(),
            GameScreen::MainMenu => self.draw_main_menu(),
            GameScreen::GameOver => self.draw_game_over(),
        }
    }


    pub fn should_draw_player(&self) -> bool {
        if !self.is_invulnerable {
            return true;
        }
        // Create a flashing effect based on time
        (self.invulnerable_timer * FLASH_FREQUENCY).sin() > 0.0
    }

    pub fn draw_playing(&mut self) {
        // Draw coins
        for coin in &self.coins {
            coin.draw();
        }

        // Draw game elements
        for platform in &self.platforms {
            platform.draw(&self.world);
        }

        // Warning ring around the shadow when it gets close (not while it is frozen)
        let shadow_pos = self.shadow.history.positions.first().copied();
        if let (Some(shadow_pos), true) = (shadow_pos, self.shadow_freeze <= 0.0) {
            let player_pos = self.world.actor_pos(self.player.collider);
            let dist = player_pos.distance(shadow_pos);
            if dist < SHADOW_ALERT_DISTANCE {
                let closeness = 1.0 - dist / SHADOW_ALERT_DISTANCE;
                let pulse = 1.0 + 0.15 * (get_time() as f32 * 12.0).sin();
                let center = shadow_pos + PLAYER_SIZE * 0.5;
                let mut color = TEXT_WARNING;
                color.a = 0.2 + 0.6 * closeness;
                draw_circle_lines(center.x, center.y, PLAYER_SIZE.x * 0.8 * pulse, 3.0, color);
            }
        }
        self.shadow.draw();

        // Draw player with flashing effect when invulnerable
        if self.should_draw_player() {
            self.player.draw(&self.world);
        }

        // Draw floating texts (fading out)
        for t in &self.floating_texts {
            let mut color = TEXT_GOLD;
            color.a = (t.life / FLOATING_TEXT_LIFETIME).clamp(0.0, 1.0);
            draw_text(&t.text, t.pos.x, t.pos.y, 24.0, color);
        }

        // Red flash when a life was just lost
        if self.hit_flash > 0.0 {
            let alpha = 0.4 * self.hit_flash / HIT_FLASH_DURATION;
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.8, 0.1, 0.05, alpha));
        }

        self.draw_ui();
    }

    pub fn draw_paused(&mut self) {
        // Draw game elements in background
        self.draw_playing();

        // Draw pause overlay
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Semi-transparent overlay
        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.0, 0.0, 0.0, 0.9));

        // Pause menu text
        let pause_text = "PAUSED";
        let text_dims = measure_text(pause_text, None, 50, 1.0);
        draw_text(
            pause_text,
            screen_w * 0.5 - text_dims.width * 0.5,
            screen_h * 0.5,
            40.0,
            WHITE,
        );

        let instruction_text = "Press ESC to resume";
        let instruction_dims = measure_text(instruction_text, None, 20, 1.0);
        draw_text(
            instruction_text,
            screen_w * 0.5 - instruction_dims.width * 0.5,
            screen_h * 0.5 + 40.0,
            20.0,
            WHITE,
        );
    }

    pub fn draw_main_menu(&self) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Title
        let title_text = "CHA(SE)DOW";
        let title_dims = measure_text(title_text, None, 50, 1.0);
        draw_text(
            title_text,
            screen_w * 0.5 - title_dims.width * 0.5,
            screen_h * 0.4,
            50.0,
            TEXT_ACCENT,
        );

        // High score
        if self.high_score > 0.0 {
            let high_score_text = format!("High Score: {:.0}", self.high_score);
            let score_dims = measure_text(&high_score_text, None, 25, 1.0);
            draw_text(
                &high_score_text,
                screen_w * 0.5 - score_dims.width * 0.5,
                screen_h * 0.5,
                25.0,
                TEXT_PRIMARY,
            );
        }

        // Start instruction
        let start_text = "Press SPACE to start";
        let start_dims = measure_text(start_text, None, 25, 1.0);
        draw_text(
            start_text,
            screen_w * 0.5 - start_dims.width * 0.5,
            screen_h * 0.6,
            25.0,
            TEXT_PRIMARY,
        );

        // Controls
        let controls_text = vec![
            "Controls:",
            "LEFT/RIGHT, A/D or Q/D - Move",
            "SPACE - Jump",
            "ESC - Pause",
            "M - Sound on/off",
        ];

        for (i, text) in controls_text.iter().enumerate() {
            let dims = measure_text(text, None, 20, 1.0);
            draw_text(
                text,
                screen_w * 0.5 - dims.width * 0.5,
                screen_h * 0.7 + i as f32 * 25.0,
                20.0,
                TEXT_SECONDARY,
            );
        }
    }

    pub fn draw_game_over(&mut self) {
        self.draw_playing();

        let screen_w = screen_width();
        let screen_h = screen_height();

        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.0, 0.0, 0.0, 0.9));

        // Game Over text in warning color
        let game_over_text = "GAME OVER";
        let text_dims = measure_text(game_over_text, None, 50, 1.0);
        draw_text(
            game_over_text,
            screen_w * 0.5 - text_dims.width * 0.5,
            screen_h * 0.4,
            50.0,
            TEXT_WARNING,
        );

        // Score in accent color
        let score_text = format!("Final Score: {:.0}", self.score);
        let score_dims = measure_text(&score_text, None, 30, 1.0);
        draw_text(
            &score_text,
            screen_w * 0.5 - score_dims.width * 0.5,
            screen_h * 0.5,
            30.0,
            TEXT_ACCENT,
        );

        // High Score in gold
        if self.new_record {
            let new_high_score_text = "New High Score!";
            let high_score_dims = measure_text(new_high_score_text, None, 25, 1.0);
            draw_text(
                new_high_score_text,
                screen_w * 0.5 - high_score_dims.width * 0.5,
                screen_h * 0.5 + 35.0,
                25.0,
                TEXT_GOLD,
            );
        }

        // Instructions in secondary color
        let instructions = vec![
            "Press SPACE to play again",
            "Press ESC for main menu",
        ];

        for (i, text) in instructions.iter().enumerate() {
            let dims = measure_text(text, None, 20, 1.0);
            draw_text(
                text,
                screen_w * 0.5 - dims.width * 0.5,
                screen_h * 0.6 + i as f32 * 30.0,
                20.0,
                TEXT_SECONDARY,
            );
        }
    }

    pub fn draw_lives(&self) {
        let heart_size = 20.0;
        let spacing = 5.0;
        let start_x = 725.0;
        let start_y = 10.0;

        for i in 0..INITIAL_LIVES {
            let x = start_x + (heart_size + spacing) * i as f32;
            let color = if i < self.lives { RED } else { GRAY };

            // Draw a simple heart shape
            draw_poly(x + heart_size / 2.0, start_y + heart_size / 2.0, 3, heart_size / 2.0, 0.0, color);
            draw_circle(x + heart_size / 3.0, start_y + heart_size / 3.0, heart_size / 4.0, color);
            draw_circle(x + 2.0 * heart_size / 3.0, start_y + heart_size / 3.0, heart_size / 4.0, color);
        }
    }

    pub fn draw_ui(&self) {
        // Draw basic info
        draw_text("Cha(se)dow", 10.0, 30.0, 50.0, TEXT_ACCENT);
        draw_text(&format!("Score: {:.0} / High Score: {:.0} ", self.score, self.high_score), 10.0, 60.0, 20.0, TEXT_ACCENT);

        // Draw lives
        self.draw_lives();

        // Draw invulnerability timer if active
        if self.is_invulnerable {
            draw_text(
                &format!("(invulnerability: {:.0}s)", self.invulnerable_timer),
                610.0, 45.0, 20.0, TEXT_SECONDARY,
            );
        }

        if self.shadow_freeze > 0.0 {
            draw_text(
                &format!("Shadow frozen: {:.1}s", self.shadow_freeze),
                10.0, 100.0, 20.0, TEXT_SECONDARY,
            );
        }

        if self.combo > 1 {
            draw_text(
                &format!("Combo x{}", combo_multiplier(self.combo)),
                10.0, 120.0, 20.0, TEXT_GOLD,
            );
        }

        // Add coin points to UI
        draw_text(
            &format!("Coins: {}", self.coin_points),
            10.0, 80.0, 20.0,
            TEXT_ACCENT
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combo_restarts_when_the_window_is_over() {
        assert_eq!(next_combo(0, 0.0), 1);
        assert_eq!(next_combo(4, 0.0), 1);
    }

    #[test]
    fn combo_grows_inside_the_window() {
        assert_eq!(next_combo(1, 1.5), 2);
        assert_eq!(next_combo(2, 0.1), 3);
    }

    #[test]
    fn combo_multiplier_is_capped() {
        assert_eq!(combo_multiplier(0), 1);
        assert_eq!(combo_multiplier(3), 3);
        assert_eq!(combo_multiplier(50), COMBO_MAX);
    }
}
