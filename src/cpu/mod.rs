mod execution;
mod graphics;
mod ui;

#[rustfmt::skip]
static SPRITES: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80,
];

#[derive(Debug, PartialEq, Eq)]
pub enum ExecutionState {
    Running,
    Paused,
    WaitingForKey(usize),
}

#[derive(Debug)]
pub enum CpuEvent {
    KeyPressed(usize),
    KeyReleased(usize),
}

#[derive(Debug)]
pub struct Cpu {
    ram: [u8; 4096],
    reg: [u8; 16],
    reg_i: u16,
    tim_delay: u8,
    tim_sound: u8,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    keyboard: [bool; 16],
    pub vram: [[bool; 32]; 64],
    pub execution_state: ExecutionState,
    _cycle: usize,
    rng: Box<rand::rngs::ThreadRng>,
}

impl Cpu {
    pub fn new() -> Self {
        let mut ram = [0;4096];
        for (i, &byte) in SPRITES.iter().enumerate() {
            ram[i] = byte;
        }
        Self {
            ram,
            reg: [0; 16],
            reg_i: 0,
            tim_delay: 0,
            tim_sound: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            keyboard: [false; 16],
            vram: [[false; 32]; 64],
            execution_state: ExecutionState::Paused,
            _cycle: 0,
            rng: Box::new(rand::thread_rng())
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        for (i, &byte) in rom.iter().enumerate() {
            self.ram[0x200+i] = byte
        }
        self.reg = [0; 16];
        self.reg_i = 0;
        self.pc = 0x200;
        self.stack = [0; 16];
        self.sp = 0;
        self.execution_state = ExecutionState::Paused;
        self.tim_delay = 0;
        self.tim_sound = 0;
        self.vram = [[false; 32]; 64];
    }

    pub fn pause(&mut self) {
        if self.execution_state == ExecutionState::Running {
            self.execution_state = ExecutionState::Paused
        }
    }

    pub fn resume(&mut self) {
        if self.execution_state == ExecutionState::Paused {
            self.execution_state = ExecutionState::Running
        }
    }

    pub fn handle_event(&mut self, e: CpuEvent) {}

    pub fn clock_cycle(&mut self) {
        self._cycle += 1;
        self._cycle %= 10;
        // if self._cycle != 0 {
        //     return
        // }

        if self.execution_state == ExecutionState::Running {
            self.tick_counters();
            self.execute_next_instruction();
        }
    }

    fn execute_next_instruction(&mut self) {
        let i = self.fetch_next_instruction();
        self.execute_instruction(i);
    }

    fn tick_counters(&mut self) {
        if self.tim_delay > 0 {
            self.tim_delay -= 1;
        }
        if self.tim_sound > 0 {
            self.tim_sound -= 1;
        }
    }
}
