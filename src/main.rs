#![allow(warnings)]
#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate image;
extern crate rayon;
#[macro_use]
extern crate conrod;

mod engine;
mod game;
mod utils;

use crate::game::Game;

fn main() {
    let mut game = Game::new("Cave game v0.1.0");
    game.run();
}
