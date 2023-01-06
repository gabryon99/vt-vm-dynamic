use std::{
    i32,
    os::raw::{c_char, c_int}
};

use vt_vm_dyn::{self, program::Program};

// Add binding for `init` function contained inside `tests/gen.c`.
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

/// Returns a pseudo-random scenario generated by the given C program.
fn generate_scenario(size: usize, seed: i32, mut probs: [i32; 5]) -> Program {
    let mut r_a: i32 = 0;
    let mut r_l: i32 = 0;

    let mut data: Vec<u8> = Vec::new();
    data.resize(size, 0);

    unsafe {
        init(
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
    // env_logger::init();
    let prog = generate_scenario(10_000, 1, [0, 1, 0, 0, 0]);
    let mut vm = vt_vm_dyn::EmulationEngine::new();
    vm.load_program(prog);
    vm.main_loop();
}

#[test]
pub fn scenario_2() {
    // acc: -1, lc: 7
    // env_logger::init();
    let prog = generate_scenario(10_000, 1, [1, 1, 1, 0, 0]);
    let mut vm = vt_vm_dyn::EmulationEngine::new();
    vm.load_program(prog);
    vm.main_loop();
}

#[test]
pub fn scenario_3() {
    // acc: 95, lc: -21
    // env_logger::init();
    let prog = generate_scenario(10_000, 1, [1, 9, 1, 5, 5]);
    let mut vm = vt_vm_dyn::EmulationEngine::new();
    vm.load_program(prog);
    vm.main_loop();
}

#[test]
pub fn scenario_4() {
    // acc: 138, lc: 0
    // env_logger::init();
    let prog = generate_scenario(50_000, 1, [1, 9, 1, 5, 5]);
    let mut vm = vt_vm_dyn::EmulationEngine::new();
    vm.load_program(prog);
    vm.main_loop();
}

#[test]
pub fn scenario_custom_0() {
    env_logger::init();
    let prog = Program { 
        data: vec![2, 2, 2, 2, 2, 2, 5, 5, 0], 
        initial_acc: 0,
        initial_lc: 2, 
    };
    let mut vm = vt_vm_dyn::EmulationEngine::new();
    vm.load_program(prog);
    vm.main_loop();
}

#[test]
pub fn scenario_custom_1() {
    env_logger::init();
    let prog = Program { 
        data: vec![4, 2, 2, 2, 2, 2, 2, 2, 5, 5, 0], 
        initial_acc: 0,
        initial_lc: 2, 
    };
    let mut vm = vt_vm_dyn::EmulationEngine::new();
    vm.load_program(prog);
    vm.main_loop();
}