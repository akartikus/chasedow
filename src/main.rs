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

struct Game {
    player: Player,
    shadow: Shadow,
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

    fn update(&mut self, dt: f32) {
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

        // Update position
        self.pos += self.vel * dt;

        // Simple ground collision
        // fixme use real ground
        if self.pos.y > screen_height() - PLAYER_SIZE.y {
            self.pos.y = screen_height() - PLAYER_SIZE.y;
            self.vel.y = 0.0;
            self.on_ground = true;
        }
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
/*    fn collides_with_player(&self, player: &Player) -> bool {
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
    }*/
}

impl Game {
    fn new() -> Self {
        Self {
            player: Player::new(),
            shadow: Shadow::new(30),  // 0.5 second delay at 60 FPS
            score: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        self.player.update(dt);
        self.shadow.update(self.player.pos);
        self.score += dt;
    }

    fn draw(&self) {
        clear_background(GRAY);
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

            game.draw();
        }else {
            clear_background(GRAY);
            draw_text(
                &format!("Game Over! Final Score: {:.1}", game.score),
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