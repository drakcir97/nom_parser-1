#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unreachable_code)]
#![allow(non_camel_case)]

use crate::enums::*;

extern crate inkwell;

use self::inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::{ExecutionEngine, JitFunction},
    module::Module,
    types::BasicTypeEnum,
    values::{BasicValueEnum, FunctionValue, InstructionValue, IntValue, PointerValue},
    IntPredicate, OptimizationLevel,
};

use core::panic;
use std::{collections::HashMap, error::Error};

type ExprFunc = unsafe extern "C" fn() -> i32;

//-----------------------------------------------------------------------------------------------

use std::process;
use std::result;
use std::str;

use std::hash;

use crate::enums::*;

use crate::enums::List::{func, var, Cons, Num};
use crate::enums::op::{add, div, mult, res, sub, wrong};
use crate::enums::variable_value::{boxs, Boolean, Nil, Number};
use crate::enums::variable::{name, parameters};

// LLVM ENUMS -----------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum llvmhashstate<'ctx> {
    state(
        Box<llvmfunctionstate<'ctx>>,
        Box<Vec<llvmhashvariable>>,
        Box<function>,
        i32,
    ),
    Nil,
}

//First one is variable name, last one is the address in addressmap and idmap
#[derive(Debug, Clone, PartialEq)]
pub enum llvmhashvariable {
    var(String, i32),
    Nil,
}

//The data located in addressmap. Value refers to a real value, address just points to another address.
#[derive(Debug, Clone, PartialEq)]
pub enum llvmhashdata<'ctx> {
    value(IntValue<'ctx>),
}

//The different states a function can be in.
#[derive(Debug, Clone, PartialEq)]
pub enum llvmfunctionstate<'ctx> {
    Running,
    Stopped,
    Declared,
    Looping,
    Calling,
    Returned(Box<llvmhashdata<'ctx>>),
}

fn unbox<T>(value: Box<T>) -> T {
    *value
}

struct CodeGen<'a, 'ctx> {
    context: &'ctx Context,
    builder: &'a Builder<'ctx>,
    module: &'a Module<'ctx>,
    execution_engine: &'a ExecutionEngine<'ctx>,
    fn_value_opt: Option<FunctionValue<'ctx>>,
    //Mem should be sorted on function name and contain a second hashmap that actually stores the variables under name with the pointer. 2021-04-25

    var: HashMap<String, PointerValue<'ctx>>,

    state: HashMap<String, llvmhashstate<'ctx>>,

    //mut currentid: i32 = 1,
}

pub fn execute(pg: Program) {
    let (nm, statements) = match pg {
        Program::pgr(v,w) => (v,w),
    };

    let context = Context::create();
    let module = context.create_module("llvm-program");
    let builder = context.create_builder();
    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    
    let mut codegen = CodeGen {
        context: &context,
        builder: &builder,
        module: &module,
        execution_engine: &execution_engine,
        var: HashMap::new(),
        fn_value_opt: None,
        state: HashMap::new(),
    };

    //let mut state: HashMap<String, llvmhashstate> = HashMap::new();

    //let mut mem: HashMap<String, HashMap<String, PointerValue>> = HashMap::new();

    let mut dec_iter = statements.iter();

    for stmt in dec_iter {
        codegen.functionDeclare(stmt.clone()); //Loop through and declare all functions into state.
    }

    let mut iter = statements.iter(); 

    for stmt in iter { //Run program
        codegen.compile_list(unbox(nm.clone()),stmt.clone());
    }
    //assert!(iter.is_ok());
}

//Memory changed to simplified version, next thing is change function calls and declares so that they use inkwell instead of our own architecture.
//  - R 2021-06-15

impl<'a, 'ctx> CodeGen<'a, 'ctx> {
    fn fn_value(&self) -> FunctionValue<'ctx> {
        self.fn_value_opt.unwrap()
    }
    fn cast_int(&self, int: i32) -> IntValue<'ctx> {
        self.context.i32_type().const_int(int as u64, false)
    }

    fn cast_bool(&self, b: bool) -> IntValue<'ctx> {
        match b {
            true => self.context.bool_type().const_int(1, false),
            false => self.context.bool_type().const_int(0, false),
        }
    }

    fn get_var(&self, na: String) -> &PointerValue<'ctx> {
        match self.var.get(&na) {
            Some(v) => return v,
            None => panic!("Not found in memory: get_var"),
        };
    }

    fn getState(&self, function: String) -> &llvmhashstate {
        if self.state.contains_key(&function) {
            let result = self.state.get(&function);
            match result {
                Some(val) => return val,
                None => panic!("Get state failed!: getState"),
            }
        } else {
            panic!("No such state exists!: getState");
        }
    }

    fn changeState(&mut self, function: String, st: llvmhashstate<'ctx>) {
        self.state.insert(function,st); //Adds state if it does not exists, updates value 'st' if it does.
    }

    fn insert_var(&mut self, functionname: String, na: String, pa: PointerValue<'ctx>) {
        self.var.insert(na,pa);
    }

    fn functionDeclare(&mut self, ls: List) {
        match ls {
            List::func(f) => {
                match f.clone() {
                    function::parameters_def(na,ar,ty,ele) => {
                        let temp: Vec<llvmhashvariable> = Vec::new();
                        let tempstring: String = unbox(na.clone());
                        self.changeState(tempstring.clone(),llvmhashstate::state(Box::new(llvmfunctionstate::Declared),Box::new(temp),Box::new(f.clone()),-1));
                    },
                    _ => (),
                };
            },
            _ => (),// Do nothing
        }
    }


    fn compile_list(&mut self, functionname: String, ls: List) -> IntValue<'ctx> {
        match ls{
            List::paran(v) => {return self.compile_list(functionname.clone(), unbox(v))},
            List::Cons(v,w,x) => {return self.compile_cons(functionname.clone(),v,w,x)},
            List::Num(v) => {return self.cast_int(v)},
            List::boolean(v)=>{return self.cast_bool(v)},
            List::func(fu) => {return self.compile_function_call(Box::new(functionname.clone()),fu)},
            List::var(v) => {return self.fetch_var(Box::new(functionname.clone()), v)},
            _ => panic!("Something went wrong: execute_List"),
        };
    }

    //TODO
    fn allocate_pointer(&mut self, functionname: String, na: String, is_bool: bool) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        //let entry = self.fn_value().get_first_basic_block().unwrap();
        //match entry.get_first_instruction() {
        //    Some(f_ins) => builder.position_before(&f_ins),
        //    None => builder.position_at_end(entry),
        //}
        let pa: PointerValue;

        if is_bool {
            pa = builder.build_alloca(self.context.bool_type(), &na);
        } else {
            pa = builder.build_alloca(self.context.i32_type(), &na);
        }

        //self.insert_var(functionname.clone(),na.clone(),pa);
        return pa;
    }

    //CHange return statements to use builder istead, not needed to chech for logic ourselves.
    fn compile_cons(&mut self, functionname: String, l: Box<List>, oper: op ,r:  Box<List>) -> IntValue<'ctx> {
        let expr = match oper{
            op::add => {
                let vall = self.compile_list(functionname.clone(), unbox(l));
                let valr = self.compile_list(functionname.clone(), unbox(r));
                return self.builder.build_int_add(vall,valr,"add");
            },
    
            op::sub => {
                let vall = self.compile_list(functionname.clone(), unbox(l));
                let valr = self.compile_list(functionname.clone(), unbox(r));
                return self.builder.build_int_sub(vall, valr, "sub");
            },
    
            op::div => {
                let vall = self.compile_list(functionname.clone(), unbox(l));
                let valr = self.compile_list(functionname.clone(), unbox(r));
                return self.builder.build_int_unsigned_div(vall,valr,"div");
            },
    
            op::res => {
                let vall= self.compile_list(functionname.clone(), unbox(l));
                let valr = self.compile_list(functionname.clone(), unbox(r));
                return self.builder.build_int_add(vall,valr,"NOT IMPLEMENTED");
            },
            op::mult => {
                let vall = self.compile_list(functionname.clone(), unbox(l));
                let valr = self.compile_list(functionname.clone(), unbox(r));
                return self.builder.build_int_mul(vall,valr,"mul");
            },
            op::less => {
                let vall = self.compile_list(functionname.clone(), unbox(l));
                let valr = self.compile_list(functionname.clone(), unbox(r));
                return self.builder.build_int_compare(IntPredicate::ULT,vall,valr,"Lesser than");
            },
            op::greater => {
                let vall = self.compile_list(functionname.clone(), unbox(l));
                let valr = self.compile_list(functionname.clone(), unbox(r));
                return self.builder.build_int_compare(IntPredicate::UGT, vall, valr, "Greater than")
            },
            op::lessEqual => {
                let vall = self.compile_list(functionname.clone(), unbox(l));
                let valr = self.compile_list(functionname.clone(), unbox(r));
                return self.builder.build_int_compare(IntPredicate::ULE,vall,valr,"Lesser or equal");
            },
            op::greatEqual => {
                let vall = self.compile_list(functionname.clone(), unbox(l));
                let valr = self.compile_list(functionname.clone(), unbox(r));
                return self.builder.build_int_compare(IntPredicate::UGE,vall,valr,"Greater or equal");
            },
            op::and => {
                let vall = self.compile_list(functionname.clone(), unbox(l));
                let valr = self.compile_list(functionname.clone(), unbox(r));
                return self.builder.build_and(vall, valr, "and")
            },
            op::or => {
                let vall = self.compile_list(functionname.clone(), unbox(l));
                let valr = self.compile_list(functionname.clone(), unbox(r));
                return self.builder.build_or(vall, valr, "or");
            },
            
    
            _ => panic!("Operand not supported: compile_cons")
        };
        return self.cast_int(0)
    }

    //TODO update it for llvm, supposed to execute function without using caller function.
    fn compile_function(&mut self,functionname: String, func_var: function) -> InstructionValue<'ctx>{
        let (na,args) = match func_var{
            function::parameters_def(n,m,_n,_o)=>{
                return self.builder.build_unreachable();
            }, //Do nothing on define, since this is handled in functionDeclare, except for main.
            function::parameters_call(v,w)=>{
                (v,w)
            },
        };

        //Fetch function arguments and call fetch_func_types_dec

        let st = self.getState(unbox(na.clone()));

        let fu_st = match st {
            llvmhashstate::state(_fust,_va,fu,_i) => fu,
            _ => panic!("Function not declared: compile_function")
        };

        let (fu_st_args,fu_st_ele) = match unbox(fu_st.clone()) {
            function::parameters_def(_na,args,_ty,ele) => (args,ele),
            _ => panic!("Something went wrong with declaring: compile_function")
        };

        let par_types = self.fetch_func_types_dec(unbox(na.clone()),unbox(fu_st_args.clone()));
        let fn_type = self.context.i32_type().fn_type(&par_types, false);
        let function = self.module.add_function(&na, fn_type, None);
        let basic_block = self.context.append_basic_block(function, &na);

        self.fn_value_opt = Some(function);
        self.builder.position_at_end(basic_block);

        //Here the input to the function should be handled, check for correct number of inputs
        //Then execute the function

        //Fetch function from state, call new function_arguments_call_declare to sort out inputs to func

        self.function_arguments_call_declare(na.clone(),args,fu_st_args.clone());
        let result = self.compile_function_elements(na.clone(),unbox(fu_st_ele));
        return result.0;
    }

    //Declares variables sent to a function through call and adds them to memeory. Handles nestled function calls.
    //Takes function args to ensure that names and order is correct when declaring variables.
    fn function_arguments_call_declare(&mut self, functionname: Box<String>, args: Box<function_arguments_call>, fuargs: Box<function_arguments>) {
        if unbox(functionname.clone()) == "main" { //If we are in main, declare no variables.
            return;
        }
        let temp: function_arguments_call = unbox(args.clone());
        match temp {
            function_arguments_call::arg_call_list(a1,a2) => {
                let (varl, far) = match unbox(fuargs) {
                    function_arguments::arg_list(le,ri) => {(le,ri)},
                    _ => return panic!("Too many inputs given to function {:?} : function_arguments_call_declare",functionname.clone()),
                };
                let fal = Box::new(function_arguments::var(varl));
                let _leftSide = self.function_arguments_call_declare(functionname.clone(),a1,fal);
                let _rightSide = self.function_arguments_call_declare(functionname.clone(),a2,far);
            },
            function_arguments_call::bx(bo) => {
                let functionargs = match unbox(fuargs.clone()) {
                    function_arguments::arg_list(le,ri) => {return panic!("jada")},
                    function_arguments::var(v) => {v},
                };
                let (varname,vartype) = match functionargs {
                    variable::parameters(n,t,_v) => {(n,t)},
                    variable::name(n) => {(n,Type::unknown(0))},
                };
                let unb = unbox(bo);
                match unb {
                    List::var(v) => {
                        let newargs = Box::new(function_arguments_call::variable(Box::new(v)));
                        self.function_arguments_call_declare(functionname.clone(), newargs, fuargs.clone());
                    },
                    List::boolean(b) => {
                        if vartype != Type::boolean {
                            panic!("Type mismatch in var: function_arguments_call_declare")
                        };
                        let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false);
                        let b = self.cast_bool(b);
                        self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone());
                        self.builder.build_store(ptr.clone(),b);

                        let ptr_val = self.get_var(unbox(varname.clone()));
                        self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
                    },
                    List::Num(n) => {
                        if vartype != Type::Integer {
                            panic!("Type mismatch in var: function_arguments_call_declare")
                        };
                        let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false);
                        let n = self.cast_int(n);
                        self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone());
                        self.builder.build_store(ptr.clone(),n);

                        let ptr_val = self.get_var(unbox(varname.clone()));
                        self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
                    },
                    _ => (),
                };
            },
            function_arguments_call::function(fu) => {
                let functionargs = match unbox(fuargs.clone()) {
                    function_arguments::arg_list(le,ri) => {panic!("Too few inputs given to function {:?} : function_arguments_call_declare",functionname.clone())},
                    function_arguments::var(v) => {v},
                };
                let (varname,vartype) = match functionargs {
                    variable::parameters(n,t,_v) => {(n,t)},
                    variable::name(n) => {(n,Type::unknown(0))},
                };
                let val = self.compile_function_call(functionname.clone(),unbox(fu));

                let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false);
                self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone());
                self.builder.build_store(ptr.clone(),val);

                let ptr_val = self.get_var(unbox(varname.clone()));
                self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
            },
            function_arguments_call::variable(va) => {
                let functionargs = match unbox(fuargs.clone()) {
                    function_arguments::arg_list(le,ri) => {panic!("Too many inputs given to function {:?} : function_arguments_call_declare",functionname.clone())},
                    function_arguments::var(v) => {v},
                };
                let (varname,vartype) = match functionargs {
                    variable::parameters(n,t,_v) => {(n,t)},
                    variable::name(n) => {(n,Type::unknown(0))},
                };
                match unbox(va.clone()) {
                    variable::parameters(na,_ty,val) => {
                        match unbox(val) {
                            variable_value::Boolean(b) => {
                                if vartype != Type::boolean {
                                    panic!("Type mismatch in var: function_arguments_call_declare")
                                };
                                let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false);
                                let b = self.cast_bool(b);
                                self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),b);

                                let ptr_val = self.get_var(unbox(varname.clone()));
                                self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
                            },
                            variable_value::Number(n) => {
                                if vartype != Type::Integer {
                                    panic!("Type mismatch in var: function_arguments_call_declare")
                                };
                                let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false);
                                let n = self.cast_int(n);
                                self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),n);

                                let ptr_val = self.get_var(unbox(varname.clone()));
                                self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
                            },
                            _ => panic!("temp"), //Might have to include more cases for variable_value here if needed.
                        };
                    },
                    variable::name(n) => {
                        let vnam = unbox(n);
                        let ptr_val = self.get_var(vnam.clone());
                        self.builder.build_load(*ptr_val, &vnam.clone()).into_int_value();
                    },
                    _ => panic!("Type not yet supported: function_arguments_call_declare"),
                };
            },
        }
    }

    fn fetch_func_types_dec(&mut self, functionname: String, args: function_arguments) -> Vec<BasicTypeEnum<'ctx>> {
        match args {
            function_arguments::arg_list(va, re) => {
                let ty = self.fetch_var_type(functionname.clone(),va);
                let mut val: Vec<BasicTypeEnum> = match ty {
                    Type::Integer => vec![self.context.i32_type().into()],
                    Type::boolean => vec![self.context.bool_type().into()],
                    _ => panic!("Incorrect type: fetch_func_types_dec"),
                };
                //let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
                let rest = self.fetch_func_types_dec(functionname.clone(),unbox(re));
                val.extend(rest);
                return val;
            },
            function_arguments::var(va) => {
                let ty = self.fetch_var_type(functionname.clone(),va);
                let val: Vec<BasicTypeEnum> = match ty {
                    Type::Integer => vec![self.context.i32_type().into()],
                    Type::boolean => vec![self.context.bool_type().into()],
                    _ => panic!("Incorrect type: fetch_func_types_dec"),
                };
                return val;
            },
            _ => panic!("Not supposed to happen: fetch_func_types")
        };
    }

    // fn fetch_func_types_call(&mut self, functionname: String, args: function_arguments_call) -> Vec<BasicTypeEnum<'ctx>> {
        
    // }

    fn fetch_var_type(&mut self, functionname: String, va: variable) -> Type {
        match va {
            variable::parameters(_na,ty,_val) => {
                return ty;
            },
            _ => {
                return Type::unknown(0);
            },
        };
    }

    //Removed check in memory so now only one match for everything, simplify memory to just include addressmap. Should make fetching easier too.
    fn declare_var(&mut self, functionname: Box<String>, variable_var: variable) -> InstructionValue<'ctx>{
        match variable_var{
            variable::parameters(na,ty,val) => {
                match unbox(val) {
                    variable_value::Boolean(b) => {
                        if ty != Type::boolean {
                            panic!("Type mismatch in var: declare_var")
                        };
                        let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),true);
                        let b = self.cast_bool(b);
                        self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                        self.builder.build_store(ptr.clone(),b)
                    },
                    variable_value::Number(n) => {
                        if ty != Type::Integer {
                            panic!("Type mismatch in var: declare_var")
                        };
                        let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                        let n = self.cast_int(n);
                        self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                        self.builder.build_store(ptr.clone(),n)
                    },
                    variable_value::boxs(b) => {
                        let ls = unbox(b);
                        match ls.clone() {
                            List::Num(n) => {
                                if ty != Type::Integer {
                                    panic!("Type mismatch in var: declare_var")
                                };
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                                let n = self.cast_int(n);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),n)
                            },
                            List::boolean(b) => {
                                if ty != Type::boolean {
                                    panic!("Type mismatch in var: declare_var")
                                };
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),true);
                                let b = self.cast_bool(b);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),b)
                            },
                            List::var(v) => {
                                let varval = self.fetch_var(functionname.clone(),v);
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),varval)
                            },
                            List::Cons(lli,op,rli) => {
                                let val = self.compile_cons(*functionname.clone(), lli, op, rli);
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),val)
                            },
                            List::func(fu) => {
                                let val = self.compile_list(unbox(functionname.clone()), ls.clone());
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),val)
                            },
                            _ => panic!("Incorrect type: declare_var"),
                        }
                    },
                    _ => panic!("Something failed: declare_var"),
                };
                panic!("Incorrect type: declare_var");
            },
            variable::name(v)=>{ // Access local var with same name and return it.
                return self.builder.build_unreachable();
            },
        };
    }

    fn fetch_var(&mut self, functionname: Box<String>, variable_var: variable) -> IntValue<'ctx>{
        match variable_var{
            variable::parameters(na,ty,val) => {
                match unbox(val) {
                    variable_value::Boolean(b) => {
                        if ty != Type::boolean {
                            panic!("Type mismatch in var: declare_var")
                        };
                        let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),true);
                        let b = self.cast_bool(b);
                        self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                        self.builder.build_store(ptr.clone(),b);
                        b
                    },
                    variable_value::Number(n) => {
                        if ty != Type::Integer {
                            panic!("Type mismatch in var: declare_var")
                        };
                        let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                        let n = self.cast_int(n);
                        self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                        self.builder.build_store(ptr.clone(),n);
                        n
                    },
                    variable_value::boxs(b) => {
                        let ls = unbox(b);
                        match ls.clone() {
                            List::Num(n) => {
                                if ty != Type::Integer {
                                    panic!("Type mismatch in var: declare_var")
                                };
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                                let n = self.cast_int(n);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),n);
                                n
                            },
                            List::boolean(b) => {
                                if ty != Type::boolean {
                                    panic!("Type mismatch in var: declare_var")
                                };
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),true);
                                let b = self.cast_bool(b);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),b);
                                b
                            },
                            List::var(v) => {
                                let varval = self.fetch_var(functionname.clone(),v);
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),varval);
                                varval
                            },
                            List::Cons(lli,op,rli) => {
                                let val = self.compile_cons(*functionname.clone(), lli, op, rli);
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),val);
                                val
                            },
                            List::func(fu) => {
                                let val = self.compile_list(unbox(functionname.clone()), ls.clone());
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),val);
                                val
                            },
                            _ => return self.cast_int(0),
                        }
                    },
                    _ => return self.cast_int(0),
                };
                return self.cast_int(0);
            },
            variable::name(v)=>{ // Access local var with same name and return it.
                let ptr = self.get_var(*v.clone());
                return self.builder.build_load(*ptr,&v.clone()).into_int_value();
            },
        };
    }

    fn compile_if(&mut self, functionname: Box<String>, if_e: if_enum) -> InstructionValue<'ctx> {
        let (ifst, if_body) = match if_e{
            if_enum::condition(v,w)=>(v,w)
        };
        // let temp = self.compile_cons(unbox(functionname.clone()), ifst, op::greater, Box::new(List::Num(0)));
        // if (temp == (self.cast_int(1))) {
        //     return self.compile_function_elements(functionname.clone(), unbox(if_body)).0;
        // }
        let cond = self.compile_cons(unbox(functionname.clone()), ifst, op::greater, Box::new(List::Num(0)));

        let then_block = self.context.append_basic_block(self.fn_value(), "then");
        let cont_block = self.context.append_basic_block(self.fn_value(), "cont");

        self.builder
            .build_conditional_branch(cond, then_block, cont_block);
        self.builder.position_at_end(then_block);
        self.compile_function_elements(functionname.clone(), unbox(if_body));

        self.builder.build_unconditional_branch(cont_block);
        self.builder.position_at_end(cont_block);

        let phi = self.builder.build_phi(self.context.i32_type(), "if");
        phi.add_incoming(&[
            (&self.cast_int(1), then_block),
            (&self.cast_int(0), cont_block),
        ]);
        phi.as_instruction()
    }

    fn compile_while(&mut self, functionname: Box<String>, while_e: while_enum) -> InstructionValue<'ctx> {
        let (while_statement, while_body) =  match while_e{
            while_enum::condition(v,w)=>(v,w),
        };
        // let temp = self.compile_cons(unbox(functionname.clone()), while_statement.clone(), op::greater, Box::new(List::Num(0)));
        // result: (InstructionValue<'ctx>,bool);
        // while (temp == (self.cast_int(1))) {
        //     result = self.compile_function_elements(functionname.clone(), unbox(while_body.clone())).0;
        // }
        // return result.0;

        let do_block = self.context.append_basic_block(self.fn_value(), "do");
        let cont_block = self.context.append_basic_block(self.fn_value(), "cont");

        let cond = self.compile_cons(unbox(functionname.clone()), while_statement.clone(), op::greater, Box::new(List::Num(0)));

        self.builder
            .build_conditional_branch(cond, do_block, cont_block);
        self.builder.position_at_end(do_block);
        self.compile_function_elements(functionname.clone(), unbox(while_body.clone()));

        self.builder
            .build_conditional_branch(cond, do_block, cont_block);
        self.builder.position_at_end(cont_block);

        let phi = self.builder.build_phi(self.context.i32_type(), "while");
        phi.add_incoming(&[
            (&self.cast_int(0), do_block),
            (&self.cast_int(1), do_block),
        ]);
        phi.as_instruction()
    }

    //Used to return  -> (InstructionValue<'ctx>, bool)
    //Use self.builder.build_return(Some(&var)) to return only value, no need to memory and local vars and state. Should be a lot smaller after everything is removed.
    fn compile_return(&mut self, functionname: Box<String>, var_val: variable_value) -> InstructionValue<'ctx> {
        match var_val {
            variable_value::Boolean(b) => {
                let b = self.cast_bool(b);
                return self.builder.build_return(Some(&b));
            },
            variable_value::Number(n) => {
                let n = self.cast_int(n);
                return self.builder.build_return(Some(&n));
            },
            variable_value::variable(v) => {
                match unbox(v) {
                    variable::name(n) => {
                        let varname = unbox(n);
                        let ptr = self.get_var(varname.clone());
                        let val = self.builder.build_load(*ptr,&varname.clone()).into_int_value();
                        return self.builder.build_return(Some(&val));
                    },
                    _ => panic!("Return does not support this type: return_execute"),
                };
            },
            variable_value::boxs(va) => {
                match unbox(va) {
                    List::var(v) => {
                        let varval = variable_value::variable(Box::new(v));
                        return self.compile_return(functionname.clone(), varval);
                    },
                    List::Cons(bl, opr, br) => {
                        let consval = self.compile_cons(unbox(functionname.clone()), bl, opr, br);
                        return self.builder.build_return(Some(&consval));
                    },
                    List::boolean(b) => {
                        let b = self.cast_bool(b);
                        return self.builder.build_return(Some(&b));
                    },
                    List::Num(n) => {
                        let n = self.cast_int(n);
                        return self.builder.build_return(Some(&n));
                    },
                    _ => panic!("Return does not support this type: return_execute"),
                }
            },
            _ => panic!("Return does not support this type: return_execute"),
        }
    }

    fn compile_function_elements(&mut self, functionname: Box<String>, fe: function_elements) -> (InstructionValue<'ctx>, bool){
        match fe {
            function_elements::ele_list(v,w)=>{
                let ele1: function_elements = unbox(v);
                let ele2: function_elements = unbox(w);
                let res1 = self.compile_function_elements(functionname.clone(), ele1);
                if (res1.1) {
                    return res1;
                }
                return self.compile_function_elements(functionname.clone(), ele2);
            },
            function_elements::boxs(v)=>{
                let box_cont: variable = unbox(v);
                (self.declare_var(functionname, box_cont), false)
            },
            function_elements::if_box(v)=>{
                let box_cont= unbox(v);
                (self.compile_if(functionname, box_cont), false)
            },
            function_elements::List(v)=>{
                panic!("Should not happen: compile_function_elements");
                //self.compile_list(unbox(functionname), v);
            },
            function_elements::function(v)=>{
                (self.compile_function(unbox(functionname), v), false)
            },
            function_elements::variable(v)=>{
                (self.declare_var(functionname, v), false)
            },
            function_elements::if_enum(v)=>{
                (self.compile_if(functionname, v), false)
            },
            function_elements::while_enum(v) => {
                (self.compile_while(functionname, v), false)
            },
            function_elements::return_val(v) => {
                return (self.compile_return(functionname,v), true);
            },
        }
    }

    fn compile_function_call(&mut self, functionname: Box<String>, fu: function) -> IntValue<'ctx> {
        let (funame, fargs) = match fu{
            function::parameters_def(na,_m,_n,_o)=>{
                return self.cast_int(0);
            }, //Do nothing on define, since this is handled in functionDeclare, except for main.
            function::parameters_call(na,arg)=>{
                (na,arg)
            },
        };
        let fname = unbox(funame.clone()).to_string();
        let fu = self.module.get_function(&fname).unwrap();
        let args = self.compile_function_arguments_call_declare(funame.clone(), fargs);
        
        let call = self
            .builder
            .build_call(fu, &args, &fname)
            .try_as_basic_value()
            .left()
            .unwrap();
        
        match call {
            value => value.into_int_value(),
        }
    }

    fn compile_function_arguments_call_declare(&mut self, functionname: Box<String>, args: Box<function_arguments_call>) -> Vec<BasicValueEnum<'ctx>> {
        let temp: function_arguments_call = unbox(args.clone());
        match temp {
            function_arguments_call::arg_call_list(a1,a2) => {
                let mut leftSide = self.compile_function_arguments_call_declare(functionname.clone(),a1);
                let mut rightSide = self.compile_function_arguments_call_declare(functionname.clone(),a2);
                leftSide.extend(rightSide);
                return leftSide;
            },
            function_arguments_call::bx(bo) => {
                let unb = unbox(bo);
                match unb {
                    List::var(v) => {
                        let newargs = Box::new(function_arguments_call::variable(Box::new(v)));
                        return self.compile_function_arguments_call_declare(functionname.clone(), newargs);
                    },
                    List::boolean(b) => {
                        let vname = "empty".to_string();
                        let ptr = self.allocate_pointer(*functionname.clone(),vname.clone(),true);
                        let b = self.cast_bool(b);
                        self.builder.build_store(ptr.clone(),b);
                        self.insert_var(*functionname.clone(), vname.clone(), ptr.clone());
                        let ptr_val = self.get_var(vname.clone());
                        let val = self.builder.build_load(*ptr_val, &vname).into_int_value();
                        let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
                        return result;
                    },
                    List::Num(n) => {
                        let vname = "empty".to_string();
                        let ptr = self.allocate_pointer(*functionname.clone(),vname.clone(),false);
                        let n = self.cast_int(n);
                        self.builder.build_store(ptr.clone(),n);
                        self.insert_var(*functionname.clone(), vname.clone(), ptr.clone());
                        let ptr_val = self.get_var(vname.clone());
                        let val = self.builder.build_load(*ptr_val, &vname).into_int_value();
                        let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
                        return result;
                    },
                    _ => panic!("asd"),
                };
            },
            function_arguments_call::function(fu) => {
                let val = self.compile_function_call(functionname.clone(), unbox(fu));
                let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
                return result;
            },
            function_arguments_call::variable(va) => {
                match unbox(va.clone()) {
                    variable::parameters(na,_ty,val) => {
                        match unbox(val) {
                            variable_value::Boolean(b) => {
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),true);
                                let b = self.cast_bool(b);
                                self.builder.build_store(ptr.clone(),b);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                let ptr_val = self.get_var(*na.clone());
                                let val = self.builder.build_load(*ptr_val, &na.clone()).into_int_value();
                                let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
                                return result;
                            },
                            variable_value::Number(n) => {
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                                let n = self.cast_int(n);
                                self.builder.build_store(ptr.clone(),n);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                let ptr_val = self.get_var(*na.clone());
                                let val = self.builder.build_load(*ptr_val, &na.clone()).into_int_value();
                                let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
                                return result;
                            },
                            _ => panic!("temp"), //Might have to include more cases for variable_value here if needed.
                        };
                    },
                    variable::name(n) => {
                        let vnam = unbox(n);
                        let ptr = self.get_var(vnam.clone());
                        let val = self.builder.build_load(*ptr,&vnam.clone()).into_int_value();

                        //Here the variable should be added to the current functions local variables. 2021-04-25
                        self.insert_var(*functionname.clone(), vnam.clone(), ptr.clone());
                        let ptr_val = self.get_var(vnam.clone());
                        let val = self.builder.build_load(*ptr_val, &vnam.clone()).into_int_value();
                        let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
                        return result;
                    },
                    _ => panic!("Type not yet supported: function_arguments_call_declare"),
                };
            },
        }
    }
}


fn main() {}