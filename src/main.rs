mod chip8;
mod cpu;
mod io;
mod memory;

use std::{
    env,
    fs::File,
    io::{BufReader, Read},
    path,
};

use anyhow::Result;
use chip8::Chip8;
use ggez::{conf, event, ContextBuilder};
use io::SCREEN_SIZE;

const DEFAULT_GAME: &'static str = "./games/br8kout.ch8";
fn main() -> Result<()> {
    let mut game_path = DEFAULT_GAME.to_string();
    let mut args = env::args();
    if args.len() > 1 {
        game_path = args.nth(1).unwrap();
    }

    let mut game_bin = BufReader::new(File::open(game_path)?);
    let mut game = Vec::new();
    game_bin.read_to_end(&mut game)?;
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };
    let cb = ContextBuilder::new("chip8", "Opadc")
        .window_setup(conf::WindowSetup::default().title("CHIP8!"))
        .window_mode(conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .add_resource_path(resource_dir);
    let (mut ctx, mut events_loop) = cb.build()?;
    let mut chip8 = Chip8::new(&mut ctx)?;

    chip8.load_prog(&game);
    for i in game {
        print!("{:#x} ", i);
    }
    event::run(&mut ctx, &mut events_loop, &mut chip8)?;
    Ok(())
}
