#![allow(warnings)]
// #![windows_subsystem = "windows"]
#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate image;
extern crate rayon;

mod engine;
mod game;
mod utils;

use crate::game::game::Game;

fn main() {
    let mut game = Game::new("Cave game v0.1.0");
    game.run();
}
