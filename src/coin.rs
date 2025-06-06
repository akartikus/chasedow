use macroquad::prelude::*;

use crate::constants::*;

#[derive(PartialEq, Clone, Copy)]
pub enum CoinKind {
    Normal,
    FreezeShadow,
}

pub struct Coin {
    pub position: Vec2,
    pub lifetime: f32,
    pub texture: Texture2D,
    pub kind: CoinKind,
}

impl Coin {
    /// The texture is loaded once by the caller and shared, so creating a coin is synchronous
    /// (blocking on an async load would freeze the game on WASM).
    pub fn new(position: Vec2, kind: CoinKind, texture: Texture2D) -> Self {
        Self {
            position,
            lifetime: COIN_LIFETIME,
            texture,
            kind,
        }
    }

    pub fn update(&mut self) -> bool {
        self.lifetime -= get_frame_time();
        self.lifetime > 0.0  // Return true if coin is still alive
    }

    pub fn draw(&self) {
        // Make coin flash when about to disappear
        if self.lifetime > 1.0 || (self.lifetime * 10.0).fract() > 0.5 {
            let tint = match self.kind {
                CoinKind::Normal => WHITE,
                CoinKind::FreezeShadow => FREEZE_COLOR,
            };
            draw_texture_ex(
                &self.texture,
                self.position.x,
                self.position.y,
                tint,
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

    pub fn collides_with_player(&self, player_pos: Vec2, player_size: Vec2) -> bool {
        let coin_rect = Rect::new(self.position.x, self.position.y, COIN_SIZE.x, COIN_SIZE.y);
        let player_rect = Rect::new(player_pos.x, player_pos.y, player_size.x, player_size.y);
        coin_rect.overlaps(&player_rect)
    }
}
