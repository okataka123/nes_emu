fn main() {
    println!("Hello, world!");
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
   Immediate,
   ZeroPage,
   ZeroPage_X,
   ZeroPage_Y,
   Absolute,
   Absolute_X,
   Absolute_Y,
   Indirect_X,
   Indirect_Y,
   NoneAddressing,
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0x10000],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0x00; 0x10000],
        }
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }
            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos+1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos+1, hi);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0;
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn run(&mut self) {
        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;

            match code {
                0xA9 => {
                    self.lda(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xA5 => {
                    self.lda(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0xB5 => {
                    self.lda(&AddressingMode::ZeroPage_X);
                    self.program_counter += 1;
                }
                0xAD => {
                    self.lda(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0xBD => {
                    self.lda(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0xB9 => {
                    self.lda(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                0xA1 => {
                    self.lda(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                0xB1 => {
                    self.lda(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }
                0x85 => {
                    self.sta(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0x95 => {
                    self.sta(&AddressingMode::ZeroPage_X);
                    self.program_counter += 1;
                }
                0x8D => {
                    self.sta(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0x9D => {
                    self.sta(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0x99 => {
                    self.sta(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                0x81 => {
                    self.sta(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                0x91 => {
                    self.sta(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }

                // BRK Implied
                0x00 => return,

                // TAX Implied
                0xAA => self.tax(),

                // INX Implied.
                0xE8 => self.inx(),

                0x69 => {
                    self.adc(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }

                _ => todo!()
            }
        }
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }
    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let carry = self.status & 0x01;

        let (rhs, carry_flag1) = value.overflowing_add(carry);
        let (n, carry_flag2) = self.register_a.overflowing_add(rhs);

        let overflow = ((self.register_a & 0x80) == (value & 0x80)) && ((value & 0x80) != (n & 0x80));
        self.register_a = n;

        self.status = if carry_flag1 || carry_flag2 {
            self.status | 0x01
        } else {
            self.status & !0x01
        };
        self.status = if overflow {
            self.status | 0x40
        } else {
            self.status & !0x40
        };
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }
        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immidiate_load_date() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }
    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }
    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0x00]);
        assert!(cpu.status & 0b1000_0000 != 0);
    }
    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xaa, 0x00]);
        cpu.reset();
        cpu.register_a = 10;
        cpu.run();
        assert_eq!(cpu.register_x, 10);
    }
    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        assert_eq!(cpu.register_x, 0xc1);
    }
    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xe8, 0xe8, 0x00]);
        cpu.reset();
        cpu.register_x = 0xff;
        cpu.run();
        println!("cpu.register_x = {}", cpu.register_x);
        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_lda_from_memory_zero_page() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);
        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);
        assert_eq!(cpu.register_a, 0x55);
    }
    #[test]
    fn test_lda_from_memory_zero_page_X() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xb5, 0x10, 0x00]);
        cpu.reset();
        cpu.mem_write(0x11, 0x56);
        cpu.register_x = 0x01;
        cpu.run();
        assert_eq!(cpu.register_a, 0x56);
    }
    #[test]
    fn test_lda_from_memory_absolute() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xad, 0x12, 0x34, 0x00]);
        cpu.reset();
        cpu.mem_write(0x3412, 0x23);
        cpu.run();
        assert_eq!(cpu.register_a, 0x23);
    }
    #[test]
    fn test_lda_from_memory_absolute_x() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xbd, 0x12, 0x34, 0x00]);
        cpu.reset();
        cpu.register_x = 0x0001;
        cpu.mem_write(0x3413, 0x23);
        cpu.run();
        assert_eq!(cpu.register_a, 0x23);
    }
    #[test]
    fn test_lda_from_memory_absolute_y() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xb9, 0x12, 0x34, 0x00]);
        cpu.reset();
        cpu.register_y = 0x0002;
        cpu.mem_write(0x3414, 0x23);
        cpu.run();
        assert_eq!(cpu.register_a, 0x23);
    }
    #[test]
    fn test_lda_from_memory_indirect_x() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xa1, 0x10, 0x00]);
        cpu.reset();
        cpu.register_x = 0x08;
        cpu.mem_write(0x18, 0x05);
        cpu.mem_write(0x19, 0x06);
        cpu.mem_write(0x0605, 0x44);
        cpu.run();
        assert_eq!(cpu.register_a, 0x44);
    }
    #[test]
    fn test_lda_from_memory_indirect_x_2() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xa1, 0x10, 0x00]);
        cpu.reset();
        cpu.register_x = 0x08;
        cpu.mem_write_u16(0x18, 0x0605);
        cpu.mem_write(0x0605, 0x44);
        cpu.run();
        assert_eq!(cpu.register_a, 0x44);
    }
    #[test]
    fn test_lda_from_memory_indirect_y() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xb1, 0x20, 0x00]);
        cpu.reset();
        cpu.register_y = 0x01;
        cpu.mem_write(0x20, 0xcd);
        cpu.mem_write(0x21, 0xab);
        cpu.mem_write(0xabce, 0x55);
        cpu.run();
        assert_eq!(cpu.register_a, 0x55);
    }
    #[test]
    fn test_lda_from_memory_indirect_y_2() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xb1, 0x20, 0x00]);
        cpu.reset();
        cpu.register_y = 0x02;
        cpu.mem_write_u16(0x20, 0xabcd);
        cpu.mem_write(0xabcf, 0x55);
        cpu.run();
        assert_eq!(cpu.register_a, 0x55);
    }
    #[test]
    fn test_sta_from_memory_zero_page() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x85, 0x10, 0x00]);
        cpu.reset();
        cpu.register_a = 0xAA;
        cpu.run();
        assert_eq!(cpu.mem_read(0x10), 0xAA);
    }
    #[test]
    fn test_adc_no_carry() {
        // carryがない場合
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x10, 0x00]);
        cpu.reset();
        cpu.register_a = 0x20;
        cpu.run();
        assert_eq!(cpu.register_a, 0x30);
        assert_eq!(cpu.status, 0x00);
    }
    #[test]
    fn test_adc_has_carry() {
        // carryがある場合
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x10, 0x00]);
        cpu.reset();
        cpu.register_a = 0x20;
        cpu.status = 0x01;
        cpu.run();
        assert_eq!(cpu.register_a, 0x31);
        assert_eq!(cpu.status, 0x00);
    }
    #[test]
    fn test_adc_occur_carry() {
        // carryが発生する場合
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x01, 0x00]);
        cpu.reset();
        cpu.register_a = 0xFF;
        cpu.run();
        assert_eq!(cpu.register_a, 0x00);
        assert_eq!(cpu.status, 0x03);
    }
    #[test]
    fn test_adc_occur_overflow_plus() {
        // plusのときにoverflowが発生する場合
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x01, 0x00]);
        cpu.reset();
        cpu.register_a = 0x7F;
        cpu.run();
        assert_eq!(cpu.register_a, 0x80);
        assert_eq!(cpu.status, 0xC0);
    }
    #[test]
    fn test_adc_occur_overflow_plus_with_carry() {
        // carryを持っていて、plusのときにoverflowが発生する場合。
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x6F, 0x00]);
        cpu.reset();
        cpu.register_a = 0x10;
        cpu.status = 0x01;
        cpu.run();
        assert_eq!(cpu.register_a, 0x80);
        assert_eq!(cpu.status, 0xC0);
    }
    #[test]
    fn test_adc_occur_overflow_minus() {
        // minusのときにoverflowが発生する場合
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x81, 0x00]);
        cpu.reset();
        cpu.register_a = 0x81;
        cpu.run();
        assert_eq!(cpu.register_a, 0x02);
        assert_eq!(cpu.status, 0x41);
    }
    #[test]
    fn test_adc_occur_overflow_minus_with_carry() {
        // minusのときにoverflowが発生する場合
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x80, 0x00]);
        cpu.reset();
        cpu.register_a = 0x80;
        cpu.status = 0x01;
        cpu.run();
        assert_eq!(cpu.register_a, 0x01);
        assert_eq!(cpu.status, 0x41);
    }
    #[test]
    fn test_adc_occur_no_overflow() {
        // overflowが発生しない場合
        let mut cpu = CPU::new();
        cpu.load(vec![0x69, 0x7F, 0x00]);
        cpu.reset();
        cpu.register_a = 0x82;
        cpu.run();
        assert_eq!(cpu.register_a, 0x01);
        assert_eq!(cpu.status, 0x01);
    }
}
