pub mod cpu;
pub mod memory;
pub mod program;
pub mod translation;

use std::process::exit;

use caches::Cache;
use cpu::{Cpu, OpCode};
use inkwell::context::Context;
use memory::{Addressable, Memory};
use program::Program;
use translation::TranslatorEngine;

const CACHE_SIZE: usize = 32;

type CodeCache<'ctx> = caches::AdaptiveCache<u32, TranslatorEngine<'ctx>>;

pub struct EmulationEngine {
    cpu: Cpu,
    memory: Memory,
}

impl EmulationEngine {

    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            memory: Memory::new(),
        }
    }

    pub fn load_program(&mut self, program: Program) {
        // Set the initial register values
        self.cpu.acc = program.initial_acc;
        self.cpu.lc = program.initial_lc;

        // Load the program in memory
        match self.memory.write_chunk(program.data) {
            Ok(()) => (),
            Err(e) => {
                log::error!(
                    "an error occurred when loading the program in memory: {}",
                    e
                );
                exit(-1);
            }
        }
    }

    #[inline(always)]
    fn debug_state(&self) {
        #[cfg(debug_assertions)]
        {
            let next_eights = (self.cpu.pc..self.cpu.pc + 8).fold(String::new(), |acc, address| {
                acc + &format!("{:#04x} ", self.memory.read(address as usize)).to_string()
            });
            log::debug!(
                "State: PC: {:#04x}, ACC: {:#4}, LC: {:#4} | {}",
                self.cpu.pc,
                self.cpu.acc,
                self.cpu.lc,
                next_eights
            );
        }
    }

    fn interpret(&mut self) -> Vec<OpCode> {

        let mut dynamic_block = Vec::new();

        loop {
            let instr: OpCode = self.memory.read(self.cpu.pc as usize).into();
            dynamic_block.push(instr);

            // log::debug!("Executing instruction: {:?}", instr);
            // log::debug!("PC: {:#04x}, ACC: {:#6}, LC: {:#6}", self.cpu.pc, self.cpu.acc, self.cpu.lc);
            self.debug_state();

            match instr {
                OpCode::HALT => {
                    self.cpu.halt = true;
                    self.cpu.pc += 1;
                    break dynamic_block;
                }
                OpCode::CLRA => {
                    self.cpu.acc = 0;
                    self.cpu.pc += 1;
                }
                OpCode::INC3A => {
                    self.cpu.acc += 3;
                    self.cpu.pc += 1;
                }
                OpCode::DECA => {
                    self.cpu.acc -= 1;
                    self.cpu.pc += 1;
                }
                OpCode::SETL => {
                    self.cpu.lc = self.cpu.acc;
                    self.cpu.pc += 1;
                }
                OpCode::BACK7 => {
                    self.cpu.lc -= 1;
                    if self.cpu.lc > 0 {
                        self.cpu.pc -= 6;
                    } else {
                        self.cpu.pc += 1;
                    }
                    break dynamic_block;
                }
            }
        }
    }

    pub fn main_loop(&mut self) {

        let llvm_context = Context::create();
        let mut code_cache = CodeCache::new(CACHE_SIZE).unwrap();

        // As long the machine is not halt
        while !self.cpu.halt {
            let pc = self.cpu.pc.clone();
            let translator = code_cache.get(&pc);

            if let Some(translator) = translator {
                log::debug!(
                    "found translation block for pc: {}, executing it...",
                    self.cpu.pc
                );
                translator.execute(&mut self.cpu);
                // PC is not updated?
                self.debug_state();
                log::debug!("after execution: pc={}", self.cpu.pc);
            } else {
                //log::debug!("translation block not found...");

                // Interpret instructions normally
                let dbb = self.interpret();
                // log::debug!("translation block: {:?}, pc: {}", dbb, self.cpu.pc);

                // Build translation block
                let translator = TranslatorEngine::new(&llvm_context);
                match translator.compile_dynamic_basic_block(dbb) {
                    Ok(_) => {
                        log::debug!("translation block successfully compiled into native code!");
                        code_cache.put(pc, translator);
                    }
                    Err(e) => {
                        log::warn!("wasn't capable to compile the translation block: {}", e);
                    }
                }
            }
        }

        self.debug_state();

        println!("[info] :: {}", self.cpu);
    }


}
