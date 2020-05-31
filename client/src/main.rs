use client::Game;
use log::info;
use fern::colors::{Color, ColoredLevelConfig};

fn main(){
    dispath_logger();
    info!("Starting game...");
    Game::run();
    info!("Game closed");
}

fn dispath_logger(){
    let colors = ColoredLevelConfig::new()
        .debug(Color::Green)
        .info(Color::Cyan)
        .trace(Color::Magenta);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Off)
        .level_for("client", log::LevelFilter::Trace)
        .level_for("engine", log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()
        .expect("Couldn't create logger");
}
