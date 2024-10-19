use macroquad::input::KeyCode::{Left, Right, Space};
use macroquad::prelude::*;

const PLAYER_SPEED: f32 = 200.0;
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 30.0);
const JUMP_FORCE: f32 = -300.0;
const GRAVITY: f32 = 800.0;

struct Player {
    pos: Vec2,
    vel: Vec2,
    on_ground: bool,
}

struct Shadow {
    positions: Vec<Vec2>,
    delay_frames: usize,
}

struct Platform {
    rect: Rect,
    color: Color,
}

struct Game {
    player: Player,
    shadow: Shadow,
    platforms: Vec<Platform>,
    score: f32,
}

impl Player {
    fn new() -> Self {
        Self {
            pos: Vec2::new(100., 100.),
            vel: Default::default(),
            on_ground: false,
        }
    }

    fn update(&mut self, dt: f32, platforms: &[Platform]) {
        // Horizontal movement
        let mut input = 0.;
        if is_key_down(Right) { input += 1.; }
        if is_key_down(Left) { input -= 1.; }

        self.vel.x = input * PLAYER_SPEED;

        // Jumping
        if is_key_pressed(Space) && self.on_ground {
            self.on_ground = false;
            self.vel.y = JUMP_FORCE;
        }

        // Apply gravity
        if !self.on_ground {
            self.vel.y += GRAVITY * dt; // is dt necessary here
        }

        // Update position and handle collisions
        let new_pos = self.pos + self.vel * dt;
        // fixme
        self.handle_collision(new_pos, platforms);

        // Simple ground collision
        // fixme use real ground
        if self.pos.y > screen_height() - PLAYER_SIZE.y {
            self.pos.y = screen_height() - PLAYER_SIZE.y;
            self.vel.y = 0.0;
            self.on_ground = true;
        }
    }

    fn handle_collision(&mut self, new_pos: Vec2, platforms: &[Platform]) {
        let mut final_pos = new_pos;
        self.on_ground = false;

        let player_rect = Rect::new(
            new_pos.x,
            new_pos.y,
            PLAYER_SIZE.x,
            PLAYER_SIZE.y,
        );

        // Check collision with each platform
        for platform in platforms {
            if player_rect.overlaps(&platform.rect) {
                // Determine which side of the platform we hit
                let prev_rect = Rect::new(
                    self.pos.x,
                    self.pos.y,
                    PLAYER_SIZE.x,
                    PLAYER_SIZE.y,
                );

                // Vertical collision
                if self.vel.y > 0.0 && prev_rect.bottom() <= platform.rect.y {
                    // Landing on top of platform
                    final_pos.y = platform.rect.y - PLAYER_SIZE.y;
                    self.vel.y = 0.0;
                    self.on_ground = true;
                } else if self.vel.y < 0.0 && prev_rect.top() >= platform.rect.bottom() {
                    // Hitting platform from below
                    final_pos.y = platform.rect.bottom();
                    self.vel.y = 0.0;
                }
                // Horizontal collision
                else if prev_rect.right() <= platform.rect.x {
                    // Hitting platform from left
                    final_pos.x = platform.rect.x - PLAYER_SIZE.x;
                } else if prev_rect.x >= platform.rect.right() {
                    // Hitting platform from right
                    final_pos.x = platform.rect.right();
                }
            }
        }

        // Ground collision (bottom of screen)
        if final_pos.y > screen_height() - PLAYER_SIZE.y {
            final_pos.y = screen_height() - PLAYER_SIZE.y;
            self.vel.y = 0.0;
            self.on_ground = true;
        }

        self.pos = final_pos;
    }

    fn draw(&self) {
        draw_rectangle(
            self.pos.x,
            self.pos.y,
            PLAYER_SIZE.x,
            PLAYER_SIZE.y,
            WHITE
        );
    }
}

impl Shadow {
    fn new(delay_frames: usize) -> Self {
        Self {
            positions: vec![Vec2::new(50.0, 100.0); delay_frames],
            delay_frames,
        }
    }

    fn update(&mut self, player_pos: Vec2) {
        self.positions.remove(0);
        self.positions.push(player_pos);
    }

    fn draw(&self) {
        if let Some(pos) = self.positions.first() {
            draw_rectangle(
                pos.x,
                pos.y,
                PLAYER_SIZE.x,
                PLAYER_SIZE.y,
                Color::new(0.0, 0.0, 0.0, 0.8)
            );
        }
    }

    //todo collisions
    fn collides_with_player(&self, player: &Player) -> bool {
        if let Some(shadow_pos) = self.positions.first() {
            let shadow_rect = Rect::new(
                shadow_pos.x,
                shadow_pos.y,
                PLAYER_SIZE.x,
                PLAYER_SIZE.y
            );
            let player_rect = Rect::new(
                player.pos.x,
                player.pos.y,
                PLAYER_SIZE.x,
                PLAYER_SIZE.y
            );
            shadow_rect.overlaps(&player_rect)
        } else {
            false
        }
    }
}

impl Platform {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            rect: Rect::new(x, y, width, height),
            color: DARKGRAY,
        }
    }

    fn draw(&self) {
        draw_rectangle(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            self.rect.h,
            self.color,
        );
    }
}

impl Game {
    fn new() -> Self {
        // Create some example platforms
        let mut platforms = Vec::new();

        // Add some platforms for a basic level
        platforms.push(Platform::new(300.0, 300.0, 200.0, 20.0));  // Long platform
        platforms.push(Platform::new(500.0, 250.0, 100.0, 20.0));  // Short platform
        platforms.push(Platform::new(300.0, 200.0, 150.0, 20.0));  // High platform
        platforms.push(Platform::new(50.0, 350.0, 700.0, 20.0));   // Ground platform

        Self {
            player: Player::new(),
            shadow: Shadow::new(30),
            platforms,
            score: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        self.player.update(dt, &self.platforms);
        self.shadow.update(self.player.pos);
        self.score += dt;
    }

    fn draw(&self) {
        clear_background(GRAY);

        // Draw platforms
        for platform in &self.platforms {
            platform.draw();
        }

        self.player.draw();
        self.shadow.draw();

        // Draw score
        draw_text(
            &format!("Score: {:.1}", self.score),
            10.0,
            30.0,
            30.0,
            WHITE
        );
    }

}

#[macroquad::main("Chasedow runner")]
async fn main() {
    let mut game = Game::new();
    let mut game_over = false;

    loop {
        if !game_over {
            game.update(get_frame_time());

            // fixme
            // if game.shadow.collides_with_player(&game.player) {
            //     game_over = true;
            // }

            game.draw();
        }else {
            clear_background(GRAY);
            draw_text(
                &format!("Game Over! Final Score: {:.1}, press R to restart", game.score),
                screen_width() / 4.0,
                screen_height() / 2.0,
                50.0,
                WHITE
            );

            if is_key_pressed(KeyCode::R) {
                game = Game::new();
                game_over = false;
            }
        }
        next_frame().await;
    }
}