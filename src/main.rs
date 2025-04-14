use macroquad::prelude::*;

mod audio;
mod coin;
mod constants;
mod game;
mod platform;
mod player;
mod shadow;

use game::GameState;

#[macroquad::main("Chasedow")]
async fn main() {
    let mut game = GameState::new().await;

    loop {
        game.update().await;
        game.draw();
        next_frame().await
    }
}