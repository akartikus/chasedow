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

// Game State
struct GameState {
    world: World,
    player: Player,
    shadow: Shadow,
    platforms: Vec<Platform>,
    score: f32,
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
        }
    }

    fn update(&mut self) {
        // Update platforms
        for platform in self.platforms.iter_mut() {
            platform.update(&mut self.world);
        }

        // Update player
        self.player.update(&mut self.world);

        // Update shadow
        let player_pos = self.world.actor_pos(self.player.collider);
        self.shadow.update(player_pos);

        // Check shadow collision
        self.shadow.collides_with_player(player_pos);

        // Update score
        self.score += get_frame_time();
    }

    fn draw(&self) {
        clear_background(BACKGROUND_COLOR);

        // Draw platforms
        for platform in &self.platforms {
            platform.draw(&self.world);
        }

        // Draw shadow and player
        self.shadow.draw();
        self.player.draw(&self.world);

        // Draw UI
        self.draw_ui();
    }

    fn draw_ui(&self) {
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 20.0, 20.0, WHITE);
        draw_text(&format!("Score: {:.0}", self.score), 10.0, 50.0, 20.0, WHITE);
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