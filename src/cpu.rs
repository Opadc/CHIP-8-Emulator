pub struct Register {
    pub v: [u8; 16],
    pub I: u16,
    pub pc: u16,
    pub sp: u16,
}
impl Register {
    pub fn new() -> Self {
        Register {
            v: [0; 16],
            I: 0,
            pc: 0x200,
            sp: 0,
        }
    }
}

pub struct Cpu {
    pub register: Register,
    pub stack: [u16; 16],
    pub delay_timer: u8,
    pub sound_timer: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            register: Register::new(),
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}
