use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::prelude::*;
use macroquad_platformer::*;

use crate::constants::*;

pub struct Player {
    pub collider: Actor,
    pub speed: Vec2,
    pub size: Vec2,
    pub texture: Texture2D,
    pub sprite: AnimatedSprite,
    pub coyote_timer: f32,
    pub jump_buffer: f32,
}

impl Player {
    pub async fn new(world: &mut World) -> Self {
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
            coyote_timer: 0.0,
            jump_buffer: 0.0,
        }
    }

    pub fn update(&mut self, world: &mut World) {
        let pos = world.actor_pos(self.collider);
        let on_ground = world.collide_check(self.collider, pos + vec2(0., 1.));

        self.handle_movement(on_ground);
        self.apply_movement(world);
    }

    pub fn handle_movement(&mut self, on_ground: bool) {
        // Apply gravity when in air
        if !on_ground {
            self.sprite.set_animation(1);
            self.speed.y += GRAVITY * get_frame_time();
        } else {
            self.sprite.set_animation(0);
        }

        // Handle horizontal movement
        // Arrow keys, WASD (QWERTY) and ZQSD (AZERTY) all work
        let right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
        let left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) || is_key_down(KeyCode::Q);
        self.speed.x = match (right, left) {
            (true, false) => PLAYER_SPEED,
            (false, true) => -PLAYER_SPEED,
            _ => 0.0,
        };

        // Handle jumping (with coyote time and jump buffering)
        let dt = get_frame_time();
        if on_ground {
            self.coyote_timer = COYOTE_TIME;
        } else {
            self.coyote_timer -= dt;
        }
        if is_key_pressed(KeyCode::Space) {
            self.jump_buffer = JUMP_BUFFER_TIME;
        } else {
            self.jump_buffer -= dt;
        }
        if self.jump_buffer > 0.0 && self.coyote_timer > 0.0 {
            self.speed.y = JUMP_FORCE;
            // Consume both timers so we can't jump twice in a row
            self.jump_buffer = 0.0;
            self.coyote_timer = 0.0;
        }
    }

    pub fn apply_movement(&mut self, world: &mut World) {
        world.move_h(self.collider, self.speed.x * get_frame_time());
        world.move_v(self.collider, self.speed.y * get_frame_time());
    }

    pub fn draw(&mut self, world: &World) {
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
        // draw_rectangle_lines(pos.x, pos.y, self.size.x, self.size.y, 2., PLAYER_COLOR);
    }
}
