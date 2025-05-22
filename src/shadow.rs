use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::prelude::*;

use crate::constants::*;

/// Pure position history behind the shadow (no graphics, so it can be unit tested).
/// The shadow is at `positions[0]`: the position the player had `delay_frames` frames ago.
pub struct ShadowHistory {
    pub positions: Vec<Vec2>,
    pub last_removed_position: Vec2,
    pub trail: Vec<Vec2>, // Latest positions the shadow went through (oldest first)
    pub delay_frames: usize,
}

impl ShadowHistory {
    pub fn new(delay_frames: usize) -> Self {
        Self {
            positions: vec![vec2(50.0, 500.0); delay_frames],
            last_removed_position: vec2(50.0, 100.0),
            trail: Vec::new(),
            delay_frames,
        }
    }

    /// Shortens the delay by `n` frames, never going below `min`.
    pub fn reduce_delay(&mut self, n: usize, min: usize) {
        if self.delay_frames >= min + n {
            self.positions.drain(0..n);
            self.delay_frames -= n;
        }
    }

    /// Records the player's current position and moves the shadow one step forward.
    pub fn push(&mut self, player_pos: Vec2) {
        self.last_removed_position = self.positions.remove(0);
        self.trail.push(self.last_removed_position);
        if self.trail.len() > SHADOW_TRAIL_LEN {
            self.trail.remove(0);
        }
        self.positions.push(player_pos);
    }
}

pub struct Shadow {
    pub history: ShadowHistory,
    pub texture: Texture2D,
    pub sprite: AnimatedSprite,
}

impl Shadow {
    pub async fn new(delay_frames: usize) -> Self {
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
            history: ShadowHistory::new(delay_frames),
            texture,
            sprite,
        }
    }

    /// Shortens the delay by `n` frames, never going below `min`.
    pub fn reduce_delay(&mut self, n: usize, min: usize) {
        self.history.reduce_delay(n, min);
    }

    pub fn update(&mut self, player_pos: Vec2) {
        self.history.push(player_pos);
    }

    pub fn draw(&mut self) {

        // fixme jump animation issue
        if let Some(pos) = self.history.positions.first() {

            let is_on_ground = self.history.last_removed_position.y == pos.y;
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

            let flip_x = pos.x < self.history.last_removed_position.x;

            // Ghost copies behind the shadow, more and more transparent
            for k in 1..=3usize {
                let offset = k * SHADOW_TRAIL_SPACING;
                if self.history.trail.len() < offset {
                    break;
                }
                let ghost_pos = self.history.trail[self.history.trail.len() - offset];
                draw_texture_ex(
                    &self.texture,
                    ghost_pos.x,
                    ghost_pos.y,
                    Color::new(1.0, 1.0, 1.0, 0.25 / k as f32),
                    DrawTextureParams {
                        dest_size: Some(vec2(PLAYER_SIZE.x, PLAYER_SIZE.y)),
                        source: Some(shadow_frame.source_rect),
                        flip_x,
                        ..Default::default()
                    },
                );
            }

            draw_texture_ex(
                &self.texture,
                pos.x,
                pos.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(PLAYER_SIZE.x, PLAYER_SIZE.y)),
                    source: Some(shadow_frame.source_rect),
                    flip_x,
                    ..Default::default()
                },
            );

            //fixme just for debug
            // draw_rectangle_lines(pos.x, pos.y, PLAYER_SIZE.x, PLAYER_SIZE.y, 3., PLAYER_COLOR);
        }
    }

    pub fn collides_with_player(&self, player_pos: Vec2) -> bool {
        if let Some(shadow_pos) = self.history.positions.first() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_replays_player_position_after_delay() {
        let mut h = ShadowHistory::new(3);
        // The first 3 pushes only consume the initial positions
        for i in 1..=3 {
            h.push(vec2(i as f32, 0.0));
        }
        // The shadow is now at the position the player had 3 frames ago
        assert_eq!(h.positions[0], vec2(1.0, 0.0));
        h.push(vec2(4.0, 0.0));
        assert_eq!(h.positions[0], vec2(2.0, 0.0));
    }

    #[test]
    fn push_keeps_the_history_length_constant() {
        let mut h = ShadowHistory::new(10);
        for i in 0..50 {
            h.push(vec2(i as f32, 0.0));
            assert_eq!(h.positions.len(), 10);
        }
    }

    #[test]
    fn trail_is_capped() {
        let mut h = ShadowHistory::new(5);
        for i in 0..100 {
            h.push(vec2(i as f32, 0.0));
        }
        assert_eq!(h.trail.len(), SHADOW_TRAIL_LEN);
    }

    #[test]
    fn reduce_delay_shortens_history_and_delay_together() {
        let mut h = ShadowHistory::new(75);
        h.reduce_delay(5, 30);
        assert_eq!(h.delay_frames, 70);
        assert_eq!(h.positions.len(), 70);
    }

    #[test]
    fn reduce_delay_never_goes_below_the_minimum() {
        let mut h = ShadowHistory::new(75);
        for _ in 0..100 {
            h.reduce_delay(5, 30);
        }
        assert_eq!(h.delay_frames, 30);
        assert_eq!(h.positions.len(), 30);
    }
}
