use voxel_game::Game;

fn main() {
    tracing_subscriber::fmt::init();
    Game::run();
}