use std::fmt;

use audio::SoundSource;
use ggez::{
    audio,
    event::KeyCode,
    graphics::{self, DrawMode, FillOptions, BLACK},
    Context,
};
use ggez::{
    graphics::{DrawParam, Drawable},
    GameResult,
};
use graphics::{Mesh, Rect, WHITE};

pub(crate) const GRID_SIZE: (usize, usize) = (64, 32);
const GRID_CELL_SIZE: (usize, usize) = (10, 10);
const GRID_NUMS: usize = GRID_SIZE.0 * GRID_SIZE.1;

pub(crate) const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32,
);

pub struct Display {
    gfx: [u8; GRID_NUMS],
}

impl Display {
    pub fn new() -> Self {
        Display {
            gfx: [0; GRID_NUMS],
        }
    }

    pub fn clear_window(&mut self) {
        for b in &mut self.gfx {
            *b = 0;
        }
    }

    //if there is collision ,return true else ruturn false
    pub fn draw_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool {
        let mut res = false;
        let x: usize = (x % 64).into();
        let y: usize = (y % 32).into();
        let mut pos = y * GRID_SIZE.0 + x;
        for i in 0..sprite.len() {
            let curr_byte = sprite[i];
            for t in 0..8 {
                let b = curr_byte >> (7 - t) & 0b0000_0001;
                if b == 1 && self.gfx[pos + t] == 1 {
                    res = true;
                }
                self.gfx[pos + t] ^= b;
            }
            pos += GRID_SIZE.0; //next row
        }

        res
    }
    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        for y in 0..GRID_SIZE.1 {
            for x in 0..GRID_SIZE.0 {
                let p = y * GRID_SIZE.0 + x;
                let color = if self.gfx[p] == 1 { WHITE } else { BLACK };
                let bounds = Rect {
                    x: (x * GRID_CELL_SIZE.0) as f32,
                    y: (y * GRID_CELL_SIZE.1) as f32,
                    w: GRID_CELL_SIZE.0 as f32,
                    h: GRID_CELL_SIZE.1 as f32,
                };
                let mesh = Mesh::new_rectangle(
                    ctx,
                    DrawMode::Fill(FillOptions::default()),
                    bounds,
                    color,
                )?;
                mesh.draw(ctx, DrawParam::default())?;
            }
        }

        Ok(())
    }
}

pub(crate) fn keyboard_match(keycode: KeyCode) -> Option<u8> {
    match keycode {
        KeyCode::Key1 => Some(0x1),
        KeyCode::Key2 => Some(0x2),
        KeyCode::Key3 => Some(0x3),
        KeyCode::Key4 => Some(0xC),
        KeyCode::Q => Some(0x4),
        KeyCode::W => Some(0x5),
        KeyCode::E => Some(0x6),
        KeyCode::R => Some(0xD),
        KeyCode::A => Some(0x7),
        KeyCode::S => Some(0x8),
        KeyCode::D => Some(0x9),
        KeyCode::F => Some(0xE),
        KeyCode::Z => Some(0xA),
        KeyCode::X => Some(0x0),
        KeyCode::C => Some(0xB),
        KeyCode::V => Some(0xF),
        _ => None,
    }
}

pub struct Sound {
    sound: audio::Source,
}

impl Sound {
    pub fn new(ctx: &mut Context) -> GameResult<Sound> {
        let sound = audio::Source::new(ctx, "/pew.ogg")?;
        let sd = Sound { sound };
        Ok(sd)
    }
    pub fn play(&mut self) -> GameResult<()> {
        self.sound.play()
    }
}

impl fmt::Debug for Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..GRID_SIZE.1 {
            for i in 0..GRID_SIZE.0 {
                write!(f, "{} ", self.gfx[row * GRID_SIZE.1 + i])?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}
