use futures::executor::block_on;
use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::prelude::*;
use macroquad_platformer::*;
use macroquad::rand::*;
use macroquad::audio::*;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

// Game Constants
const GRAVITY: f32 = 500.0;
const PLAYER_SPEED: f32 = 150.0;
const JUMP_FORCE: f32 = -350.0;
const PLATFORM_SPEED: f32 = 50.0;
const SHADOW_FRAMES_DELAY: usize = 75;

// Size Constants
const PLAYER_SIZE: Vec2 = vec2(12.0 *4., 12.0 *4.);
const GROUND_SIZE: Vec2 = vec2(800.0, 12.0);
const PLATFORM_SIZE: Vec2 = vec2(200.0, 12.0);

// Colors
const PLAYER_COLOR: Color = Color::new(0.45, 0.26, 0.20, 1.0);  // Rustic brown for player
const PLATFORM_COLOR: Color = Color::new(0.76, 0.60, 0.42, 1.0);  // Sandy beige for moving platforms
const STATIC_PLATFORM_COLOR: Color = Color::new(0.87, 0.68, 0.45, 1.0);  // Light sand for static platforms
const SHADOW_COLOR: Color = Color::new(0.2, 0.1, 0.05, 0.6);  // Dark sepia shadow
const BACKGROUND_COLOR: Color = Color::new(0.98, 0.90, 0.75, 1.0);  // Bright, warm sunshine yellow

// Text colors for different purposes
const TEXT_PRIMARY: Color = Color::new(0.45, 0.26, 0.20, 1.0);    // Deep brown - for main text
const TEXT_SECONDARY: Color = Color::new(0.65, 0.35, 0.25, 1.0);  // Lighter brown - for less important info
const TEXT_ACCENT: Color = Color::new(0.8, 0.4, 0.2, 1.0);        // Terracotta - for highlights/scores
const TEXT_WARNING: Color = Color::new(0.7, 0.3, 0.2, 1.0);       // Reddish brown - for warnings/game over
const TEXT_GOLD: Color = Color::new(0.85, 0.6, 0.2, 1.0);         // Desert gold - for high scores

// Lives system
const INITIAL_LIVES: i32 = 3;
const INVULNERABILITY_DURATION: f32 = 3.0; // Seconds of invulnerability after getting hit
const FLASH_FREQUENCY: f32 = 10.0; // Higher number = faster flashing

// Add these constants at the top
const COIN_SIZE: Vec2 = vec2(12.0 * 3., 12.0 * 3.);  // Scale the 12x12 sprite by 3
const COIN_SPAWN_INTERVAL: f32 = 3.0;  // Spawn a new coin every 3 seconds
const COIN_LIFETIME: f32 = 5.0;  // Coins disappear after 5 seconds
const COIN_POINTS: i32 = 10;     // Points earned per coin


#[derive(PartialEq)]
enum GameScreen {
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

struct GameAudio {
    background_music: Sound,
    // jump_sound: Sound,
    // game_over_sound: Sound,
}

impl GameAudio {
    async fn new() -> Self {
        set_pc_assets_folder("assets");
        Self {
            background_music: load_sound("background.ogg").await.expect("Failed to load background music"),
            // jump_sound: load_sound("jump.ogg").await.expect("Failed to load jump sound"),
            // game_over_sound: load_sound("game_over.ogg").await.expect("Failed to load game over sound"),
        }
    }

    fn play_background(&self) {
        // if !is_sound_playing(&self.background_music) {
            play_sound(&self.background_music, PlaySoundParams {
                looped: true,
                volume: 0.5,
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

    fn stop_all(&self) {
        stop_sound(&self.background_music);
        // stop_audio(&self.jump_sound);
        // stop_audio(&self.game_over_sound);
    }
}

// Game State
struct GameState {
    world: World,
    player: Player,
    shadow: Shadow,
    platforms: Vec<Platform>,
    score: f32,
    screen: GameScreen,
    high_score: f32,
    lives: i32,
    invulnerable_timer: f32,
    is_invulnerable: bool,
    audio: GameAudio,
    coins: Vec<Coin>,
    coin_spawn_timer: f32,
    coin_points: i32,
}

impl GameState {
    async fn new() -> Self {
        let mut world = World::new();
        let player = Player::new(&mut world).await;
        let shadow = Shadow::new(SHADOW_FRAMES_DELAY).await;
        let platforms = create_platforms(&mut world).await;
        let audio = GameAudio::new().await;

        Self {
            world,
            player,
            shadow,
            platforms,
            score: 0.0,
            screen: GameScreen::MainMenu,
            high_score: 0.0,
            lives: INITIAL_LIVES,
            invulnerable_timer: 0.0,
            is_invulnerable: false,
            audio,
            coins: Vec::new(),
            coin_spawn_timer: 0.0,
            coin_points: 0,
        }
    }

    async fn reset_game(&mut self) {
        // Update high score before resetting
        if self.score > self.high_score {
            self.high_score = self.score;
        }

        // Reset world and game elements
        self.world = World::new();
        self.player = Player::new(&mut self.world).await;
        self.shadow = Shadow::new(25).await;
        self.platforms = create_platforms(&mut self.world).await;
        self.score = 0.0;
        self.lives = INITIAL_LIVES;
        self.invulnerable_timer = 0.0;

        self.coins.clear();
        self.coin_spawn_timer = 0.0;
        self.coin_points = 0;
    }

    fn handle_shadow_collision(&mut self) {
        if self.invulnerable_timer <= 0.0 {
            self.lives -= 1;
            if self.lives <= 0 {
                self.screen = GameScreen::GameOver;
            } else {
                // Start invulnerability period
                self.invulnerable_timer = INVULNERABILITY_DURATION;
                // Optional: Reset player position after hit
                let mut new_pos = self.world.actor_pos(self.player.collider);
                // new_pos.y -= 50.0; // Move player up a bit to avoid immediate re-collision
                self.world.set_actor_position(self.player.collider, new_pos);
            }
        }
    }

    async fn update(&mut self) {
        match self.screen {
            GameScreen::Playing => self.update_playing(),
            GameScreen::Paused => self.update_paused(),
            GameScreen::MainMenu => self.update_main_menu().await,
            GameScreen::GameOver => self.update_game_over().await,
        }
    }

    fn spawn_coin(&mut self) {
        // Random position within window bounds
        let x = gen_range(0.0, WINDOW_WIDTH - COIN_SIZE.x);
        let y = gen_range(100.0, WINDOW_HEIGHT - COIN_SIZE.y - 50.0);  // Keep above ground level

        // Spawn the coin
        block_on(async {
            let coin = Coin::new(vec2(x, y)).await;
            self.coins.push(coin);
        });
    }

    fn update_playing(&mut self) {
        // fixme https://github.com/not-fl3/macroquad/issues/440 ???
        // self.audio.play_background();

        // Update coin spawn timer
        self.coin_spawn_timer -= get_frame_time();
        if self.coin_spawn_timer <= 0.0 {
            self.spawn_coin();
            self.coin_spawn_timer = COIN_SPAWN_INTERVAL;
        }

        // Update existing coins
        let player_pos = self.world.actor_pos(self.player.collider);
        let mut i = 0;
        while i < self.coins.len() {
            if !self.coins[i].update() {
                self.coins.remove(i);
            } else if self.coins[i].collides_with_player(player_pos, PLAYER_SIZE) {
                self.coin_points += COIN_POINTS;
                self.coins.remove(i);
            } else {
                i += 1;
            }
        }

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
        for platform in self.platforms.iter_mut() {
            platform.update(&mut self.world);
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

        self.shadow.update(player_pos);

        // Check for collision with shadow
        if self.shadow.collides_with_player(player_pos) {
            self.is_invulnerable = true;
            self.handle_shadow_collision();
        }

        self.score += get_frame_time();
    }

    fn update_paused(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = GameScreen::Playing;
        }
    }

    async fn update_main_menu(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            self.reset_game().await;
            self.screen = GameScreen::Playing;
        }
    }

    async fn update_game_over(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            self.reset_game().await;
            self.screen = GameScreen::Playing;
        } else if is_key_pressed(KeyCode::Escape) {
            self.screen = GameScreen::MainMenu;
        }
    }

    fn draw(&mut self) {
        clear_background(BACKGROUND_COLOR);

        match self.screen {
            GameScreen::Playing => self.draw_playing(),
            GameScreen::Paused => self.draw_paused(),
            GameScreen::MainMenu => self.draw_main_menu(),
            GameScreen::GameOver => self.draw_game_over(),
        }
    }


    fn should_draw_player(&self) -> bool {
        if !self.is_invulnerable {
            return true;
        }
        // Create a flashing effect based on time
        (self.invulnerable_timer * FLASH_FREQUENCY).sin() > 0.0
    }

    fn draw_playing(&mut self) {
        // Draw coins
        for coin in &self.coins {
            coin.draw();
        }

        // Draw game elements
        for platform in &self.platforms {
            platform.draw(&self.world);
        }
        self.shadow.draw();

        // Draw player with flashing effect when invulnerable
        if self.should_draw_player() {
            self.player.draw(&self.world);
        }

        self.draw_ui();
    }

    fn draw_paused(&mut self) {
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

    fn draw_main_menu(&self) {
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
            "LEFT/RIGHT - Move",
            "SPACE - Jump",
            "ESC - Pause",
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

    fn draw_game_over(&mut self) {
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
        if self.score > self.high_score {
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

    fn draw_lives(&self) {
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

    fn draw_ui(&self) {
        // Draw basic info
        draw_text("Cha(se)down", 10.0, 30.0, 50.0, TEXT_ACCENT);
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

        // Add coin points to UI
        draw_text(
            &format!("Coins: {}", self.coin_points),
            10.0, 80.0, 20.0,
            TEXT_ACCENT
        );
    }
}

struct Player {
    collider: Actor,
    speed: Vec2,
    size: Vec2,
    texture: Texture2D,
    sprite: AnimatedSprite,
}

impl Player {
    async fn new(world: &mut World) -> Self {
        set_pc_assets_folder("assets");
        let texture = load_texture("player.png").await.expect("Couldn't load player texture");
        texture.set_filter(FilterMode::Nearest);
        let mut sprite = AnimatedSprite::new(
            12,
            12,
            &[
                Animation {
                    name: "walk".to_string(),
                    row: 0,
                    frames: 6,
                    fps: 12,
                },
                Animation {
                    name: "jump".to_string(),
                    row: 2,
                    frames: 3,
                    fps: 12,
                },
            ],
            true,
        );
        sprite.set_animation(0);

        Self {
            collider: world.add_actor(vec2(250.0, 500.0), PLAYER_SIZE.x as i32, PLAYER_SIZE.y as i32),
            speed: Vec2::ZERO,
            size: PLAYER_SIZE,
            texture,
            sprite,
        }
    }

    fn update(&mut self, world: &mut World) {
        let pos = world.actor_pos(self.collider);
        let on_ground = world.collide_check(self.collider, pos + vec2(0., 1.));

        self.handle_movement(on_ground);
        self.apply_movement(world);
    }

    fn handle_movement(&mut self, on_ground: bool) {
        // Apply gravity when in air
        if !on_ground {
            self.sprite.set_animation(1);
            self.speed.y += GRAVITY * get_frame_time();
        } else {
            self.sprite.set_animation(0);
        }

        // Handle horizontal movement
        self.speed.x = match (is_key_down(KeyCode::Right), is_key_down(KeyCode::Left)) {
            (true, false) => PLAYER_SPEED,
            (false, true) => -PLAYER_SPEED,
            _ => 0.0,
        };

        // Handle jumping
        if is_key_pressed(KeyCode::Space) && on_ground {
            self.speed.y = JUMP_FORCE;
        }
    }

    fn apply_movement(&mut self, world: &mut World) {
        world.move_h(self.collider, self.speed.x * get_frame_time());
        world.move_v(self.collider, self.speed.y * get_frame_time());
    }

    fn draw(&mut self, world: &World) {
        let player_frame = self.sprite.frame();

        // Do not update to next frame if :
        let is_last_jump_frame = player_frame.source_rect.x == 16.*2. && player_frame.source_rect.y == 16.*2. && self.speed.y != 0.0;

        if !(is_last_jump_frame) {
            self.sprite.update();
        }

        let pos = world.actor_pos(self.collider);
        draw_texture_ex(
            &self.texture,
            pos.x,
            pos.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.size.x, self.size.y)),
                source: Some(player_frame.source_rect),
                flip_x: self.speed.x <= 0.,
                ..Default::default()
            },
        );

        //fixme just for debug
        draw_rectangle_lines(pos.x, pos.y, self.size.x, self.size.y, 2., PLAYER_COLOR);
    }
}

struct Shadow {
    positions: Vec<Vec2>,
    last_removed_position: Vec2,
    delay_frames: usize,
    texture: Texture2D,
    sprite: AnimatedSprite,
}

impl Shadow {
    async fn new(delay_frames: usize) -> Self {
        set_pc_assets_folder("assets");
        let texture = load_texture("player.png").await.expect("Couldn't load player texture");
        texture.set_filter(FilterMode::Nearest);
        let mut sprite = AnimatedSprite::new(
            12,
            12,
            &[
                Animation {
                    name: "walk".to_string(),
                    row: 1,
                    frames: 6,
                    fps: 12,
                },
                Animation {
                    name: "jump".to_string(),
                    row: 3,
                    frames: 3,
                    fps: 12,
                },
            ],
            true,
        );
        sprite.set_animation(0);

        Self {
            positions: vec![vec2(50.0, 500.0); delay_frames],
            last_removed_position: vec2(50.0, 100.0),
            delay_frames,
            texture,
            sprite,
        }
    }

    fn update(&mut self, player_pos: Vec2) {
        self.last_removed_position = self.positions.remove(0);
        self.positions.push(player_pos);

    }

    fn draw(&mut self) {

        // fixme jump animation issue
        if let Some(pos) = self.positions.first() {

            let is_on_ground = self.last_removed_position.y == pos.y;
            if is_on_ground {
                self.sprite.set_animation(0);
            }else{
                self.sprite.set_animation(1);
            }

            let shadow_frame = self.sprite.frame();

            // Do not update to next frame if :
            let is_last_jump_frame = shadow_frame.source_rect.x == 16.*2. && shadow_frame.source_rect.y == 16.*3. && !is_on_ground;

            if !(is_last_jump_frame) {
                self.sprite.update();
            }

            draw_texture_ex(
                &self.texture,
                pos.x,
                pos.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(PLAYER_SIZE.x, PLAYER_SIZE.y)),
                    source: Some(shadow_frame.source_rect),
                    flip_x: pos.x < self.last_removed_position.x,
                    ..Default::default()
                },
            );

            //fixme just for debug
            draw_rectangle_lines(pos.x, pos.y, PLAYER_SIZE.x, PLAYER_SIZE.y, 3., PLAYER_COLOR);
        }
    }

    fn collides_with_player(&self, player_pos: Vec2) -> bool {
        if let Some(shadow_pos) = self.positions.first() {
            //fixme fix sprite sheet file, remove margin
            let no_margin_x = PLAYER_SIZE.x - 4. * 4.;
            let no_margin_y = PLAYER_SIZE.y - 4. * 4.;
            let shadow_rect = Rect::new(shadow_pos.x, shadow_pos.y, no_margin_x, no_margin_y);
            let player_rect = Rect::new(player_pos.x, player_pos.y, no_margin_x, no_margin_y);
            shadow_rect.overlaps(&player_rect)
        } else {
            false
        }
    }
}

struct Platform {
    collider: Solid,
    speed: f32,
    size: Vec2,
    cactus_texture: Texture2D,
    // Store both position and size for each cactus
    cacti: Vec<(f32, f32)>, // (position, size)
}

impl Platform {
    async fn new(world: &mut World, pos: Vec2, size: Vec2, is_moving: bool) -> Self {
        set_pc_assets_folder("assets");
        let cactus_texture: Texture2D = load_texture("player.png").await.unwrap();
        cactus_texture.set_filter(FilterMode::Nearest);

        // Randomly decide to place 1 or 2 cacti
        let num_cacti = gen_range(1, 3);

        // Generate random positions and sizes along the platform
        let mut cacti = Vec::new();
        let min_size = 12.0 * 3.0; // Minimum size (36 pixels)
        let max_size = 12.0 * 5.0; // Maximum size (60 pixels)

        for _ in 0..num_cacti {
            let cactus_size = gen_range(min_size, max_size);
            let x_offset = gen_range(pos.x, pos.x + size.x - cactus_size);
            cacti.push((x_offset, cactus_size));
        }

        Self {
            collider: world.add_solid(pos, size.x as i32, size.y as i32),
            speed: if is_moving { PLATFORM_SPEED } else { 0.0 },
            size,
            cactus_texture,
            cacti,
        }
    }

    fn update(&mut self, world: &mut World) {
        if self.speed != 0.0 {
            world.solid_move(self.collider, self.speed * get_frame_time(), 0.0);
            let pos = world.solid_pos(self.collider);

            if (self.speed > 1.0 && pos.x >= 500.0) || (self.speed < -1.0 && pos.x <= 150.0) {
                self.speed *= -1.0;
            }
        }
    }

    fn draw(&self, world: &World) {
        let pos = world.solid_pos(self.collider);
        let color = if self.speed == 0.0 { STATIC_PLATFORM_COLOR } else { PLATFORM_COLOR };

        // Draw platform
        draw_rectangle(pos.x, pos.y, self.size.x, self.size.y, color);

        // Draw cacti
        if self.speed == 0.0 {
            for &(x_offset, cactus_size) in &self.cacti {
                let size = vec2(cactus_size, cactus_size);
                draw_texture_ex(
                    &self.cactus_texture,
                    x_offset,  // x position with offset
                    pos.y - size.x,  // y position
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(size),
                        source: Some(Rect::new(
                            3.0 * 12.0,
                            2.0 * 12.0,
                            12.0,
                            12.0,
                        )),
                        ..Default::default()
                    },
                );
            }
        }
    }
}

async fn create_platforms(world: &mut World) -> Vec<Platform> {
    vec![
        // Moving platform
        Platform::new(world, vec2(100.0, 100.0), PLATFORM_SIZE, true).await,

        // Static platforms
        Platform::new(world, vec2(50.0, 200.0), PLATFORM_SIZE, false).await,
        Platform::new(world, vec2(550.0, 200.0), PLATFORM_SIZE, false).await,

        Platform::new(world, vec2(300.0, 300.0), PLATFORM_SIZE, false).await,

        Platform::new(world, vec2(50.0, 400.0), PLATFORM_SIZE, false).await,
        Platform::new(world, vec2(550.0, 400.0), PLATFORM_SIZE, false).await,

        // Moving platform
        Platform::new(world, vec2(500.0, 500.0), PLATFORM_SIZE, true).await,

        // Ground platform
        Platform::new(world, vec2(0.0, 585.0), GROUND_SIZE, false).await,
    ]
}

struct Coin {
    position: Vec2,
    lifetime: f32,
    texture: Texture2D,
}

impl Coin {
    async fn new(position: Vec2) -> Self {
        set_pc_assets_folder("assets");
        let texture = load_texture("player.png").await.expect("Couldn't load player texture");
        texture.set_filter(FilterMode::Nearest);

        Self {
            position,
            lifetime: COIN_LIFETIME,
            texture,
        }
    }

    fn update(&mut self) -> bool {
        self.lifetime -= get_frame_time();
        self.lifetime > 0.0  // Return true if coin is still alive
    }

    fn draw(&self) {
        // Make coin flash when about to disappear
        if self.lifetime > 1.0 || (self.lifetime * 10.0).fract() > 0.5 {
            draw_texture_ex(
                &self.texture,
                self.position.x,
                self.position.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(COIN_SIZE),
                    source: Some(Rect::new(
                        4.0 * 12.0,  // X position in spritesheet
                        2.0 * 12.0,  // Y position in spritesheet
                        12.0,        // Width of sprite
                        12.0,        // Height of sprite
                    )),
                    ..Default::default()
                },
            );
        }
    }

    fn collides_with_player(&self, player_pos: Vec2, player_size: Vec2) -> bool {
        let coin_rect = Rect::new(self.position.x, self.position.y, COIN_SIZE.x, COIN_SIZE.y);
        let player_rect = Rect::new(player_pos.x, player_pos.y, player_size.x, player_size.y);
        coin_rect.overlaps(&player_rect)
    }
}


#[macroquad::main("Chasedow")]
async fn main() {
    let mut game = GameState::new().await;

    loop {
        game.update().await;
        game.draw();
        next_frame().await
    }
}