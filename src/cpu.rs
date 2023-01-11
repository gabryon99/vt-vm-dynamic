use std::fmt::Display;

#[repr(C)]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Cpu {
    pub acc: i32,   // The accumulator register
    pub lc: i32,    // The loop counter register
    pub pc: usize,  // The program counter register
    pub halt: bool, // Flag keeping the current running state
}

impl Cpu {
    pub fn new(acc: i32, lc: i32, pc: usize, halt: bool) -> Self {
        Self { acc, lc, pc, halt }
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    HALT = 0,  // HALT = true
    CLRA = 1,  // A  = 0, PC += 1
    INC3A = 2, // A += 3, PC += 1
    DECA = 3,  // A -= 1, PC += 1
    SETL = 4,  // L  = A, PC += 1
    BACK7 = 5, // L -= 1, if L > 0 then PC -= 6 else PC += 1
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            v if v == Self::HALT as u8 => Ok(Self::HALT),
            v if v == Self::CLRA as u8 => Ok(Self::CLRA),
            v if v == Self::INC3A as u8 => Ok(Self::INC3A),
            v if v == Self::DECA as u8 => Ok(Self::DECA),
            v if v == Self::SETL as u8 => Ok(Self::SETL),
            v if v == Self::BACK7 as u8 => Ok(Self::BACK7),
            _ => Err(()),
        }
    }
}
