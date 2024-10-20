use macroquad::prelude::*;
use macroquad_platformer::*;

// Game Constants
const GRAVITY: f32 = 500.0;
const PLAYER_SPEED: f32 = 100.0;
const JUMP_FORCE: f32 = -225.0;
const PLATFORM_SPEED: f32 = 50.0;

// Size Constants
const PLAYER_SIZE: Vec2 = vec2(20.0, 20.0);
const GROUND_SIZE: Vec2 = vec2(800.0, 20.0);
const PLATFORM_SIZE: Vec2 = vec2(200.0, 20.0);

// Colors
const PLAYER_COLOR: Color = BLUE;
const PLATFORM_COLOR: Color = GREEN;
const STATIC_PLATFORM_COLOR: Color = GRAY;
const SHADOW_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.8);
const BACKGROUND_COLOR: Color = LIGHTGRAY;

// Lives system
const INITIAL_LIVES: i32 = 3;
const INVULNERABILITY_DURATION: f32 = 2.0; // Seconds of invulnerability after getting hit
const FLASH_FREQUENCY: f32 = 15.0; // Higher number = faster flashing


#[derive(PartialEq)]
enum GameScreen {
    MainMenu,
    Playing,
    Paused,
    GameOver,
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
}

impl GameState {
    fn new() -> Self {
        let mut world = World::new();
        let player = Player::new(&mut world);
        let shadow = Shadow::new(25);
        let platforms = create_platforms(&mut world);

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
        }
    }

    fn reset_game(&mut self) {
        // Update high score before resetting
        if self.score > self.high_score {
            self.high_score = self.score;
        }

        // Reset world and game elements
        self.world = World::new();
        self.player = Player::new(&mut self.world);
        self.shadow = Shadow::new(25);
        self.platforms = create_platforms(&mut self.world);
        self.score = 0.0;
        self.lives = INITIAL_LIVES;
        self.invulnerable_timer = 0.0;
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
                new_pos.y -= 50.0; // Move player up a bit to avoid immediate re-collision
                self.world.set_actor_position(self.player.collider, new_pos);
            }
        }
    }

    fn update(&mut self) {
        match self.screen {
            GameScreen::Playing => self.update_playing(),
            GameScreen::Paused => self.update_paused(),
            GameScreen::MainMenu => self.update_main_menu(),
            GameScreen::GameOver => self.update_game_over(),
        }
    }

    fn update_playing(&mut self) {
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

        let player_pos = self.world.actor_pos(self.player.collider);
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

    fn update_main_menu(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            self.reset_game();
            self.screen = GameScreen::Playing;
        }
    }

    fn update_game_over(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            self.reset_game();
            self.screen = GameScreen::Playing;
        } else if is_key_pressed(KeyCode::Escape) {
            self.screen = GameScreen::MainMenu;
        }
    }

    fn draw(&self) {
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

    fn draw_playing(&self) {
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

    fn draw_paused(&self) {
        // Draw game elements in background
        self.draw_playing();

        // Draw pause overlay
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Semi-transparent overlay
        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.0, 0.0, 0.0, 0.75));

        // Pause menu text
        let pause_text = "PAUSED";
        let text_dims = measure_text(pause_text, None, 40, 1.0);
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
        let title_text = "CHASEDOW";
        let title_dims = measure_text(title_text, None, 50, 1.0);
        draw_text(
            title_text,
            screen_w * 0.5 - title_dims.width * 0.5,
            screen_h * 0.4,
            50.0,
            WHITE,
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
                WHITE,
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
            WHITE,
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
                WHITE,
            );
        }
    }

    fn draw_game_over(&self) {
        // Draw final game state in background
        self.draw_playing();

        let screen_w = screen_width();
        let screen_h = screen_height();

        // Semi-transparent overlay
        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.0, 0.0, 0.0, 0.75));

        // Game Over text
        let game_over_text = "GAME OVER";
        let text_dims = measure_text(game_over_text, None, 50, 1.0);
        draw_text(
            game_over_text,
            screen_w * 0.5 - text_dims.width * 0.5,
            screen_h * 0.4,
            50.0,
            RED,
        );

        // Score
        let score_text = format!("Final Score: {:.0}", self.score);
        let score_dims = measure_text(&score_text, None, 30, 1.0);
        draw_text(
            &score_text,
            screen_w * 0.5 - score_dims.width * 0.5,
            screen_h * 0.5,
            30.0,
            WHITE,
        );

        // High Score
        if self.score > self.high_score {
            let new_high_score_text = "New High Score!";
            let high_score_dims = measure_text(new_high_score_text, None, 25, 1.0);
            draw_text(
                new_high_score_text,
                screen_w * 0.5 - high_score_dims.width * 0.5,
                screen_h * 0.5 + 35.0,
                25.0,
                GOLD,
            );
        }

        // Instructions
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
                WHITE,
            );
        }
    }

    fn draw_lives(&self) {
        let heart_size = 20.0;
        let spacing = 5.0;
        let start_x = 10.0;
        let start_y = 100.0;

        for i in 0..INITIAL_LIVES {
            let x = start_x + (heart_size + spacing) * i as f32;
            let color = if i < self.lives { RED } else { GRAY };

            // Draw a simple heart shape
            draw_poly(x + heart_size/2.0, start_y + heart_size/2.0, 3, heart_size/2.0, 0.0, color);
            draw_circle(x + heart_size/3.0, start_y + heart_size/3.0, heart_size/4.0, color);
            draw_circle(x + 2.0*heart_size/3.0, start_y + heart_size/3.0, heart_size/4.0, color);
        }
    }

    fn draw_ui(&self) {
        // Draw basic info
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 20.0, 20.0, WHITE);
        draw_text(&format!("Score: {:.0}", self.score), 10.0, 50.0, 20.0, WHITE);
        draw_text(&format!("High Score: {:.0}", self.high_score), 10.0, 80.0, 20.0, WHITE);

        // Draw lives
        self.draw_lives();

        // Draw invulnerability timer if active
        if self.is_invulnerable {
            draw_text(
                &format!("Invulnerable: {:.1}s", self.invulnerable_timer),
                10.0, 140.0, 20.0, YELLOW
            );
        }
    }
}

struct Player {
    collider: Actor,
    speed: Vec2,
    size: Vec2,
}

impl Player {
    fn new(world: &mut World) -> Self {
        Self {
            collider: world.add_actor(vec2(250.0, 80.0), PLAYER_SIZE.x as i32, PLAYER_SIZE.y as i32),
            speed: Vec2::ZERO,
            size: PLAYER_SIZE,
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
            self.speed.y += GRAVITY * get_frame_time();
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

    fn draw(&self, world: &World) {
        let pos = world.actor_pos(self.collider);
        draw_rectangle(pos.x, pos.y, self.size.x, self.size.y, PLAYER_COLOR);
    }
}

struct Shadow {
    positions: Vec<Vec2>,
    delay_frames: usize,
}

impl Shadow {
    fn new(delay_frames: usize) -> Self {
        Self {
            positions: vec![vec2(50.0, 100.0); delay_frames],
            delay_frames,
        }
    }

    fn update(&mut self, player_pos: Vec2) {
        self.positions.remove(0);
        self.positions.push(player_pos);
    }

    fn draw(&self) {
        if let Some(pos) = self.positions.first() {
            draw_rectangle(pos.x, pos.y, PLAYER_SIZE.x, PLAYER_SIZE.y, SHADOW_COLOR);
        }
    }

    fn collides_with_player(&self, player_pos: Vec2) -> bool {
        if let Some(shadow_pos) = self.positions.first() {
            let shadow_rect = Rect::new(shadow_pos.x, shadow_pos.y, PLAYER_SIZE.x, PLAYER_SIZE.y);
            let player_rect = Rect::new(player_pos.x, player_pos.y, PLAYER_SIZE.x, PLAYER_SIZE.y);
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
}

impl Platform {
    fn new(world: &mut World, pos: Vec2, size: Vec2, is_moving: bool) -> Self {
        Self {
            collider: world.add_solid(pos, size.x as i32, size.y as i32),
            speed: if is_moving { PLATFORM_SPEED } else { 0.0 },
            size,
        }
    }

    fn update(&mut self, world: &mut World) {
        if self.speed != 0.0 {
            world.solid_move(self.collider, self.speed * get_frame_time(), 0.0);
            let pos = world.solid_pos(self.collider);

            if (self.speed > 1.0 && pos.x >= 220.0) || (self.speed < -1.0 && pos.x <= 150.0) {
                self.speed *= -1.0;
            }
        }
    }

    fn draw(&self, world: &World) {
        let pos = world.solid_pos(self.collider);
        let color = if self.speed == 0.0 { STATIC_PLATFORM_COLOR } else { PLATFORM_COLOR };
        draw_rectangle(pos.x, pos.y, self.size.x, self.size.y, color);
    }
}

fn create_platforms(world: &mut World) -> Vec<Platform> {
    vec![
        // Ground platform
        Platform::new(world, vec2(0.0, 300.0), GROUND_SIZE, false),
        // Static platforms
        Platform::new(world, vec2(50.0, 200.0), PLATFORM_SIZE, false),
        Platform::new(world, vec2(300.0, 150.0), PLATFORM_SIZE, false),
        // Moving platform
        Platform::new(world, vec2(500.0, 250.0), PLATFORM_SIZE, true),
    ]
}

#[macroquad::main("Chasedow")]
async fn main() {
    let mut game = GameState::new();

    loop {
        game.update();
        game.draw();
        next_frame().await
    }
}