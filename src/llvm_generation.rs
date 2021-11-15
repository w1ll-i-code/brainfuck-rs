use crate::ast::Command;
use crate::optimiser::CommandFolded;
use crate::Config;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::values::{FunctionValue, PointerValue};
use inkwell::{AddressSpace, IntPredicate, OptimizationLevel};
use std::cell::Cell;

pub struct Generator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    functions: Functions<'ctx>,
    label_count: Cell<usize>,
}

struct Functions<'ctx> {
    calloc_fn: FunctionValue<'ctx>,
    getchar_fn: FunctionValue<'ctx>,
    putchar_fn: FunctionValue<'ctx>,
    main_fn: FunctionValue<'ctx>,
}

impl<'ctx> Generator<'ctx> {
    pub fn new(context: &'ctx Context, config: &Config) -> Generator<'ctx> {
        Self::init_targets();
        let module = context.create_module(&config.output_file);

        let functions = Self::init_functions(context, &module);

        Generator {
            context,
            module,
            builder: context.create_builder(),
            functions,
            label_count: Cell::new(0),
        }
    }

    fn init_targets() {
        Target::initialize_all(&InitializationConfig::default());
    }

    pub fn generate(&self, program: &[CommandFolded]) {
        let (data, ptr) = self.build_main();
        self.init_pointers(&data, &ptr);

        self.build_program(&ptr, program);

        self.build_free(&data);
        self.return_zero();
    }

    fn build_program(&self, ptr: &PointerValue, program: &[CommandFolded]) {
        use CommandFolded::*;
        for command in program {
            match command {
                Add(i) => self.build_add(*i, ptr),
                Move(i) => self.build_add_ptr(*i as isize, ptr),
                Loop(sub_program) => self.build_loop(ptr, sub_program),
                MoveValue { pos_rel, mul } => self.build_move_value(ptr, *pos_rel, *mul),
                SetZero => self.build_set_zero(ptr),
                Read => self.build_get(ptr),
                Print => self.build_put(ptr),
            }
        }
    }

    fn init_functions(context: &'ctx Context, module: &Module<'ctx>) -> Functions<'ctx> {
        let i32_type = context.i32_type();
        let i64_type = context.i64_type();
        let i8_type = context.i8_type();
        let i8_ptr_type = i8_type.ptr_type(AddressSpace::Generic);

        let calloc_fn_type = i8_ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let calloc_fn = module.add_function("calloc", calloc_fn_type, Some(Linkage::External));

        let getchar_fn_type = i32_type.fn_type(&[], false);
        let getchar_fn = module.add_function("getchar", getchar_fn_type, Some(Linkage::External));

        let putchar_fn_type = i32_type.fn_type(&[i32_type.into()], false);
        let putchar_fn = module.add_function("putchar", putchar_fn_type, Some(Linkage::External));

        let main_fn_type = i32_type.fn_type(&[], false);
        let main_fn = module.add_function("main", main_fn_type, Some(Linkage::External));

        Functions {
            calloc_fn,
            getchar_fn,
            putchar_fn,
            main_fn,
        }
    }

    fn build_main(&self) -> (PointerValue, PointerValue) {
        let basic_block = self
            .context
            .append_basic_block(self.functions.main_fn, "entry");
        self.builder.position_at_end(basic_block);

        let i8_type = self.context.i8_type();
        let i8_ptr_type = i8_type.ptr_type(AddressSpace::Generic);

        let data = self.builder.build_alloca(i8_ptr_type, "data");
        let ptr = self.builder.build_alloca(i8_ptr_type, "ptr");

        (data, ptr)
    }

    fn init_pointers(&self, ptr: &PointerValue, data: &PointerValue) {
        let i64_type = self.context.i64_type();
        let i64_memory_size = i64_type.const_int(30_000, false);
        let i64_element_size = i64_type.const_int(1, false);

        let data_ptr = self.builder.build_call(
            self.functions.calloc_fn,
            &[i64_memory_size.into(), i64_element_size.into()],
            "calloc_call",
        );
        let data_ptr_result: Result<_, _> = data_ptr.try_as_basic_value().flip().into();
        let data_ptr_basic_val = data_ptr_result.expect("calloc returned void for some reason!");

        self.builder.build_store(*data, data_ptr_basic_val);
        self.builder.build_store(*ptr, data_ptr_basic_val);
    }

    fn build_add_ptr(&self, amount: isize, ptr: &PointerValue) {
        let i64_type = self.context.i64_type();
        let i64_amount = i64_type.const_int(amount as u64, false);
        let ptr_load = self
            .builder
            .build_load(*ptr, "load ptr")
            .into_pointer_value();
        // unsafe because we are calling an unsafe function, since we could index out of bounds of the calloc
        let result = unsafe {
            self.builder
                .build_in_bounds_gep(ptr_load, &[i64_amount], "add to pointer")
        };
        self.builder.build_store(*ptr, result);
    }

    fn build_add(&self, amount: isize, ptr: &PointerValue) {
        let i64_type = self.context.i64_type();
        let i64_amount = i64_type.const_int(amount as u64, false);
        let ptr_load = self
            .builder
            .build_load(*ptr, "load ptr")
            .into_pointer_value();
        let ptr_val = self.builder.build_load(ptr_load, "load ptr value");
        let result =
            self.builder
                .build_int_add(ptr_val.into_int_value(), i64_amount, "add to data ptr");
        self.builder.build_store(ptr_load, result);
    }

    fn build_set_zero(&self, ptr: &PointerValue) {
        let i64_type = self.context.i64_type();
        let i64_amount = i64_type.const_int(0, false);
        let ptr_load = self
            .builder
            .build_load(*ptr, "load ptr")
            .into_pointer_value();
        self.builder.build_store(ptr_load, i64_amount);
    }

    fn build_move_value(&self, ptr: &PointerValue, pos_rel: isize, mul: isize) {
        let i64_type = self.context.i64_type();
        let i64_pos = i64_type.const_int(pos_rel as u64, false);
        let i64_mul = i64_type.const_int(mul as u64, false);

        let ptr_load = self
            .builder
            .build_load(*ptr, "load ptr")
            .into_pointer_value();

        let ptr_val = self.builder.build_load(ptr_load, "load ptr value");
        let ptr_val =
            self.builder
                .build_int_mul(ptr_val.into_int_value(), i64_mul, "add to data ptr");

        // unsafe because we are calling an unsafe function, since we could index out of bounds of the calloc
        let ptr_load = unsafe {
            self.builder
                .build_in_bounds_gep(ptr_load, &[i64_pos], "add to pointer")
        };

        let i64_stored = self
            .builder
            .build_load(ptr_load, "load ptr value")
            .into_int_value();
        let result = self
            .builder
            .build_int_add(ptr_val, i64_stored, "add result to element");

        self.builder.build_store(ptr_load, result);
    }

    fn build_get(&self, ptr: &PointerValue) {
        let getchar_call = self
            .builder
            .build_call(self.functions.getchar_fn, &[], "getchar call");
        let getchar_result: Result<_, _> = getchar_call.try_as_basic_value().flip().into();
        let getchar_basicvalue = getchar_result.expect("getchar returned void for some reason!");
        let i8_type = self.context.i8_type();
        let truncated = self.builder.build_int_truncate(
            getchar_basicvalue.into_int_value(),
            i8_type,
            "getchar truncate result",
        );
        let ptr_value = self
            .builder
            .build_load(*ptr, "load ptr value")
            .into_pointer_value();
        self.builder.build_store(ptr_value, truncated);
    }

    fn build_put(&self, ptr: &PointerValue) {
        let char_to_put = self.builder.build_load(
            self.builder
                .build_load(*ptr, "load ptr value")
                .into_pointer_value(),
            "load ptr ptr value",
        );
        let s_ext = self.builder.build_int_s_extend(
            char_to_put.into_int_value(),
            self.context.i32_type(),
            "putchar sign extend",
        );
        self.builder
            .build_call(self.functions.putchar_fn, &[s_ext.into()], "putchar call");
    }

    fn build_loop(&self, ptr: &PointerValue, program: &[CommandFolded]) {
        let start = self.context.append_basic_block(
            self.functions.main_fn,
            format!("loop_start_{}", self.label_count.get()).as_str(),
        );
        let body = self.context.append_basic_block(
            self.functions.main_fn,
            format!("loop_body_{}", self.label_count.get()).as_str(),
        );
        let end = self.context.append_basic_block(
            self.functions.main_fn,
            format!("loop_end_{}", self.label_count.get()).as_str(),
        );

        self.label_count.replace(self.label_count.get() + 1);

        self.builder.build_unconditional_branch(start);
        self.builder.position_at_end(start);

        let i8_type = self.context.i8_type();
        let i8_zero = i8_type.const_int(0, false);
        let ptr_load = self
            .builder
            .build_load(*ptr, "load ptr")
            .into_pointer_value();
        let ptr_value = self
            .builder
            .build_load(ptr_load, "load ptr value")
            .into_int_value();
        let cmp = self.builder.build_int_compare(
            IntPredicate::NE,
            ptr_value,
            i8_zero,
            "compare value at pointer to zero",
        );

        // jump to the while_end if the data at ptr was zero
        self.builder.build_conditional_branch(cmp, body, end);
        self.builder.position_at_end(body);

        self.build_program(ptr, program);

        self.builder.build_unconditional_branch(start);
        self.builder.position_at_end(end);
    }

    fn build_free(&self, data: &PointerValue) {
        self.builder
            .build_free(self.builder.build_load(*data, "load").into_pointer_value());
    }

    fn return_zero(&self) {
        let i32_type = self.context.i32_type();
        let i32_zero = i32_type.const_int(0, false);
        self.builder.build_return(Some(&i32_zero));
    }

    pub fn write_to_file(&self, output_filename: &str) -> Result<(), String> {
        let target_triple = TargetMachine::get_default_triple();
        let cpu = TargetMachine::get_host_cpu_name().to_string();
        let features = TargetMachine::get_host_cpu_features().to_string();

        let target = Target::from_triple(&target_triple).map_err(|e| format!("{:?}", e))?;
        let target_machine = target
            .create_target_machine(
                &target_triple,
                &cpu,
                &features,
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| "Unable to create target machine!".to_string())?;

        target_machine
            .write_to_file(&self.module, FileType::Object, output_filename.as_ref())
            .map_err(|e| format!("{:?}", e))
    }
}
