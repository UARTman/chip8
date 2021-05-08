use log::{debug, error, trace, warn};
use rand::Rng;

impl super::Cpu {
    pub fn fetch_next_instruction(&mut self) -> [u8; 2] {
        let instruction = &self.ram[self.pc as usize..=self.pc as usize + 2];
        trace!(
            "Instruction 0x{:02X}{:02X} at {}",
            instruction[0],
            instruction[1],
            self.pc
        );
        self.pc += 2;
        [instruction[0], instruction[1]]
    }

    fn parse_instruction(instruction: [u8; 2]) -> (u8, u8, u8, u16) {
        let x = instruction[0] & 0x0F;
        let y = instruction[1] >> 4;
        let kk = instruction[1];
        let nnn = kk as u16 + ((x as u16) << 8);
        (x, y, kk, nnn)
    }

    /// Execute a given instruction.
    pub fn execute_instruction(&mut self, instruction: [u8; 2]) {
        let (x, y, byte, addr) = Self::parse_instruction(instruction);
        let last_digit = byte & 0x0F;
        match instruction[0] >> 4 {
            0 => match instruction {
                [0x00, 0xE0] => {
                    debug!("CLS");
                    for i in self.vram.iter_mut() {
                        for j in i.iter_mut() {
                            *j = false;
                        }
                    }
                }
                [0x00, 0xEE] => {
                    debug!("RET");
                    if self.sp == 0 {
                        error!("Trying to return with empty stack! Execution halted.");
                        self.pause();
                        // panic!("{:?}", self.execution_state)
                    } else {
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                        trace!("Returning to {:X}", self.pc);
                    }
                }
                _ => {
                    debug!("SYS {:X}", addr);
                    warn!("SYS is an unsupported instruction! Continuing execution.")
                }
            },
            1 => {
                debug!("JP {:X}", addr);
                self.pc = addr;
                trace!("Jumping to {:X}", self.pc);
            }
            2 => {
                debug!("CALL {:X}", addr);
                if self.sp == 16 {
                    error!("Stack overflow at 0x{:X}", self.pc-2);
                    self.execution_state = super::ExecutionState::Paused;
                    return
                }
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = addr;
            }
            3 => {
                debug!("SE V{:X}, {:X}", x, byte);
                if self.reg[x as usize] == byte {
                    self.pc += 2;
                }
            }
            4 => {
                debug!("SNE V{:X}, {:X}", x, byte);
                if self.reg[x as usize] != byte {
                    self.pc += 2;
                }
            }
            5 => {
                debug!("SE V{:X}, V{:X}", x, y);
                if self.reg[x as usize] == self.reg[y as usize] {
                    self.pc += 2;
                }
            }
            6 => {
                debug!("LD V{:X}, {:X}", x, byte);
                self.reg[x as usize] = byte;
            }
            7 => {
                debug!("ADD V{:X}, {:X}", x, byte);
                self.reg[x as usize] = self.reg[x as usize].wrapping_add(byte);
            }
            8 => match last_digit {
                0 => {
                    debug!("LD V{:X}, V{:X}", x, y);
                    self.reg[x as usize] = self.reg[y as usize];
                }
                1 => {
                    debug!("OR V{:X}, V{:X}", x, y);
                    self.reg[x as usize] |= self.reg[y as usize];
                }
                2 => {
                    debug!("AND V{:X}, V{:X}", x, y);
                    self.reg[x as usize] &= self.reg[y as usize];
                }
                3 => {
                    debug!("XOR V{:X}, V{:X}", x, y);
                    self.reg[x as usize] ^= self.reg[y as usize];
                }
                4 => {
                    debug!("ADD V{:X}, V{:X}", x, y);
                    let results = self.reg[x as usize].overflowing_add(self.reg[y as usize]);
                    self.reg[x as usize] = results.0;
                    self.reg[15] = results.1 as u8;
                }
                5 => {
                    debug!("SUB V{:X}, V{:X}", x, y);
                    let (result, rev_carry) =
                        self.reg[x as usize].overflowing_sub(self.reg[y as usize]);
                    self.reg[x as usize] = result;
                    self.reg[15] = !rev_carry as u8;
                }
                6 => {
                    debug!("SHR V{:X}", y);
                    let carry = self.reg[x as usize] & 0b00000001;
                    self.reg[x as usize] >>= 1;
                    self.reg[15] = carry;
                }
                7 => {
                    debug!("SUBN V{:X}, V{:X}", x, y);
                    let (result, rev_carry) =
                        self.reg[y as usize].overflowing_sub(self.reg[y as usize]);
                    self.reg[x as usize] = result;
                    self.reg[15] = !rev_carry as u8;
                }
                0xE => {
                    debug!("SHL V{:X}", x);
                    let (result, carry) =
                        self.reg[x as usize].overflowing_shl(self.reg[y as usize] as u32);
                    self.reg[x as usize] = result;
                    self.reg[15] = carry as u8;
                }
                _ => debug!("0x8__{:X}", last_digit),
            },
            9 => {
                debug!("SNE V{:X}, V{:X}", x, y);
                if self.reg[x as usize] != self.reg[y as usize] {
                    self.pc += 2;
                }
            }
            0xA => {
                debug!("LD I, {:X}", addr);
                self.reg_i = addr;
            }
            0xB => {
                debug!("JP V0, {:X}", addr);
                self.pc = self.reg[0] as u16 + addr;
            }
            0xC => {
                debug!("RND V{:X}, {:X}", x, byte);
                self.reg[x as usize] = rand::thread_rng().gen::<u8>()  & byte;
                // warn!("RND unimplemented!");
            }
            0xD => {
                debug!("DRW V{:X}, V{:X}, {:X}", x, y, last_digit);
                let (vx, vy) = (self.reg[x as usize], self.reg[y as usize]);
                let mut sprite = Vec::new();
                for i in 0..last_digit as usize {
                    sprite.push(self.ram[self.reg_i as usize + i]);
                }
                let result = self.draw_sprite(
                    vx as usize,
                    vy as usize,
                    &sprite,
                );
                self.reg[15] = result as u8;
            }
            0xE => match byte {
                0x9E => {
                    debug!("SKP V{:X}", x);
                    if self.keyboard[self.reg[x as usize] as usize] {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    debug!("SKNP V{:X}", x);
                    if !self.keyboard[self.reg[x as usize] as usize] {
                        self.pc += 2;
                    }
                }
                _ => debug!("0xE_{:X}", byte),
            },
            0xF => match byte {
                0x07 => {
                    debug!("LD V{:X}, DT", x);
                    self.reg[x as usize] = self.tim_delay;
                }
                0x0A => {
                    debug!("LD V{:X}, K", x);
                    self.execution_state =
                        super::ExecutionState::WaitingForKey(self.reg[x as usize] as usize);
                }
                0x15 => {
                    debug!("LD DT, V{:X}", x);
                    self.tim_delay = self.reg[x as usize];
                }
                0x18 => {
                    debug!("LD ST, V{:X}", x);
                    self.tim_sound = self.reg[x as usize];
                }
                0x1E => {
                    debug!("ADD I, V{:X}", x);
                    self.reg_i += self.reg[x as usize] as u16;
                }
                0x29 => {
                    debug!("LD F, V{:X}", x);
                    self.reg_i = (self.reg[x as usize] * 5) as u16;
                }
                0x33 => {
                    debug!("LD B, V{:X}", x);
                    self.ram[self.reg_i as usize] = self.reg[x as usize] / 100;
                    self.ram[self.reg_i as usize + 1] = (self.reg[x as usize] / 10) % 10;
                    self.ram[self.reg_i as usize + 2] = self.reg[x as usize] % 10;
                    // warn!("BCD isn't implemented!");
                }
                0x55 => {
                    debug!("LD [I], V{:X}", x);
                    for i in 0..=x as usize {
                        self.ram[self.reg_i as usize + i] = self.reg[i];
                    }
                }
                0x65 => {
                    debug!("LD V{:X}, [I]", x);
                    for i in 0..=x as usize {
                        self.reg[i] = self.ram[self.reg_i as usize + i];
                    }
                }
                _ => debug!("0xF_{:X}", byte),
            },
            _ => debug!("0x{:X}{:X}", instruction[0], instruction[1]),
        }
    }
}

#[cfg(test)]
mod test {
    use simple_logger::SimpleLogger;

    use crate::cpu::{Cpu, ExecutionState};

    #[test]
    fn test_cls() {
        let _ = SimpleLogger::new().init();
        let mut cpu = Cpu::new();
        for i in cpu.vram.iter_mut() {
            for j in i.iter_mut() {
                *j = true;
            }
        }
        cpu.execute_instruction([0x00, 0xE0]);
        let or_result = cpu
            .vram
            .iter()
            .map(|x| x.iter().fold(false, |x, &y| x | y))
            .fold(false, |x, y| x | y);
        assert_eq!(or_result, false, "Screen should be black");
    }

    #[test]
    fn test_ret() {
        let _ = SimpleLogger::new().init();
        let mut cpu = Cpu::new();
        cpu.stack[0] = 2048;
        cpu.sp = 1;
        cpu.execute_instruction([0x00, 0xEE]);
        assert_eq!(cpu.sp, 0, "Stack pointer should decrease by 1");
        assert_eq!(
            cpu.pc, 2048,
            "Program counter should point at the return address"
        );
    }

    #[test]
    fn test_ret_graceful() {
        let _ = SimpleLogger::new().init();
        let mut cpu = Cpu::new();
        cpu.execute_instruction([0x00, 0xEE]);
        assert_eq!(
            cpu.execution_state,
            ExecutionState::Paused,
            "This incorrect command should stop the execution."
        )
    }

    #[test]
    fn test_jump() {
        let _ = SimpleLogger::new().init();
        let mut cpu = Cpu::new();
        cpu.execute_instruction([0x12, 0x34]);
        assert_eq!(
            cpu.pc, 0x234,
            "Program counter should point to the target address"
        );
    }

    #[test]
    fn test_call() {
        let _ = SimpleLogger::new().init();
        let mut cpu = Cpu::new();
        cpu.execute_instruction([0x23, 0x45]);
        assert_eq!(cpu.sp, 1, "Stack pointer should go up by 1");
        assert_eq!(
            cpu.stack[0], 0x200,
            "Return address should be pushed onto a stack"
        );
        assert_eq!(
            cpu.pc, 0x345,
            "Program counter should be pointing to the target address"
        );
    }

    #[test]
    fn test_se() {
        let _ = SimpleLogger::new().init();
        let mut cpu = Cpu::new();
        cpu.reg[4] = 0x56;
        cpu.execute_instruction([0x34, 0x56]);
        assert_eq!(
            cpu.pc, 0x202,
            "Program counter should skip the next instruction if V4 == 0x56"
        );
        let mut cpu = Cpu::new();
        cpu.execute_instruction([0x34, 0x56]);
        assert_eq!(
            cpu.pc, 0x200,
            "Program counter should not skip the next instruction if V4 != 0x56"
        );
    }

    #[test]
    fn test_sne() {
        let _ = SimpleLogger::new().init();
        let mut cpu = Cpu::new();
        cpu.execute_instruction([0x41, 0x23]);
        assert_eq!(
            cpu.pc, 0x202,
            "Program counter should skip the next instruction if V1 != 0x23"
        );
        let mut cpu = Cpu::new();
        cpu.reg[1] = 0x23;
        cpu.execute_instruction([0x41, 0x23]);
        assert_eq!(
            cpu.pc, 0x200,
            "Program counter should not skip the next instruction if V1 == 0x23"
        );
    }

    #[test]
    fn test_se_reg() {
        let _ = SimpleLogger::new().init();
        let mut cpu = Cpu::new();
        cpu.execute_instruction([0x50, 0x10]);
        assert_eq!(
            cpu.pc, 0x202,
            "Program counter should skip the next instrution is V0 == V1"
        );
        let mut cpu = Cpu::new();
        cpu.reg[0] = 1;
        cpu.execute_instruction([0x50, 0x10]);
        assert_eq!(
            cpu.pc, 0x200,
            "Program counter should not skip the next instrution is V0 != V1"
        );
    }

    #[test]
    fn test_ld() {
        let _ = SimpleLogger::new().init();
        let mut cpu = Cpu::new();
        cpu.execute_instruction([0x60, 0x12]);
        assert_eq!(cpu.reg[0], 0x12, "0x12 should be loaded into V0");
    }

    #[test]
    fn test_add_byte() {
        let _ = SimpleLogger::new().init();
        let mut cpu = Cpu::new();
        cpu.reg[0] = 1;
        cpu.execute_instruction([0x70, 0x02]);
        assert_eq!(cpu.reg[0], 3, "V0 should 1+2=3");
        cpu.reg[0] = 255;
        cpu.execute_instruction([0x70, 0x01]);
        assert_eq!(cpu.reg[0], 0, "Addition should be overflowing")
    }

    #[test]
    fn test_parse() {
        let result = Cpu::parse_instruction([0x12, 0x34]);
        assert_eq!(result, (0x2, 0x3, 0x34, 0x234))
    }

    #[test]
    #[ignore = "not yet needed"]
    fn test_nopanic() {
        let _ = SimpleLogger::new().init();
        let mut cpu = Cpu::new();
        for i in 0..=255u8 {
            for j in 0..=255u8 {
               cpu.execute_instruction([i, j]); 
            }
        }
    }
    
}
