use std::cell::RefCell;

use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::{ExecutionEngine, FunctionLookupError, JitFunction},
    module::Module,
    values::{FunctionValue, PointerValue},
    AddressSpace, OptimizationLevel,
};

use crate::cpu::{self, Cpu, OpCode};

const FUNC_NAME: &str = "dbb";

type CompiledFunc = unsafe extern "C" fn(*mut cpu::Cpu) -> ();

pub struct TranslationBlock<'ctx> {
    fun: JitFunction<'ctx, CompiledFunc>,
}

impl<'ctx> TranslationBlock<'ctx> {
    fn new(fun: JitFunction<'ctx, CompiledFunc>) -> Self {
        Self { fun }
    }

    pub fn execute(&self, cpu: &mut Cpu) {
        unsafe {
            self.fun.call(cpu);
        }
    }
}

extern "C" fn debug_cpu_state(cpu: &Cpu) {
    log::warn!("[LLVM] :: PC: {:#04x}, ACC: {:#4}, LC: {:#4}", cpu.pc, cpu.acc, cpu.lc);
}

struct FunctionContext<'ctx> {
    function: FunctionValue<'ctx>,
    _debug_function: FunctionValue<'ctx>,
    _cpu_ptr: PointerValue<'ctx>,
    acc_ptr: PointerValue<'ctx>,
    lc_ptr: PointerValue<'ctx>,
    pc_ptr: PointerValue<'ctx>,
    halt_ptr: PointerValue<'ctx>,
}

pub struct TranslatorEngine<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
    fun_context: RefCell<Option<FunctionContext<'ctx>>>,
    translation_block: RefCell<Option<TranslationBlock<'ctx>>>,
}

impl<'ctx> TranslatorEngine<'ctx> {

    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("translator_engine");
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::Default)
            .unwrap();
        let builder = context.create_builder();
        Self {
            module,
            execution_engine,
            builder,
            fun_context: RefCell::new(None),
            translation_block: RefCell::new(None),
        }
    }

    pub fn execute(&self, cpu: &mut Cpu) {
        let tb = self.translation_block.borrow();
        tb.as_ref().unwrap().execute(cpu);
    }

    pub fn compile_dynamic_basic_block(&self, dbb: Vec<OpCode>) -> Result<(), String> {
        self.setup_prologue();

        dbb.iter().for_each(|instr| match instr {
            OpCode::HALT => self.halt(),
            OpCode::CLRA => self.clra(),
            OpCode::INC3A => self.inc3a(),
            OpCode::DECA => self.deca(),
            OpCode::SETL => self.setl(),
            OpCode::BACK7 => self.back7(),
        });

        self.setup_epilogue();

        // Print LLVM module to the stderr
        // self.module.print_to_stderr();

        // Verify the module's correctness before executing it.
        
        match self.module.verify() {
        Ok(_) => (),
            Err(msg) => {
                log::error!(
                    "Error while verifying LLVM module: {}",
                    msg.to_str().unwrap()
                );
                return Err("Unable to verify the correctness of the LLVM function".to_string())
            }
        }

        match self.jit_compile() {
            Ok(fun) => {
                // The function has been compiled successfully, increase the id generator
                self.translation_block
                    .replace(Some(TranslationBlock::new(fun)));
                Ok(())
            }
            Err(err) => Err(format!(
                "Something went wrong when compiling the dynamic basic block: {}",
                err.to_string()
            )
            .to_string()),
        }
    }

    fn jit_compile(&self) -> Result<JitFunction<'ctx, CompiledFunc>, FunctionLookupError> {
        unsafe { self.execution_engine.get_function(FUNC_NAME) }
    }

    fn _call_debug(&self) {
        let fun_context = self.fun_context.borrow();
        let fun_context = fun_context.as_ref().unwrap();
        let cpu = self.builder.build_load(fun_context._cpu_ptr, "");
        self.builder
            .build_call(fun_context._debug_function, &[cpu.into()], "");
    }

    fn setup_prologue(&self) {
        let i32_type = self.module.get_context().i32_type();
        let i32_ptr_type = i32_type.ptr_type(AddressSpace::default());
        let bool_ptr_type = self
            .module
            .get_context()
            .bool_type()
            .ptr_type(AddressSpace::default());

        let unit_type = self.module.get_context().void_type();
        let bool_type = self.module.get_context().bool_type();

        let cpu_type = self.module.get_context().opaque_struct_type("struct.cpu");
        cpu_type.set_body(
            &[
                i32_type.into(),
                i32_type.into(),
                i32_type.into(),
                bool_type.into(),
            ],
            false,
        );

        let cpu_struct_ptr_type = cpu_type.ptr_type(AddressSpace::default());

        let print_fun_type = unit_type.fn_type(&[cpu_struct_ptr_type.into()], false);
        let print_fun = self.module.add_function(
            "debug_cpu_state",
            print_fun_type,
            Some(inkwell::module::Linkage::External),
        );

        // Magic trick with execution engine
        self.execution_engine
            .add_global_mapping(&print_fun, debug_cpu_state as usize);

        let fn_type = unit_type.fn_type(&[cpu_struct_ptr_type.into()], false);
        let fun_val = self.module.add_function(FUNC_NAME, fn_type, None);

        let entry_bb = self
            .module
            .get_context()
            .append_basic_block(fun_val, "entry");
        let code_bb = self
            .module
            .get_context()
            .append_basic_block(fun_val, "start");

        self.builder.position_at_end(entry_bb);

        // Alloca struct point
        let cpu_param = fun_val.get_first_param().unwrap().into_pointer_value();
        let cpu_ptr = self.builder.build_alloca(cpu_struct_ptr_type, "cpu");

        let acc_ptr = self.builder.build_alloca(i32_ptr_type, "acc_ptr");
        let lc_ptr = self.builder.build_alloca(i32_ptr_type, "lc_ptr");
        let pc_ptr = self.builder.build_alloca(i32_ptr_type, "pc_ptr");
        let halt_ptr = self.builder.build_alloca(bool_ptr_type, "halt_ptr");

        self.builder.build_store(cpu_ptr, cpu_param);

        let struct_ptr = self.builder.build_load(cpu_ptr, "").into_pointer_value();

        let acc_ptr_val = self.builder.build_struct_gep(struct_ptr, 0, "").unwrap();
        self.builder.build_store(acc_ptr, acc_ptr_val);

        let lc_ptr_val = self.builder.build_struct_gep(struct_ptr, 1, "").unwrap();
        self.builder.build_store(lc_ptr, lc_ptr_val);

        let pc_ptr_val = self.builder.build_struct_gep(struct_ptr, 2, "").unwrap();
        self.builder.build_store(pc_ptr, pc_ptr_val);

        let halt_ptr_val = self.builder.build_struct_gep(struct_ptr, 3, "").unwrap();
        self.builder.build_store(halt_ptr, halt_ptr_val);

        self.builder.build_unconditional_branch(code_bb);
        self.builder.position_at_end(code_bb);

        self.fun_context.replace(Some(FunctionContext {
            function: fun_val,
            _cpu_ptr: cpu_ptr,
            acc_ptr,
            lc_ptr,
            pc_ptr,
            halt_ptr,
            _debug_function: print_fun,
        }));
    }

    fn setup_epilogue(&self) {
        self.builder.build_return(None);
    }

    fn build_increase_program_counter(&self) {
        let fun_context = self.fun_context.borrow();
        let fun_context = fun_context.as_ref().unwrap();
        let one = self.module.get_context().i32_type().const_int(1, false);
        let pc_ptr = self
            .builder
            .build_load(fun_context.pc_ptr, "")
            .into_pointer_value();
        let old_pc = self.builder.build_load(pc_ptr, "").into_int_value();
        let inc_pc = self.builder.build_int_nuw_add(old_pc, one, "");
        self.builder.build_store(pc_ptr, inc_pc);
    }

    fn halt(&self) {
        let fun_context = self.fun_context.borrow();
        let fun_context = fun_context.as_ref().unwrap();
        let true_val = self.module.get_context().bool_type().const_int(1, false);
        let halt_ptr = self
            .builder
            .build_load(fun_context.halt_ptr, "")
            .into_pointer_value();
        self.builder.build_store(halt_ptr, true_val);
        self.build_increase_program_counter();
    }

    fn clra(&self) {
        let fun_context = self.fun_context.borrow();
        let fun_context = fun_context.as_ref().unwrap();
        let zero = self.module.get_context().i32_type().const_zero();
        let acc_ptr = self
            .builder
            .build_load(fun_context.acc_ptr, "")
            .into_pointer_value();
        self.builder.build_store(acc_ptr, zero);
        self.build_increase_program_counter();
    }

    fn inc3a(&self) {
        let fun_context = self.fun_context.borrow();
        let fun_context = fun_context.as_ref().unwrap();
        let three = self.module.get_context().i32_type().const_int(3, false);
        let acc_ptr = self
            .builder
            .build_load(fun_context.acc_ptr, "")
            .into_pointer_value();
        let old_acc = self.builder.build_load(acc_ptr, "").into_int_value();
        let new_acc = self.builder.build_int_nsw_add(old_acc, three, "");
        self.builder.build_store(acc_ptr, new_acc);
        self.build_increase_program_counter();
    }

    fn deca(&self) {
        let fun_context = self.fun_context.borrow();
        let fun_context = fun_context.as_ref().unwrap();
        let one = self.module.get_context().i32_type().const_int(1, false);
        let acc_ptr = self
            .builder
            .build_load(fun_context.acc_ptr, "")
            .into_pointer_value();
        let old_acc = self.builder.build_load(acc_ptr, "").into_int_value();
        let new_acc = self.builder.build_int_nsw_sub(old_acc, one, "");
        self.builder.build_store(acc_ptr, new_acc);
        self.build_increase_program_counter();
    }

    fn setl(&self) {
        let fun_context = self.fun_context.borrow();
        let fun_context = fun_context.as_ref().unwrap();
        let acc_ptr = self
            .builder
            .build_load(fun_context.acc_ptr, "")
            .into_pointer_value();
        let acc_val = self.builder.build_load(acc_ptr, "").into_int_value();
        let lc_ptr = self
            .builder
            .build_load(fun_context.lc_ptr, "")
            .into_pointer_value();
        self.builder.build_store(lc_ptr, acc_val);
        self.build_increase_program_counter();
    }

    fn back7(&self) {
        let fun_context = self.fun_context.borrow();
        let fun_context = fun_context.as_ref().unwrap();
        let i32_type = self.module.get_context().i32_type();

        let zero = i32_type.const_zero();
        let one = i32_type.const_int(1, false);
        let six = i32_type.const_int(6, false);

        let lc_ptr = self
            .builder
            .build_load(fun_context.lc_ptr, "")
            .into_pointer_value();
        let lc_val = self.builder.build_load(lc_ptr, "").into_int_value();
        let dec_lc_val = self.builder.build_int_nsw_sub(lc_val, one, "");
        self.builder.build_store(lc_ptr, dec_lc_val);

        let pc_ptr = self
            .builder
            .build_load(fun_context.pc_ptr, "")
            .into_pointer_value();
        let pc_val = self.builder.build_load(pc_ptr, "").into_int_value();

        // if.then
        let then_bb = self
            .module
            .get_context()
            .append_basic_block(fun_context.function, "if.then");
        let else_bb = self
            .module
            .get_context()
            .append_basic_block(fun_context.function, "if.else");
        let cont_bb = self
            .module
            .get_context()
            .append_basic_block(fun_context.function, "if.cont");

        let icmp = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SGT, dec_lc_val, zero, "");
        self.builder
            .build_conditional_branch(icmp, then_bb, else_bb);

        // then block
        self.builder.position_at_end(then_bb);

        let dec_pc_six = self.builder.build_int_nuw_sub(pc_val, six, "");

        self.builder.build_unconditional_branch(cont_bb);

        // else block
        self.builder.position_at_end(else_bb);
        let inc_pc_one = self.builder.build_int_nuw_add(pc_val, one, "");
        self.builder.build_unconditional_branch(cont_bb);

        // cont block
        self.builder.position_at_end(cont_bb);
        let phi = self.builder.build_phi(i32_type, "");
        phi.add_incoming(&[(&dec_pc_six, then_bb), (&inc_pc_one, else_bb)]);
        // Store the new program counter
        self.builder
            .build_store(pc_ptr, phi.as_basic_value().into_int_value());
    }
}
