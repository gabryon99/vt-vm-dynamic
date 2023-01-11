pub mod cpu;
pub mod memory;
pub mod program;
pub mod translation;

use caches::Cache;
use cpu::{Cpu, OpCode};
use log::{debug, warn, info};
use memory::{Addressable, Memory};

use inkwell::context::Context;
use program::Program;
use translation::TranslationContext;

const CACHE_SIZE: usize = 32;
const MAX_EXECUTIONS: u64 = 1;

type CodeCache<'ctx> = caches::AdaptiveCache<usize, TranslationContext<'ctx>>;

#[derive(Default)]
pub struct EmulationEngine {
    pub(crate) cpu: Cpu,
    memory: Memory,
}

impl EmulationEngine {
    pub fn load_program(&mut self, program: Program) {
        // Set the initial register values
        self.cpu.acc = program.initial_acc;
        self.cpu.lc = program.initial_lc;

        // Load the program in memory
        self.memory
            .write_chunk(program.data)
            .expect("Failed to write program into memory!");
    }

    fn debug_state(&self) {
        let next_eights = (self.cpu.pc..self.cpu.pc + 8).fold(String::new(), |acc, address| {
            acc + &format!("{:#04x} ", self.memory.read(address as usize))
        });
        debug!(
            "State: PC: {:#04x}, ACC: {:#4}, LC: {:#4} | {}",
            self.cpu.pc, self.cpu.acc, self.cpu.lc, next_eights
        );
    }

    fn interpret(&mut self) -> Vec<OpCode> {

        let mut dynamic_block = Vec::new();

        loop {
            let instr = OpCode::try_from(self.memory.read(self.cpu.pc))
                .expect("Unknown OpCode read from memory.");

            dynamic_block.push(instr);

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

        // As long the machine is not stopped
        while !self.cpu.halt {
            let pc = self.cpu.pc;
            let tbb = code_cache.get_mut(&pc);

            if let Some(tbb) = tbb {
                tbb.executions += 1;

                if tbb.executions >= MAX_EXECUTIONS && !tbb.has_compiled() {
                    match tbb.compile_dynamic_basic_block() {
                        Ok(_) => {
                            debug!("translation block successfully compiled into native code!");
                        }
                        Err(e) => {
                            warn!("wasn't capable to compile the translation block: {}", e);
                        }
                    }
                }

                if tbb.has_compiled() {
                    debug!("executing native code...");
                    tbb.execute(&mut self.cpu);
                } else {
                    let _ = self.interpret();
                }

                self.debug_state();

            } else {

                debug!("translation block not found...");

                // Interpret instructions normally and Build translation block
                let dbb = self.interpret();
                let tbb = TranslationContext::new(&llvm_context, dbb);
                code_cache.put(pc, tbb);

                self.debug_state();
            }
        }

        info!("{}", self.cpu);

    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::program::Program;

    mod bytecode_gen {

        use std::os::raw::{c_char, c_int};

        extern "C" {
            pub fn init(
                buf: *mut c_char,
                size: c_int,
                prob: *mut c_int,
                seed: c_int,
                r_a: *mut c_int,
                r_l: *mut c_int,
            );
        }
    }

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn generate_scenario(size: usize, seed: i32, mut probs: [i32; 5]) -> Program {
        let mut r_a: i32 = 0;
        let mut r_l: i32 = 0;

        let mut data: Vec<u8> = Vec::new();
        data.resize(size, 0);

        unsafe {
            bytecode_gen::init(
                data.as_mut_ptr() as *mut i8,
                size as i32,
                probs.as_mut_ptr(),
                seed,
                &mut r_a,
                &mut r_l,
            )
        }

        Program::new(data, r_a, r_l)
    }

    #[test]
    pub fn scenario_1() {
        // acc: 30003, lc: 7
        init();
        let prog = generate_scenario(10_000, 1, [0, 1, 0, 0, 0]);
        let mut vm = EmulationEngine::default();
        vm.load_program(prog);
        vm.main_loop();
        assert_eq!(vm.cpu, Cpu::new(30003, 7, 10000, true));
    }

    #[test]
    pub fn scenario_2() {
        // acc: -1, lc: 7
        init();
        let prog = generate_scenario(10_000, 1, [1, 1, 1, 0, 0]);
        let mut vm = EmulationEngine::default();
        vm.load_program(prog);
        vm.main_loop();
        assert_eq!(vm.cpu, Cpu::new(-1, 7, 10000, true));
    }

    #[test]
    pub fn scenario_3() {
        // acc: 95, lc: -21
        init();
        let prog = generate_scenario(10_000, 1, [1, 9, 1, 5, 5]);
        let mut vm = EmulationEngine::default();
        vm.load_program(prog);
        vm.main_loop();
        assert_eq!(vm.cpu, Cpu::new(95, -21, 10_000, true));
    }

    #[test]
    pub fn scenario_4() {
        // acc: 138, lc: 0
        init();
        let prog = generate_scenario(50_000, 1, [1, 9, 1, 5, 5]);
        let mut vm = EmulationEngine::default();
        vm.load_program(prog);
        vm.main_loop();
        assert_eq!(vm.cpu, Cpu::new(128, 0, 50_000, true));
    }

    #[test]
    pub fn scenario_custom_0() {
        init();
        let prog = Program::new(vec![2, 2, 2, 2, 2, 2, 5, 5, 0], 0, 2);
        let mut vm = EmulationEngine::default();
        vm.load_program(prog);
        vm.main_loop();
        assert_eq!(vm.cpu, Cpu::new(36, -1, 9, true));
    }

    #[test]
    pub fn scenario_custom_1() {
        init();
        let prog = Program::new(vec![4, 2, 2, 2, 2, 2, 2, 2, 5, 5, 0], 0, 2);
        let mut vm = EmulationEngine::default();
        vm.load_program(prog);
        vm.main_loop();
        assert_eq!(vm.cpu, Cpu::new(21, -2, 11, true));
    }
}
