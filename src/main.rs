#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate image;
extern crate rayon;

mod engine;
mod game;

use crate::game::game::Game;

fn main() {
    let mut game = Game::new("Voxel Game");
    game.run();
}
