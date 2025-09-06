use macroquad::prelude::*;

pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 600.0;

// Game Constants
pub const GRAVITY: f32 = 500.0;
pub const PLAYER_SPEED: f32 = 150.0;
pub const JUMP_FORCE: f32 = -350.0;
pub const COYOTE_TIME: f32 = 0.1;       // Seconds during which a jump is still allowed after leaving a platform
pub const JUMP_BUFFER_TIME: f32 = 0.1;  // Seconds during which an early jump press is remembered
pub const PLATFORM_SPEED: f32 = 50.0;
pub const SHADOW_FRAMES_DELAY: usize = 75;
pub const SHADOW_TRAIL_LEN: usize = 12;       // Past shadow positions kept for the trail
pub const SHADOW_TRAIL_SPACING: usize = 4;    // Frames between two ghost copies
pub const SHADOW_ALERT_DISTANCE: f32 = 150.0; // Below this distance (px) the shadow shows a warning ring
pub const SHADOW_DELAY_STEP: usize = 5;        // Frames removed from the shadow delay at each step
pub const SHADOW_DELAY_MIN: usize = 30;        // The shadow never gets closer than this
pub const SHADOW_DELAY_INTERVAL: f32 = 30.0;   // Seconds between two steps

// Size Constants
pub const PLAYER_SIZE: Vec2 = vec2(12.0 *4., 12.0 *4.);
pub const GROUND_SIZE: Vec2 = vec2(800.0, 12.0);
pub const PLATFORM_SIZE: Vec2 = vec2(200.0, 12.0);

// Colors
pub const PLAYER_COLOR: Color = Color::new(0.45, 0.26, 0.20, 1.0);  // Rustic brown for player
pub const PLATFORM_COLOR: Color = Color::new(0.76, 0.60, 0.42, 1.0);  // Sandy beige for moving platforms
pub const STATIC_PLATFORM_COLOR: Color = Color::new(0.87, 0.68, 0.45, 1.0);  // Light sand for static platforms
pub const SHADOW_COLOR: Color = Color::new(0.2, 0.1, 0.05, 0.6);  // Dark sepia shadow
pub const BACKGROUND_COLOR: Color = Color::new(0.98, 0.90, 0.75, 1.0);  // Bright, warm sunshine yellow

// Text colors for different purposes
pub const TEXT_PRIMARY: Color = Color::new(0.45, 0.26, 0.20, 1.0);    // Deep brown - for main text
pub const TEXT_SECONDARY: Color = Color::new(0.65, 0.35, 0.25, 1.0);  // Lighter brown - for less important info
pub const TEXT_ACCENT: Color = Color::new(0.8, 0.4, 0.2, 1.0);        // Terracotta - for highlights/scores
pub const TEXT_WARNING: Color = Color::new(0.7, 0.3, 0.2, 1.0);       // Reddish brown - for warnings/game over
pub const TEXT_GOLD: Color = Color::new(0.85, 0.6, 0.2, 1.0);         // Desert gold - for high scores

// Lives system
pub const INITIAL_LIVES: i32 = 3;
pub const INVULNERABILITY_DURATION: f32 = 3.0; // Seconds of invulnerability after getting hit
pub const FLASH_FREQUENCY: f32 = 10.0; // Higher number = faster flashing

// Coins and pickups
pub const COIN_SIZE: Vec2 = vec2(12.0 * 3., 12.0 * 3.);  // Scale the 12x12 sprite by 3
pub const COIN_SPAWN_INTERVAL: f32 = 3.0;  // Spawn a new coin every 3 seconds
pub const COIN_LIFETIME: f32 = 5.0;  // Coins disappear after 5 seconds
pub const COIN_POINTS: i32 = 10;     // Points earned per coin
pub const COIN_SPECIAL_POINTS: i32 = 50;      // Points earned per special coin
pub const COIN_SPECIAL_LIFETIME: f32 = 2.5;  // Special coins disappear faster
pub const SPECIAL_SPAWN_CHANCE: i32 = 10;    // One coin out of N is a special coin
pub const SPECIAL_COIN_COLOR: Color = Color::new(1.0, 0.55, 0.1, 1.0);  // Tint of the special coin
pub const EXTRA_LIFE_SPAWN_CHANCE: i32 = 30;  // One coin out of N is an extra life (only if a life is missing)
pub const EXTRA_LIFE_COLOR: Color = Color::new(0.9, 0.2, 0.25, 1.0);  // Tint of the extra life pickup
pub const FREEZE_DURATION: f32 = 2.0;        // Seconds the shadow stays frozen after a freeze pickup
pub const FREEZE_SPAWN_CHANCE: i32 = 8;       // One coin out of N is a freeze pickup
pub const FREEZE_COLOR: Color = Color::new(0.5, 0.75, 1.0, 1.0);  // Tint of the freeze pickup
pub const HIT_FLASH_DURATION: f32 = 0.25;  // Seconds the screen flashes red after losing a life
pub const FLOATING_TEXT_LIFETIME: f32 = 0.7;  // Seconds a floating text stays visible
pub const FLOATING_TEXT_SPEED: f32 = 30.0;    // Pixels per second, upwards
