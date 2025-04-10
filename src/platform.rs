use macroquad::prelude::*;
use macroquad::rand::*;
use macroquad_platformer::*;

use crate::constants::*;

pub struct Platform {
    pub collider: Solid,
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
            speed: if is_moving { PLATFORM_SPEED } else { 0.0 },
            size,
            cactus_texture,
            cacti,
        }
    }

    pub fn update(&mut self, world: &mut World) {
        if self.speed != 0.0 {
            world.solid_move(self.collider, self.speed * get_frame_time(), 0.0);
            let pos = world.solid_pos(self.collider);

            if (self.speed > 1.0 && pos.x >= 500.0) || (self.speed < -1.0 && pos.x <= 150.0) {
                self.speed *= -1.0;
            }
        }
    }

    pub fn draw(&self, world: &World) {
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

pub async fn create_platforms(world: &mut World) -> Vec<Platform> {
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
