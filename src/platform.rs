use macroquad::prelude::*;
use macroquad::rand::*;
use macroquad_platformer::*;

use crate::constants::*;

/// Life cycle of a fragile platform. The f32 is the time left in the state.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FadeState {
    Solid,
    Warning(f32),
    Gone(f32),
}

/// Pure transition function of a fragile platform:
/// - Solid -> Warning when the player stands on it
/// - Warning -> Gone when the warning time is over
/// - Gone -> Solid when the time is over AND the player is not where it comes back
///   (otherwise the player would be stuck inside the platform)
pub fn next_fade_state(state: FadeState, dt: f32, stood_on: bool, area_clear: bool) -> FadeState {
    match state {
        FadeState::Solid => {
            if stood_on { FadeState::Warning(FADE_WARNING_TIME) } else { FadeState::Solid }
        }
        FadeState::Warning(t) => {
            if t - dt <= 0.0 { FadeState::Gone(FADE_GONE_TIME) } else { FadeState::Warning(t - dt) }
        }
        FadeState::Gone(t) => {
            let t = (t - dt).max(0.0);
            if t == 0.0 && area_clear { FadeState::Solid } else { FadeState::Gone(t) }
        }
    }
}

pub struct Platform {
    pub collider: Solid,
    pub fading: bool,
    pub fade_state: FadeState,
    pub home: Vec2, // Original position, used to bring a fragile platform back
    pub speed: f32,
    pub size: Vec2,
    pub cactus_texture: Texture2D,
    // Store both position and size for each cactus
    pub cacti: Vec<(f32, f32)>, // (position, size)
}

impl Platform {
    pub async fn new(world: &mut World, pos: Vec2, size: Vec2, is_moving: bool) -> Self {
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
            fading: false,
            fade_state: FadeState::Solid,
            home: pos,
            speed: if is_moving { PLATFORM_SPEED } else { 0.0 },
            size,
            cactus_texture,
            cacti,
        }
    }

    /// Makes this platform fragile: it vanishes shortly after the player lands on it.
    pub fn fading(mut self) -> Self {
        self.fading = true;
        self
    }

    /// True if the player's feet are on top of the platform.
    fn is_stood_on(&self, world: &World, player_pos: Vec2) -> bool {
        let pos = world.solid_pos(self.collider);
        let feet = player_pos.y + PLAYER_SIZE.y;
        (feet - pos.y).abs() <= 2.0
            && player_pos.x + PLAYER_SIZE.x > pos.x
            && player_pos.x < pos.x + self.size.x
    }

    /// True if the player is not overlapping the place where the platform reappears.
    fn area_clear(&self, player_pos: Vec2) -> bool {
        let area = Rect::new(self.home.x - 4.0, self.home.y - 4.0, self.size.x + 8.0, self.size.y + 8.0);
        let player = Rect::new(player_pos.x, player_pos.y, PLAYER_SIZE.x, PLAYER_SIZE.y);
        !area.overlaps(&player)
    }

    fn update_fading(&mut self, world: &mut World, player_pos: Vec2) {
        let new_state = next_fade_state(
            self.fade_state,
            get_frame_time(),
            self.is_stood_on(world, player_pos),
            self.area_clear(player_pos),
        );
        // The physics world cannot disable a solid, so a vanished platform is parked off screen
        match (self.fade_state, new_state) {
            (FadeState::Warning(_), FadeState::Gone(_)) => {
                world.solid_move(self.collider, 0.0, FADE_OFFSCREEN_OFFSET);
            }
            (FadeState::Gone(_), FadeState::Solid) => {
                world.solid_move(self.collider, 0.0, -FADE_OFFSCREEN_OFFSET);
            }
            _ => {}
        }
        self.fade_state = new_state;
    }

    pub fn update(&mut self, world: &mut World, player_pos: Vec2) {
        if self.fading {
            self.update_fading(world, player_pos);
        }
        if self.speed != 0.0 {
            world.solid_move(self.collider, self.speed * get_frame_time(), 0.0);
            let pos = world.solid_pos(self.collider);

            if (self.speed > 1.0 && pos.x >= 500.0) || (self.speed < -1.0 && pos.x <= 150.0) {
                self.speed *= -1.0;
            }
        }
    }

    pub fn draw(&self, world: &World) {
        // A vanished platform is not drawn at all
        let alpha = match self.fade_state {
            FadeState::Gone(_) => return,
            // Blink during the warning
            FadeState::Warning(t) => if (t * 14.0).sin() > 0.0 { 0.45 } else { 1.0 },
            FadeState::Solid => 1.0,
        };
        let pos = world.solid_pos(self.collider);
        let mut color = if self.fading {
            FADING_PLATFORM_COLOR
        } else if self.speed == 0.0 {
            STATIC_PLATFORM_COLOR
        } else {
            PLATFORM_COLOR
        };
        color.a = alpha;

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
                    Color::new(1.0, 1.0, 1.0, alpha),
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

pub async fn create_platforms(world: &mut World) -> Vec<Platform> {
    vec![
        // Moving platform
        Platform::new(world, vec2(100.0, 100.0), PLATFORM_SIZE, true).await,

        // Static platforms
        Platform::new(world, vec2(50.0, 200.0), PLATFORM_SIZE, false).await,
        Platform::new(world, vec2(550.0, 200.0), PLATFORM_SIZE, false).await,

        // Fragile platform: vanishes shortly after the player lands on it
        Platform::new(world, vec2(300.0, 300.0), PLATFORM_SIZE, false).await.fading(),

        Platform::new(world, vec2(50.0, 400.0), PLATFORM_SIZE, false).await,
        // Fragile platform
        Platform::new(world, vec2(550.0, 400.0), PLATFORM_SIZE, false).await.fading(),

        // Moving platform
        Platform::new(world, vec2(500.0, 500.0), PLATFORM_SIZE, true).await,

        // Ground platform
        Platform::new(world, vec2(0.0, 585.0), GROUND_SIZE, false).await,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_platform_stays_solid_until_stood_on() {
        assert_eq!(next_fade_state(FadeState::Solid, 0.5, false, true), FadeState::Solid);
        assert_eq!(
            next_fade_state(FadeState::Solid, 0.016, true, true),
            FadeState::Warning(FADE_WARNING_TIME)
        );
    }

    #[test]
    fn warning_counts_down_then_platform_vanishes() {
        assert_eq!(next_fade_state(FadeState::Warning(1.0), 0.25, true, true), FadeState::Warning(0.75));
        assert_eq!(
            next_fade_state(FadeState::Warning(0.1), 0.25, false, true),
            FadeState::Gone(FADE_GONE_TIME)
        );
    }

    #[test]
    fn platform_comes_back_after_the_delay() {
        assert_eq!(next_fade_state(FadeState::Gone(1.0), 0.5, false, true), FadeState::Gone(0.5));
        assert_eq!(next_fade_state(FadeState::Gone(0.2), 0.5, false, true), FadeState::Solid);
    }

    #[test]
    fn platform_waits_while_the_player_is_where_it_comes_back() {
        // Time is over but the player is in the way: stay gone, never trap the player
        assert_eq!(next_fade_state(FadeState::Gone(0.2), 0.5, false, false), FadeState::Gone(0.0));
        assert_eq!(next_fade_state(FadeState::Gone(0.0), 0.016, false, true), FadeState::Solid);
    }
}
