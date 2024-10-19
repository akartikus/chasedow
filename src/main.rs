use macroquad::prelude::*;
use macroquad_platformer::*;

const GRAVITY: f32 = 500.0;
const PLAYER_SPEED: f32 = 100.0;
const JUMP_FORCE: f32 = -225.0;
const PLATFORM_SPEED: f32 = 50.0;

// Colors for the rectangles
const PLAYER_COLOR: Color = BLUE;
const PLATFORM_COLOR: Color = GREEN;
const STATIC_PLATFORM_COLOR: Color = GRAY;

struct Player {
    collider: Actor,
    speed: Vec2,
    size: Vec2,
}

impl Player {
    fn new(world: &mut World) -> Self {
        let size = vec2(20.0, 20.0);
        Self {
            collider: world.add_actor(vec2(250.0, 80.0), size.x as i32, size.y as i32),
            speed: vec2(0., 0.),
            size,
        }
    }

    fn update(&mut self, world: &mut World) {
        let pos = world.actor_pos(self.collider);
        let on_ground = world.collide_check(self.collider, pos + vec2(0., 1.));

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

        // Apply movement
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
                20., // Player size
                20.,
                Color::new(0.0, 0.0, 0.0, 0.8),
            );
        }
    }

    //todo collisions
    fn collides_with_player(&self, player_pos: Vec2) -> bool {
        const PLAYER_SIZE: Vec2 = vec2(20.0, 20.0);

        if let Some(shadow_pos) = self.positions.first() {
            let shadow_rect = Rect::new(
                shadow_pos.x,
                shadow_pos.y,
                PLAYER_SIZE.x,
                PLAYER_SIZE.y,
            );
            let player_rect = Rect::new(
                player_pos.x,
                player_pos.y,
                PLAYER_SIZE.x,
                PLAYER_SIZE.y,
            );
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

#[macroquad::main("Simple Platformer")]
async fn main() {
    let mut world = World::new();

    // Create game objects
    let mut player = Player::new(&mut world);

    // Shadow
    let shadow = Shadow::new(3);


    // Create various platforms
    let mut platforms = vec![
        // Ground platform
        Platform::new(&mut world, vec2(0.0, 300.0), vec2(800.0, 20.0), false),

        // Static platforms
        Platform::new(&mut world, vec2(50.0, 200.0), vec2(300.0, 20.0), false),
        Platform::new(&mut world, vec2(300.0, 150.0), vec2(200.0, 20.0), false),

        // Moving platform
        Platform::new(&mut world, vec2(170.0, 250.0), vec2(200.0, 20.0), true),
    ];

    // Setup camera
    let camera = Camera2D::from_display_rect(Rect::new(0.0, 300.0, 800.0, -300.0));

    // Game loop
    loop {
        clear_background(LIGHTGRAY);
        set_camera(&camera);

        // Update and draw platforms
        for platform in platforms.iter_mut() {
            platform.update(&mut world);
            platform.draw(&world);
        }


        // Update and draw player
        player.update(&mut world);

        player.draw(&world);
        shadow.draw(player);

        // Draw FPS counter
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 20.0, 20.0, WHITE);

        next_frame().await
    }
}