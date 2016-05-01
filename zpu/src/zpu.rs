use std::fs::File;
use std::io::{Cursor, Seek, SeekFrom, Read};
use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    NoOp,
    Jump,
    Halt,
    Increment,
    ShiftRight,
    ShiftLeft,
    Move,
    Add,
    Subtract,
    Multiply,
    Divide,
    IfEqual,
    IfNotEqual,
    MemoryMove,
    MemorySet,
    XOr,
    In,
    Out,
    Push,
    Pop,
    IfZero,
    IfGreater,
    IfLess,
    Compare,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    X,
    Y,
    Z,
    Null,
}

impl Register {
    pub fn hex_value(&self) -> u8 {
        match *self {
            Register::Null => 0x0,
            Register::A => 0x1,
            Register::B => 0x2,
            Register::C => 0x3,
            Register::D => 0x4,
            Register::E => 0x5,
            Register::X => 0x6,
            Register::Y => 0x7,
            Register::Z => 0x8,
        }
    }

    pub fn from_value(value: u8) -> Register {
        match value {
            0x1 => Register::A,
            0x2 => Register::B,
            0x3 => Register::C,
            0x4 => Register::D,
            0x5 => Register::E,
            0x6 => Register::X,
            0x7 => Register::Y,
            0x8 => Register::Z,
            _ => Register::Null,
        }
    }
}

impl Opcode {
    pub fn hex_value(&self) -> u16 {
        match *self {
            Opcode::NoOp => 0x0,
            Opcode::Jump => 0x1,
            Opcode::Halt => 0x2,
            Opcode::Increment => 0x3,
            Opcode::ShiftRight => 0x4,
            Opcode::ShiftLeft => 0x5,
            Opcode::Move => 0x6,
            Opcode::Add => 0x7,
            Opcode::Subtract => 0x8,
            Opcode::Multiply => 0x9,
            Opcode::Divide => 0xA,
            Opcode::IfEqual => 0xB,
            Opcode::IfNotEqual => 0xC,
            Opcode::MemoryMove => 0xD,
            Opcode::MemorySet => 0xE,
            Opcode::XOr => 0xF,
            Opcode::In => 0x10,
            Opcode::Out => 0x11,
            Opcode::Push => 0x12,
            Opcode::Pop => 0x13,
            Opcode::IfZero => 0x14,
            Opcode::IfGreater => 0x15,
            Opcode::IfLess => 0x16,
            Opcode::Compare => 0x17,
        }
    }

    pub fn from_value(value: u16) -> Opcode {
        match value {
            0x0 => Opcode::NoOp,
            0x1 => Opcode::Jump,
            0x2 => Opcode::Halt,
            0x3 => Opcode::Increment,
            0x4 => Opcode::ShiftRight,
            0x5 => Opcode::ShiftLeft,
            0x6 => Opcode::Move,
            0x7 => Opcode::Add,
            0x8 => Opcode::Subtract,
            0x9 => Opcode::Multiply,
            0xA => Opcode::Divide,
            0xB => Opcode::IfEqual,
            0xC => Opcode::IfNotEqual,
            0xD => Opcode::MemoryMove,
            0xE => Opcode::MemorySet,
            0xF => Opcode::XOr,
            0x10 => Opcode::In,
            0x11 => Opcode::Out,
            0x12 => Opcode::Push,
            0x13 => Opcode::Pop,
            0x14 => Opcode::IfZero,
            0x15 => Opcode::IfGreater,
            0x16 => Opcode::IfLess,
            0x17 => Opcode::Compare,
            _ => Opcode::NoOp,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Output {
    pub port: u32,
    pub data: u32,
}

impl Output {
    pub fn new(port: u32, data: u32) -> Output {
        Output {
            port: port,
            data: data,
        }
    }
}

#[derive(Debug)]
pub struct ZResult {
    pub running: bool,
    pub output: Option<Output>,
}

impl ZResult {
    pub fn new(running: bool, output: Option<Output>) -> ZResult {
        ZResult {
            running: running,
            output: output,
        }
    }
}

pub struct ZPU {
    pub program: Cursor<Vec<u8>>,
    pub registers: [u32; 8],
    pub pc: u32,
    pub cmp_flag: i32,
    pub zero_flag: bool,
    pub running: bool,
}

impl ZPU {
    pub fn new(filename: &str) -> ZPU {
        let mut file = File::open(filename).unwrap();
        let mut file_buffer = Vec::new();
        file.read_to_end(&mut file_buffer).unwrap();
        let program = Cursor::new(file_buffer);
        ZPU {
            program: program,
            registers: [0, 0, 0, 0, 0, 0, 0, 0],
            pc: 0,
            cmp_flag: 0,
            zero_flag: false,
            running: true,
        }
    }

    fn reset(&mut self) {
        self.registers = [0, 0, 0, 0, 0, 0, 0, 0];
        self.pc = 0;
        self.cmp_flag = 0;
        self.zero_flag = false;
        self.running = true;
    }

    fn jump(&mut self, value: u32) -> Option<Output> {
        self.pc = value;
        None
    }

    fn je(&mut self, value: u32) -> Option<Output> {
        if self.cmp_flag == 0 {
            self.pc = value;
        }
        None
    }

    fn jn(&mut self, value: u32) -> Option<Output> {
        if self.cmp_flag != 0 {
            self.pc = value;
        }
        None
    }

    fn jg(&mut self, value: u32) -> Option<Output> {
        if self.cmp_flag > 0 {
            self.pc = value;
        }
        None
    }

    fn jl(&mut self, value: u32) -> Option<Output> {
        if self.cmp_flag < 0 {
            self.pc = value;
        }
        None
    }

    fn jz(&mut self, value: u32) -> Option<Output> {
        if self.zero_flag {
            self.pc = value;
        }
        None
    }

    fn cmp(&mut self, reg: Register, value: u32) -> Option<Output> {
        let idx = (reg.hex_value() as usize) - 1;
        if self.registers[idx] == value {
            self.zero_flag = true;
            self.cmp_flag = 0;
        } else if self.registers[idx] < value {
            self.zero_flag = false;
            self.cmp_flag = -1;
        } else {
            self.zero_flag = false;
            self.cmp_flag = 1;
        }
        None
    }

    fn inc(&mut self, reg: Register) -> Option<Output> {
        let idx = (reg.hex_value() as usize) - 1;
        self.registers[idx] += 1;
        None
    }

    fn shr(&mut self, reg: Register, value: u32) -> Option<Output> {
        let idx = (reg.hex_value() as usize) - 1;
        self.registers[idx] = self.registers[idx] >> value;
        None
    }

    fn shl(&mut self, reg: Register, value: u32) -> Option<Output> {
        let idx = (reg.hex_value() as usize) - 1;
        self.registers[idx] = self.registers[idx] << value;
        None
    }

    fn mov(&mut self, reg: Register, value: u32) -> Option<Output> {
        let idx = (reg.hex_value() as usize) - 1;
        self.registers[idx] = value;
        None
    }

    fn add(&mut self, reg: Register, value: u32) -> Option<Output> {
        let idx = (reg.hex_value() as usize) - 1;
        self.registers[idx] = self.registers[idx] + value;
        None
    }

    fn sub(&mut self, reg: Register, value: u32) -> Option<Output> {
        let idx = (reg.hex_value() as usize) - 1;
        self.registers[idx] = self.registers[idx] - value;
        None
    }

    fn mul(&mut self, reg: Register, value: u32) -> Option<Output> {
        let idx = (reg.hex_value() as usize) - 1;
        self.registers[idx] = self.registers[idx] * value;
        None
    }

    fn div(&mut self, reg: Register, value: u32) -> Option<Output> {
        let idx = (reg.hex_value() as usize) - 1;
        self.registers[idx] = self.registers[idx] / value;
        None
    }

    fn out(&mut self, reg: Register, value: u32) -> Option<Output> {
        let idx = (reg.hex_value() as usize) - 1;
        let port = self.registers[idx];
        Some(Output::new(port, value))
    }

    pub fn load_program(&mut self, filename: &str) {
        let mut file = File::open(filename).unwrap();
        let mut file_buffer = Vec::new();
        file.read_to_end(&mut file_buffer).unwrap();
        self.program = Cursor::new(file_buffer);

        self.reset();
    }

    pub fn step(&mut self) -> ZResult {
        if self.running {
            let seek_val = (self.pc as u64) * 4;
            self.program.seek(SeekFrom::Start(seek_val)).unwrap();
            let value = self.program.read_u32::<LittleEndian>().unwrap();
            let inst = Opcode::from_value((value >> 16) as u16);
            let reg1 = Register::from_value((value >> 8) as u8);
            let reg2 = Register::from_value((value) as u8);

            let mut data = None;
            if reg2 == Register::Null {
                data = Some(self.program.read_u32::<LittleEndian>().unwrap());
                //println!("{:?} {:?}, {:?}", inst, reg1, data.unwrap());
            } else {
                //println!("{:?} {:?}, {:?}", inst, reg1, reg2);
            }

            let result = self.execute(inst, reg1, reg2, data);
            return ZResult::new(self.running, result);
        } else {
            return ZResult::new(self.running, None);
        }
    }

    pub fn execute(&mut self, inst: Opcode, reg1: Register, reg2: Register, data: Option<u32>) -> Option<Output> {
        let val;
        if reg2 == Register::Null {
            if data.is_some() {
                let data = data.unwrap();
                val = data;
                self.pc += 2;
            } else {
                panic!("Invalid instruction being executed!");
            }
        } else {
            self.pc += 1;
            val = self.registers[(reg2.hex_value() - 1) as usize];
        }

        let output = match inst {
            Opcode::NoOp => None,
            Opcode::Move => self.mov(reg1, val),
            Opcode::Increment => self.inc(reg1),
            Opcode::ShiftRight => self.shr(reg1, val),
            Opcode::ShiftLeft => self.shl(reg1, val),
            Opcode::Add => self.add(reg1, val),
            Opcode::Subtract => self.sub(reg1, val),
            Opcode::Multiply => self.mul(reg1, val),
            Opcode::Divide => self.div(reg1, val),
            Opcode::Jump => self.jump(val),
            Opcode::IfEqual => self.je(val),
            Opcode::IfNotEqual => self.jn(val),
            Opcode::IfGreater => self.jg(val),
            Opcode::IfLess => self.jl(val),
            Opcode::IfZero => self.jz(val),
            Opcode::Compare => self.cmp(reg1, val),
            Opcode::Out => self.out(reg1, val),
            Opcode::Halt => { self.running = false; None},
            _ => { println!("[EXEC] Not yet handled!"); None},
        };

        //println!("[{:?}] {:?} | [PC] {} | [FLAGS] C: {}, Z: {}", inst, self.registers, self.pc, self.cmp_flag, self.zero_flag);
        return output;
    }
}
