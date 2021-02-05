use crate::{
    cpu::Cpu,
    io::{self, Display, Sound},
    memory::Memory,
};
use ggez::{
    event::{EventHandler, KeyCode, KeyMods},
    graphics::{self},
    timer, Context, GameResult,
};
use graphics::BLACK;
use rand::prelude::*;

pub struct Chip8 {
    cpu: Cpu,
    memory: Memory,
    display: Display,
    sound: Sound,
    delay_timer: u8,
    sound_timer: u8,
    rng: ThreadRng,
    key_pressed: Option<u8>,
}

impl Chip8 {
    pub fn new(ctx: &mut Context) -> GameResult<Chip8> {
        let chip8 = Chip8 {
            cpu: Cpu::new(),
            memory: Memory::new(),
            display: Display::new(),
            sound: Sound::new(ctx)?,
            delay_timer: 0,
            sound_timer: 0,
            rng: thread_rng(),
            key_pressed: None,
        };
        Ok(chip8)
    }

    pub fn load_prog(&mut self, data: &[u8]) {
        self.memory.load_prog(self.cpu.register.pc.into(), data);
    }

    fn fetch_opcode(&mut self) -> u16 {
        let upper = self.memory[self.cpu.register.pc] as u16;
        let res = (upper << 8) | self.memory[self.cpu.register.pc + 1] as u16;
        self.cpu.register.pc = self.cpu.register.pc.overflowing_add(2).0; //as long as fenct

        res
    }

    fn decode(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;
        let n = opcode & 0x000F;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let vx = self.cpu.register.v[x];
        let vy = self.cpu.register.v[y];
        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    //disp_clear()
                    0x00E0 => {
                        self.display.clear_window();
                    }
                    //return
                    0x00EE => {
                        self.cpu.register.sp -= 1;
                        self.cpu.register.pc = self.cpu.stack[self.cpu.register.sp as usize];
                    }
                    //call
                    _ => {
                        self.cpu.stack[self.cpu.register.sp as usize] = self.cpu.register.pc;
                        self.cpu.register.sp += 1;
                        self.cpu.register.pc = nnn;
                    }
                }
            }
            //1NNN jmp to NNN
            0x1000 => {
                self.cpu.register.pc = nnn;
            }
            //2NNN Calls subroutine at NNN
            0x2000 => {
                self.cpu.stack[self.cpu.register.sp as usize] = self.cpu.register.pc;
                self.cpu.register.sp += 1;
                self.cpu.register.pc = nnn;
            }
            //3XNN if(vx == NN) skip next instruction
            0x3000 => {
                if self.cpu.register.v[x] == nn {
                    self.cpu.register.pc += 2;
                }
            }
            //4XNN if(VX != NN) skip next instruction
            0x4000 => {
                if self.cpu.register.v[x] != nn {
                    self.cpu.register.pc += 2;
                }
            }

            //5XY0 if (VX == VY) skip next instruction
            0x5000 => {
                if n != 0 {
                    todo!("invalid expre")
                }
                let vx = self.cpu.register.v[x];
                let vy = self.cpu.register.v[y];
                if vx == vy {
                    self.cpu.register.pc += 2;
                }
            }
            //6XNN set Vx to NN
            0x6000 => {
                self.cpu.register.v[x] = nn;
            }
            //7XNN add NN to VX
            0x7000 => {
                self.cpu.register.v[x] = self.cpu.register.v[x].overflowing_add(nn).0;
            }
            //0x8XYN reg cal
            0x8000 => {
                match opcode & 0x000F {
                    //VX=VY
                    0 => {
                        self.cpu.register.v[x] = vy;
                    }
                    //VX or VY
                    1 => {
                        self.cpu.register.v[x] = vx | vy;
                    }
                    //VX = VX & VY
                    2 => {
                        self.cpu.register.v[x] = vx & vy;
                    }
                    //VX = VX ^ VY
                    3 => {
                        self.cpu.register.v[x] = vx ^ vy;
                    }
                    //VX = VX + VY with set carry bit
                    4 => {
                        let (v, of) = vy.overflowing_add(vx);
                        self.cpu.register.v[x] = v;
                        if of {
                            self.cpu.register.v[0xF] = 1;
                        } else {
                            self.cpu.register.v[0xF] = 0;
                        }
                    }
                    //vx = vx- vy
                    5 => {
                        let (v, of) = (vx).overflowing_sub(vy);
                        self.cpu.register.v[x] = v;
                        if of {
                            self.cpu.register.v[0xF] = 0;
                        } else {
                            self.cpu.register.v[0xF] = 1;
                        }
                    }
                    //vx = vx >> 1
                    6 => {
                        let v = vx & 0x01;
                        self.cpu.register.v[x] = vx >> 1;
                        self.cpu.register.v[0xF] = v;
                    }
                    //vx = vy-vx
                    7 => {
                        let (v, of) = vy.overflowing_sub(vx);
                        self.cpu.register.v[x] = v;
                        if of {
                            self.cpu.register.v[0xF] = 0;
                        } else {
                            self.cpu.register.v[0xF] = 1;
                        }
                    }
                    //vx = vx << 1
                    0xE => {
                        let v = (vx & 0x80) >> 7;
                        self.cpu.register.v[x] = vx << 1;
                        self.cpu.register.v[0xF] = v;
                    }
                    _ => {
                        panic!(
                            "Invalid Expression: {} as address {}",
                            opcode,
                            self.cpu.register.pc - 2
                        )
                    }
                }
            }
            //9XY0 if Vx != Vy jmp next instruction
            0x9000 => {
                if opcode & 0x000F != 0 {
                    panic!(
                        "Invalid Expression: {} at address {}",
                        opcode,
                        self.cpu.register.pc - 2
                    )
                }
                if vx != vy {
                    self.cpu.register.pc += 2;
                }
            }
            //ANNN I = NNN
            0xA000 => {
                self.cpu.register.I = nnn;
            }
            //BNNN pc += V0 + NNN
            0xB000 => {
                self.cpu.register.pc += self.cpu.register.v[0] as u16 + nnn;
            }
            //CXNN Vx = rand() & NN
            0xC000 => {
                let rand: u8 = self.rng.gen_range(0, 255);
                self.cpu.register.v[x] = (rand & nn) as u8;
            }
            //DXYN draw(Vx, Vy, N) WARNING: wiki is Wrong!,hight is not n+1
            0xD000 => {
                let I = self.cpu.register.I;
                let mut data = Vec::new();
                for i in 0..n {
                    data.push(self.memory[I + i]);
                }
                if self.display.draw_sprite(vx, vy, &data) {
                    self.cpu.register.v[0xF] = 1;
                } else {
                    self.cpu.register.v[0xF] = 0;
                }
            }
            //if key()==/!= Vx jmp next instruction
            0xE000 => match opcode & 0x00FF {
                0x9E => {
                    if let Some(key) = self.key_pressed {
                        if key == vx {
                            self.cpu.register.pc += 2;
                        }
                    }
                    self.key_pressed = None;
                }
                0xA1 => {
                    if let Some(key) = self.key_pressed {
                        if key != vx {
                            self.cpu.register.pc += 2;
                        }
                    }
                    self.key_pressed = None;
                }
                _ => {
                    panic!(
                        "Invalid Expression: {:#x} at address {}",
                        opcode,
                        self.cpu.register.pc - 2
                    )
                }
            },

            0xF000 => {
                match opcode & 0x00FF {
                    //LD VX, DT
                    0x07 => {
                        self.cpu.register.v[x] = self.delay_timer;
                    }
                    //LD VX, KEY
                    0x0A => {
                        while self.key_pressed.is_none() {
                            timer::yield_now();
                        }
                    }
                    //LD DT, VX
                    0x15 => {
                        self.delay_timer = vx;
                    }
                    //LD ST, VX
                    0x18 => {
                        self.sound_timer = vx;
                    }
                    //ADD I,VX
                    0x1E => {
                        let (v, _of) = self
                            .cpu
                            .register
                            .I
                            .overflowing_add(self.cpu.register.v[x].into());
                        self.cpu.register.I += v;
                        if self.cpu.register.I > 0x0FFF {
                            self.cpu.register.v[0xF] = 1;
                        } else {
                            self.cpu.register.v[0xF] = 0;
                        }
                    }
                    //LD I, FONT(VX)
                    0x29 => {
                        self.cpu.register.I = (vx * 5).into();
                    }
                    //BCD VX
                    0x33 => {
                        let mut t = vx;
                        for i in 0..3 {
                            let rem = t % 10;
                            t = t / 10;
                            self.memory[self.cpu.register.I + 3 - i] = rem;
                        }
                    }
                    //LD [I] VX
                    0x55 => {
                        for i in 0..=x {
                            self.memory[self.cpu.register.I + i as u16] = self.cpu.register.v[i]
                        }
                    }
                    //LD VX [I]
                    0x65 => {
                        for i in 0..=x {
                            self.cpu.register.v[i] = self.memory[self.cpu.register.I + i as u16];
                        }
                    }
                    _ => {
                        panic!(
                            "Invalid Expression: {:#x} at address {}",
                            opcode,
                            self.cpu.register.pc - 2
                        )
                    }
                }
            }

            _ => {
                panic!(
                    "Invalid Expression: {} at address {}",
                    opcode,
                    self.cpu.register.pc - 2
                )
            }
        }
    }
}

impl EventHandler for Chip8 {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 80;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.sound.play()?;
                self.sound_timer -= 1;
            }
            let opcode = self.fetch_opcode();
            //println!("{:#x} {}", opcode, self.cpu.register.pc);
            //println!("{:?}", self.key_pressed);
            self.decode(opcode);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, BLACK);
        self.display.draw(ctx)?;
        graphics::present(ctx)?;

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        self.key_pressed = io::keyboard_match(keycode);
    }
}

#[cfg(test)]
mod tests {}
