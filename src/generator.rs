//!
//! The LLVM generator.
//!

use std::collections::HashMap;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicTypeEnum;
use inkwell::types::FunctionType;
use inkwell::types::IntType;
use inkwell::types::StringRadix;
use inkwell::values::BasicValue;
use inkwell::values::BasicValueEnum;
use inkwell::values::FunctionValue;
use inkwell::values::InstructionOpcode;
use inkwell::values::IntValue;
use inkwell::values::PointerValue;
use inkwell::IntPredicate;
use regex::Regex;

use crate::parser::block::statement::expression::function_call::FunctionCall;
use crate::parser::block::statement::expression::Expression;
use crate::parser::block::statement::for_loop::ForLoop;
use crate::parser::block::statement::function_definition::FunctionDefinition;
use crate::parser::block::statement::if_conditional::IfConditional;
use crate::parser::block::statement::switch::Switch;
use crate::parser::block::statement::variable_declaration::VariableDeclaration;
use crate::parser::block::statement::Statement;
use crate::parser::block::Block;
use crate::parser::identifier::Identifier;
use crate::parser::literal::Literal;
use crate::parser::r#type::Type;

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub function: Option<FunctionValue<'ctx>>,
    pub leave_bb: Option<BasicBlock<'ctx>>,
    pub break_bb: Option<BasicBlock<'ctx>>,
    pub continue_bb: Option<BasicBlock<'ctx>>,
    pub variables: HashMap<String, PointerValue<'ctx>>,
    pub functions: HashMap<String, FunctionValue<'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    fn translate_type(&self, t: &Option<Type>) -> IntType<'ctx> {
        match t {
            Some(Type::Bool) => self.context.bool_type(),
            Some(Type::Int(n)) => self.context.custom_width_int_type(*n),
            Some(Type::UInt(n)) => self.context.custom_width_int_type(*n),
            _ => self.context.custom_width_int_type(256),
        }
    }

    fn translate_fn_type(
        &self,
        ret_values: &[Identifier],
        par_types: &[BasicTypeEnum<'ctx>],
    ) -> FunctionType<'ctx> {
        if ret_values.is_empty() {
            self.context.void_type().fn_type(par_types, false)
        } else if ret_values.len() == 1 {
            self.translate_type(&ret_values[0].yul_type)
                .fn_type(par_types, false)
        } else {
            let ret_types: Vec<_> = ret_values
                .iter()
                .map(|v| BasicTypeEnum::IntType(self.translate_type(&v.yul_type)))
                .collect();
            let ret_type = self.context.struct_type(&ret_types[..], false);
            ret_type.fn_type(par_types, false)
        }
    }

    fn translate_builtin(&self, call: &FunctionCall) -> Option<BasicValueEnum> {
        // TODO: Figure out how to use high-order functions to reduce code duplication.
        match call.name.as_str() {
            "add" => {
                let val = self.builder.build_int_add(
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                Some(val.as_basic_value_enum())
            }
            "sub" => {
                let val = self.builder.build_int_sub(
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                Some(val.as_basic_value_enum())
            }
            "mul" => {
                let val = self.builder.build_int_mul(
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                Some(val.as_basic_value_enum())
            }
            "div" => {
                let val = self.builder.build_int_unsigned_div(
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                Some(val.as_basic_value_enum())
            }
            "sdiv" => {
                let val = self.builder.build_int_signed_div(
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                Some(val.as_basic_value_enum())
            }
            "mod" => {
                let val = self.builder.build_int_unsigned_rem(
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                Some(val.as_basic_value_enum())
            }
            "smod" => {
                let val = self.builder.build_int_signed_rem(
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                Some(val.as_basic_value_enum())
            }
            "lt" => {
                let val = self.builder.build_int_compare(
                    IntPredicate::ULT,
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                let val =
                    self.builder
                        .build_int_cast(val, self.context.custom_width_int_type(256), "");
                Some(val.as_basic_value_enum())
            }
            "slt" => {
                let val = self.builder.build_int_compare(
                    IntPredicate::SLT,
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                let val =
                    self.builder
                        .build_int_cast(val, self.context.custom_width_int_type(256), "");
                Some(val.as_basic_value_enum())
            }
            "gt" => {
                let val = self.builder.build_int_compare(
                    IntPredicate::UGT,
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                let val =
                    self.builder
                        .build_int_cast(val, self.context.custom_width_int_type(256), "");
                Some(val.as_basic_value_enum())
            }
            "sgt" => {
                let val = self.builder.build_int_compare(
                    IntPredicate::SGT,
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                let val =
                    self.builder
                        .build_int_cast(val, self.context.custom_width_int_type(256), "");
                Some(val.as_basic_value_enum())
            }
            "eq" => {
                let val = self.builder.build_int_compare(
                    IntPredicate::EQ,
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                let val =
                    self.builder
                        .build_int_cast(val, self.context.custom_width_int_type(256), "");
                Some(val.as_basic_value_enum())
            }
            "and" => {
                let val = self.builder.build_and(
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                Some(val.as_basic_value_enum())
            }
            "or" => {
                let val = self.builder.build_or(
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                Some(val.as_basic_value_enum())
            }
            "xor" => {
                let val = self.builder.build_xor(
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                Some(val.as_basic_value_enum())
            }
            "shl" => {
                let val = self.builder.build_left_shift(
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    "",
                );
                Some(val.as_basic_value_enum())
            }
            "shr" => {
                let val = self.builder.build_right_shift(
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    false,
                    "",
                );
                Some(val.as_basic_value_enum())
            }
            "sar" => {
                let val = self.builder.build_right_shift(
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.translate_expression(&call.arguments[1])
                        .into_int_value(),
                    true,
                    "",
                );
                Some(val.as_basic_value_enum())
            }
            "iszero" => {
                let val = self.builder.build_right_shift(
                    self.translate_expression(&call.arguments[0])
                        .into_int_value(),
                    self.context.custom_width_int_type(256).const_int(0, false),
                    true,
                    "",
                );
                let val =
                    self.builder
                        .build_int_cast(val, self.context.custom_width_int_type(256), "");
                Some(val.as_basic_value_enum())
            }
            //TODO: implement once we support it
            "revert" => Some(
                self.context
                    .custom_width_int_type(256)
                    .const_int(0, false)
                    .as_basic_value_enum(),
            ),
            "mstore" => Some(
                self.context
                    .custom_width_int_type(256)
                    .const_int(0, false)
                    .as_basic_value_enum(),
            ),
            "mload" => Some(
                self.context
                    .custom_width_int_type(256)
                    .const_int(0, false)
                    .as_basic_value_enum(),
            ),
            "calldataload" => Some(
                self.context
                    .custom_width_int_type(256)
                    .const_int(0, false)
                    .as_basic_value_enum(),
            ),
            _ => None,
        }
    }

    fn translate_literal(&self, lit: &Literal) -> BasicValueEnum {
        let i256_ty = self.context.custom_width_int_type(256);
        let decimal = Regex::new("^[0-9]+$").unwrap();
        let hex = Regex::new("^0x[0-9a-fA-F]+$").unwrap();
        let lit = lit.value.as_str();
        if decimal.is_match(lit) {
            i256_ty
                .const_int_from_string(lit, StringRadix::Decimal)
                .unwrap()
                .as_basic_value_enum()
        } else if hex.is_match(lit) {
            i256_ty
                .const_int_from_string(&lit[2..], StringRadix::Hexadecimal)
                .unwrap()
                .as_basic_value_enum()
        } else {
            unreachable!();
        }
    }

    fn translate_expression(&self, e: &Expression) -> BasicValueEnum {
        match e {
            Expression::Literal(l) => self.translate_literal(&l),
            Expression::Identifier(var) => self.builder.build_load(self.variables[&var.name], ""),
            Expression::FunctionCall(call) => match self.translate_builtin(call) {
                Some(expr) => expr,
                None => {
                    let args: Vec<BasicValueEnum> = call
                        .arguments
                        .iter()
                        .map(|arg| self.translate_expression(&arg))
                        .collect();
                    let function = self
                        .module
                        .get_function(call.name.as_str())
                        .unwrap_or_else(|| panic!("Undeclared function {}", call.name));
                    let ret = self
                        .builder
                        .build_call(function, &args, "")
                        .try_as_basic_value();
                    if ret.is_left() {
                        ret.expect_left("Unexpected call")
                    } else {
                        // Void function call. Just return any value for consistensy
                        self.context
                            .custom_width_int_type(256)
                            .const_int(0, false)
                            .as_basic_value_enum()
                    }
                }
            },
        }
    }

    fn translate_variable_declaration(&mut self, vd: &VariableDeclaration) {
        assert!(!vd.names.is_empty());
        if vd.names.len() > 1 {
            // TODO: implement
            unreachable!();
        }
        for name in &vd.names {
            let val = self
                .builder
                .build_alloca(self.translate_type(&name.yul_type), name.name.as_str());
            self.variables.insert(name.name.clone(), val);
        }

        if let Some(init) = &vd.initializer {
            self.builder.build_store(
                self.variables[&vd.names[0].name],
                self.translate_expression(&init),
            );
        };
    }

    fn translate_if_statement(&mut self, ifstmt: &IfConditional) {
        let cond = self.builder.build_int_cast(
            self.translate_expression(&ifstmt.condition)
                .into_int_value(),
            self.context.bool_type(),
            "",
        );
        let then = self
            .context
            .append_basic_block(self.function.unwrap(), "if.then");
        let join = self
            .context
            .append_basic_block(self.function.unwrap(), "if.join");
        self.builder.build_conditional_branch(cond, then, join);
        self.builder.position_at_end(then);
        self.translate_function_body(&ifstmt.body);
        self.builder.build_unconditional_branch(join);
        self.builder.position_at_end(join);
    }

    fn translate_switch_statement(&mut self, switchstmt: &Switch) {
        let default = self
            .context
            .append_basic_block(self.function.unwrap(), "switch.default");
        let join = self
            .context
            .append_basic_block(self.function.unwrap(), "switch.join");
        let cases: Vec<(IntValue<'ctx>, BasicBlock<'ctx>)> = switchstmt
            .cases
            .iter()
            .map(|case| {
                let lit = self
                    .context
                    .custom_width_int_type(256)
                    .const_int_from_string(case.label.value.as_str(), StringRadix::Decimal)
                    .unwrap();
                let bb = self
                    .context
                    .append_basic_block(self.function.unwrap(), "switch.case");
                (lit, bb)
            })
            .collect();
        self.builder.build_switch(
            self.translate_expression(&switchstmt.expression)
                .into_int_value(),
            default,
            &cases,
        );
        for (idx, case) in cases.iter().enumerate() {
            self.builder.position_at_end(case.1);
            self.translate_function_body(&switchstmt.cases[idx].body);
            self.builder.build_unconditional_branch(join);
        }
        self.builder.position_at_end(default);
        match &switchstmt.default {
            Some(bb) => self.translate_function_body(&bb),
            None => (),
        }
        self.builder.build_unconditional_branch(join);
        self.builder.position_at_end(join);
    }

    fn translate_for_loop(&mut self, forloop: &ForLoop) {
        self.translate_function_body(&forloop.initializer);
        let cond = self
            .context
            .append_basic_block(self.function.unwrap(), "for.cond");
        let body = self
            .context
            .append_basic_block(self.function.unwrap(), "for.body");
        let inc = self
            .context
            .append_basic_block(self.function.unwrap(), "for.inc");
        let exit = self
            .context
            .append_basic_block(self.function.unwrap(), "for.exit");
        self.builder.build_unconditional_branch(cond);
        self.builder.position_at_end(cond);
        let condition = self.builder.build_int_cast(
            self.translate_expression(&forloop.condition)
                .into_int_value(),
            self.context.bool_type(),
            "",
        );
        self.builder.build_conditional_branch(condition, body, exit);
        self.break_bb = Some(exit);
        self.continue_bb = Some(inc);
        self.builder.position_at_end(body);
        self.translate_function_body(&forloop.body);
        self.builder.build_unconditional_branch(inc);
        self.builder.position_at_end(inc);
        self.translate_function_body(&forloop.finalizer);
        self.builder.build_unconditional_branch(cond);
        self.break_bb = None;
        self.continue_bb = None;
        self.builder.position_at_end(exit);
    }

    fn translate_function_body(&mut self, body: &Block) {
        for stmt in &body.statements {
            match stmt {
                // The scope can be cleaned up on exit, but let's LLVM do the job. We can also rely
                // on YUL renaming so we don't need to track scope.
                Statement::Block(b) => self.translate_function_body(&b),
                Statement::VariableDeclaration(vd) => self.translate_variable_declaration(&vd),
                // TODO: support tuples
                Statement::Assignment(var) => {
                    self.builder.build_store(
                        self.variables[&var.names[0].name],
                        self.translate_expression(&var.initializer),
                    );
                }
                Statement::Expression(expr) => {
                    self.translate_expression(&expr);
                }
                Statement::IfConditional(ifstmt) => self.translate_if_statement(&ifstmt),
                Statement::Switch(switchstmt) => self.translate_switch_statement(&switchstmt),
                Statement::ForLoop(forloop) => self.translate_for_loop(&forloop),
                Statement::Leave => {
                    self.builder
                        .build_unconditional_branch(self.leave_bb.unwrap());
                }
                Statement::Break => {
                    self.builder
                        .build_unconditional_branch(self.break_bb.unwrap());
                }
                Statement::Continue => {
                    self.builder
                        .build_unconditional_branch(self.continue_bb.unwrap());
                }
                _ => unreachable!(),
            }
        }
    }

    fn create_function(&mut self, name: &str, fn_t: FunctionType<'ctx>) -> FunctionValue<'ctx> {
        let function = self.module.add_function(name, fn_t, None);
        self.functions.insert(name.to_string(), function);
        function
    }

    fn translate_function_definition(&mut self, fd: &FunctionDefinition) {
        let par_types: Vec<_> = fd
            .parameters
            .iter()
            .map(|par| self.translate_type(&par.yul_type))
            .collect();
        let name = fd.name.as_str();
        let function = self.module.get_function(name).unwrap();
        let ret_ty = function.get_type().get_return_type();
        self.function = Some(function);
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        for (idx, param) in fd.parameters.iter().enumerate() {
            let par = self
                .builder
                .build_alloca(par_types[idx], param.name.as_str());
            self.variables.insert(param.name.clone(), par);
            self.builder
                .build_store(par, function.get_nth_param(idx as u32).unwrap());
        }

        let ret_ptr = match ret_ty {
            None => None,
            Some(t) => Some(self.builder.build_alloca(t, "result")),
        };

        let ret_values: Vec<_> = fd
            .result
            .iter()
            .map(|v| {
                let val = self
                    .builder
                    .build_alloca(self.translate_type(&v.yul_type), v.name.as_str());
                self.variables.insert(v.name.clone(), val);
                val
            })
            .collect();

        let exit = self.context.append_basic_block(function, "exit");
        self.leave_bb = Some(exit);

        self.translate_function_body(&fd.body);

        let last_instr = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_last_instruction();

        match last_instr {
            None => {
                self.builder.build_unconditional_branch(exit);
            }
            Some(i) => match i.get_opcode() {
                InstructionOpcode::Br => (),
                InstructionOpcode::Switch => (),
                _ => {
                    self.builder.build_unconditional_branch(exit);
                }
            },
        };

        self.builder.position_at_end(exit);

        match ret_ptr {
            None => self.builder.build_return(None),
            Some(ret_ptr) => {
                if ret_values.len() == 1 {
                    self.builder
                        .build_return(Some(&self.builder.build_load(ret_values[0], "")))
                } else {
                    for (idx, val) in ret_values.iter().enumerate() {
                        self.builder.build_store(
                            self.builder
                                .build_struct_gep(ret_ptr, idx as u32, "")
                                .unwrap(),
                            self.builder.build_load(*val, ""),
                        );
                    }
                    let ret = self.builder.build_load(ret_ptr, "");
                    self.builder.build_return(Some(&ret))
                }
            }
        };
    }

    fn translate_function_declaration(&mut self, fd: &FunctionDefinition) {
        let name = fd.name.as_str();
        let par_types: Vec<_> = fd
            .parameters
            .iter()
            .map(|par| BasicTypeEnum::IntType(self.translate_type(&par.yul_type)))
            .collect();
        let fn_t = self.translate_fn_type(&fd.result, &par_types);
        self.create_function(name, fn_t);
    }

    fn translate_module(&mut self, block: &Block) {
        for stmt in &block.statements {
            match stmt {
                Statement::FunctionDefinition(fd) => self.translate_function_declaration(fd),
                Statement::VariableDeclaration(_) => unreachable!(),
                _ => unreachable!(),
            }
        }
        for stmt in &block.statements {
            match stmt {
                Statement::FunctionDefinition(fd) => self.translate_function_definition(fd),
                Statement::VariableDeclaration(_) => unreachable!(),
                _ => unreachable!(),
            }
        }
    }

    pub fn compile(statement: &Statement, run: &Option<&str>) -> u64 {
        let context = Context::create();
        let module = context.create_module("module");
        let builder = context.create_builder();

        let mut compiler = Compiler {
            context: &context,
            builder: &builder,
            module: &module,
            function: None,
            leave_bb: None,
            break_bb: None,
            continue_bb: None,
            variables: HashMap::new(),
            functions: HashMap::new(),
        };

        match statement {
            Statement::Block(block) => {
                compiler.translate_module(&block);
            }
            _ => unreachable!(),
        }
        println!("{}", module.print_to_string().to_string());
        match run {
            Some(name) => {
                let execution_engine = module.create_interpreter_execution_engine().unwrap();
                let entry = module.get_function(name).unwrap();
                let result = unsafe { execution_engine.run_function(entry, &[]) }.as_int(false);
                println!("Result: {:?}", result);
                result
            }
            None => 0,
        }
    }
}
