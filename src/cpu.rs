use std::{process::exit, fmt::Display};

#[repr(C)]
pub struct Cpu {
    pub acc: i32,   // The accumulator register
    pub lc: i32,    // The loop counter register
    pub pc: u32,    // The program counter register
    pub halt: bool  // Flag keeping the current running state
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Cpu [ acc: {}, lc: {}, pc: {}, halt: {} ]", self.acc, self.lc, self.pc, self.halt)
    }
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            acc: 0,
            lc: 0,
            pc: 0,
            halt: false
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    HALT    =   0,  // HALT = true
    CLRA    =   1,  // A  = 0, PC += 1
    INC3A   =   2,  // A += 3, PC += 1
    DECA    =   3,  // A -= 1, PC += 1
    SETL    =   4,  // L  = A, PC += 1
    BACK7   =   5   // L -= 1, if L > 0 then PC -= 6 else PC += 1
}

impl From<u8> for OpCode {
    fn from(v: u8) -> Self {
        match v {
           0 => Self::HALT,
           1 => Self::CLRA,
           2 => Self::INC3A,
           3 => Self::DECA,
           4 => Self::SETL,
           5 => Self::BACK7,
           _ => {
            log::error!("unknown code `{}`", v);
            exit(-1);
           }
        }
    }
}