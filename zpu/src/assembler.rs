use std::u32;
use std::fs::File;
use std::io::{Read, Write};
use std::collections::HashMap;

use byteorder::{LittleEndian, WriteBytesExt};

use zpu::{Opcode, Register};

#[derive(Debug)]
pub struct AResult {
    pub compile_err: String,
}

impl AResult {
    pub fn new(err: Option<String>) -> AResult {
        let compile_err;
        if err.is_some() {
            compile_err = err.unwrap();
        } else {
            compile_err = String::new();
        }
        AResult {
            compile_err: compile_err,
        }
    }
}

fn write_inst(buffer: &mut Vec<u8>, op: Opcode, reg1: u8, reg2: u8, data: u32) {
    let write_v = ((op.hex_value() as u32) << 16) | ((reg1 as u32) << 8) | reg2 as u32;
//    println!("[BIN-WRITE] {}, {:?}, {}, {}", write_v, op, reg1, reg2);
    buffer.write_u32::<LittleEndian>(write_v).unwrap();
    if reg2 == 0 {
//        println!("[BIN-WRITE-DATA] {}", data);
        buffer.write_u32::<LittleEndian>(data).unwrap();
    }
}

fn write_program(filename: &str, program: &Vec<u8>) {
    let mut file = File::create(filename).unwrap();
    file.write_all(&program).unwrap();
}

pub fn assemble_program(file_in: &str, file_out: &str) -> AResult {
    let mut program = Vec::new();
    let mut instructions = Vec::new();
    let mut label_map = HashMap::new();
    let mut pc = 0;

    let mut file = File::open(file_in).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();

    for line in text.lines() {
        let line = line.to_lowercase();
        if line.is_empty() || line.contains(';') {
            continue;
        }

        let line = line.replace(',', "");
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens[0].contains(":") {
            let label = tokens[0].replace(':', "");
            label_map.insert(label, pc);
        } else if tokens.len() > 1 {
            let opcode = match tokens[0] {
                "nop" => Some(Opcode::NoOp),
                "jmp" => Some(Opcode::Jump),
                "hlt" => Some(Opcode::Halt),
                "inc" => Some(Opcode::Increment),
                "shr" => Some(Opcode::ShiftRight),
                "shl" => Some(Opcode::ShiftLeft),
                "mov" => Some(Opcode::Move),
                "add" => Some(Opcode::Add),
                "sub" => Some(Opcode::Subtract),
                "mul" => Some(Opcode::Multiply),
                "div" => Some(Opcode::Divide),
                "je" => Some(Opcode::IfEqual),
                "jn" => Some(Opcode::IfNotEqual),
                "mmov" => Some(Opcode::MemoryMove),
                "mset" => Some(Opcode::MemorySet),
                "xor" => Some(Opcode::XOr),
                "in" => Some(Opcode::In),
                "out" => Some(Opcode::Out),
                "push" => Some(Opcode::Push),
                "pop" => Some(Opcode::Pop),
                "jz" => Some(Opcode::IfZero),
                "jg" => Some(Opcode::IfGreater),
                "jl" => Some(Opcode::IfLess),
                "cmp" => Some(Opcode::Compare),
                _ => None,
            };

            let mut triggered = false;
            if opcode.is_some() {
                let opcode = opcode.unwrap();
                if opcode == Opcode::Jump || opcode == Opcode::IfGreater || opcode == Opcode::IfLess ||
                    opcode == Opcode::IfEqual || opcode == Opcode::IfNotEqual || opcode == Opcode::IfZero {
                    triggered = true;
                    let label = tokens[1].to_owned();
                    let data = label_map.get(&label);
                    pc += 1;
                    if data.is_some() {
                        let data = data.unwrap();
                        instructions.push((opcode, 0, 0, *data, String::new()));
                    } else {
                        instructions.push((opcode, 0, 0, u32::MAX, label));
                    }
                } else if opcode == Opcode::Increment {
                    triggered = true;
                    let reg = match tokens[1] {
                        "a" => Some(Register::A),
                        "b" => Some(Register::B),
                        "c" => Some(Register::C),
                        "d" => Some(Register::D),
                        "e" => Some(Register::E),
                        "x" => Some(Register::X),
                        "y" => Some(Register::Y),
                        "z" => Some(Register::Z),
                        _ => None,
                    };
                    if reg.is_some() {
                        let reg = reg.unwrap();
                        instructions.push((opcode, reg.hex_value(), 0, 0, String::new()));
                    } else {
                        return AResult::new(Some(format!("Not a register: {}", tokens[1])));
                    }
                }
            }
            if !triggered {
                let reg1 = match tokens[1] {
                    "a" => Some(Register::A),
                    "b" => Some(Register::B),
                    "c" => Some(Register::C),
                    "d" => Some(Register::D),
                    "e" => Some(Register::E),
                    "x" => Some(Register::X),
                    "y" => Some(Register::Y),
                    "z" => Some(Register::Z),
                    _ => None,
                };

                if tokens.len() == 3 {
                    let mut data: Option<u32> = None;
                    let mut reg2 = None;
                    match tokens[2] {
                        "a" => { reg2 = Some(Register::A); },
                        "b" => { reg2 = Some(Register::B); },
                        "c" => { reg2 = Some(Register::C); },
                        "d" => { reg2 = Some(Register::D); },
                        "e" => { reg2 = Some(Register::E); },
                        "x" => { reg2 = Some(Register::X); },
                        "y" => { reg2 = Some(Register::Y); },
                        "z" => { reg2 = Some(Register::Z); },
                        _ => {
                            data = tokens[2].parse().ok();
                        },
                    }

                    if opcode.is_some() && reg1.is_some() && reg2.is_some() {
                        let opcode = opcode.unwrap();
                        let reg1 = reg1.unwrap();
                        let reg2 = reg2.unwrap();
                        pc += 1;
                        instructions.push((opcode, reg1.hex_value(), reg2.hex_value(), 0, String::new()));
                    } else if opcode.is_some() && reg1.is_some() && data.is_some() {
                        let opcode = opcode.unwrap();
                        let reg1 = reg1.unwrap();
                        let data = data.unwrap();
                        pc += 2;
                        instructions.push((opcode, reg1.hex_value(), 0, data, String::new()));
                    } else {
                        return AResult::new(Some(format!("invalid line: {:?}", line)));
                    }
                } else {
                    return AResult::new(Some(format!("invalid line: {:?}", line)));
                }
            }
        } else {
            return AResult::new(Some(format!("invalid line: {:?}", line)));
        }
    }
    for inst in instructions.iter() {
        let opcode = inst.0;
        let r1 = inst.1;
        let r2 = inst.2;
        let data = inst.3;
        let label = inst.4.clone();

        if (opcode == Opcode::Jump || opcode == Opcode::IfGreater || opcode == Opcode::IfLess ||
            opcode == Opcode::IfEqual || opcode == Opcode::IfNotEqual || opcode == Opcode::IfZero) && data == u32::MAX {
            let data = label_map.get(&label);
            if data.is_some() {
                let data = data.unwrap();
                write_inst(&mut program, opcode, 0, 0, *data + 1);
            } else {
                return AResult::new(Some(format!("Label not found! {}", label)));
            }
        } else {
            write_inst(&mut program, opcode, r1, r2, data);
        }
    }
    write_inst(&mut program, Opcode::Halt, 0, 0, 0);

    write_program(file_out, &program);
    return AResult::new(None);
}
