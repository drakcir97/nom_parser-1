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

    var: HashMap<String, PointerValue<'ctx>>, //Stores pointers for variables.

    state: HashMap<String, llvmhashstate<'ctx>>, //Store state like in interpreter, used for fetching whole function when calling.

    //mut currentid: i32 = 1,
}

pub fn execute(pg: Program)  -> Result<(), Box<dyn Error>> {
    let (nm, statements) = match pg {
        Program::pgr(v,w) => (v,w),
    };

    let context = Context::create();
    let module = context.create_module("llvm-program");
    let builder = context.create_builder();
    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    
    let mut codegen = CodeGen { //Declare structure for LLVM.
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

    // let mut iter = statements.iter(); 

    // for stmt in iter { //Run program
    //     codegen.compile_list(unbox(nm.clone()),stmt.clone());
    // }

    let mut iter = statements.iter(); 

    // for stmt in iter { //Run program
    //     codegen.runMainFunction(stmt.clone()); //Run main function
    // }

    for stmt in iter { //Run main
        match stmt {
            List::func(f) => {
                match f.clone() {
                    function::parameters_def(na,ar,ty,ele) => { //When we find a define, run a "call" to that function. Specifically needed for LLVM since it relies on this to "make" the function for calling later.
                        let fc = buildFunctionCall(f.clone());
                        println!("{:?}",f.clone());
                        println!("{:?}",fc.clone());
                        codegen.compile_function("FirstCall".to_string(),fc);
                    },
                    _ => (),
                };
            },
            _ => (),// Do nothing
        }
    }

    codegen.module.print_to_stderr();
    let compiled_program: JitFunction<ExprFunc> =
        unsafe { codegen.execution_engine.get_function("main").ok().unwrap() };

    unsafe {
        println!("llvm-result: {} ", compiled_program.call());
    }

    Ok(())
    //assert!(iter.is_ok());
}

//Memory changed to simplified version, next thing is change function calls and declares so that they use inkwell instead of our own architecture.
//  - R 2021-06-15

impl<'a, 'ctx> CodeGen<'a, 'ctx> {
    fn fn_value(&self) -> FunctionValue<'ctx> {
        self.fn_value_opt.unwrap()
    }
    
    //Casts a standard rust integer to an IntValue that is used by Inkwell
    fn cast_int(&self, int: i32) -> IntValue<'ctx> {
        self.context.i32_type().const_int(int as u64, false)
    }

    //Casts a boolean to an intvalue thats either 1 for true, or 0 for false
    fn cast_bool(&self, b: bool) -> IntValue<'ctx> {
        match b {
            true => self.context.bool_type().const_int(1, false),
            false => self.context.bool_type().const_int(0, false),
        }
    }

    //Fetches pointer for a variable name.
    fn get_var(&self, na: String) -> &PointerValue<'ctx> {
        //println!("get var {:?}",na.clone());
        match self.var.get(&na) {
            Some(v) => return v,
            None => panic!("Not found in memory: get_var {:?}",na),
        };
    }
    //Fetches an input variable to see wether it already exists in memory or not
    fn try_var(&self, na: String) -> bool {
        match self.var.get(&na) {
            Some(v) => {return true;},
            None => {return false;},
        };
    }

    //Returns state for a functionname, used for fetching other enum of the function when calling.
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

    //Adds state if it doesnt exist, updates value 'st' if it does.
    fn changeState(&mut self, function: String, st: llvmhashstate<'ctx>) {
        self.state.insert(function,st); 
    }

    fn insert_var(&mut self, functionname: String, na: String, pa: PointerValue<'ctx>) {
        self.var.insert(na,pa);
    }

    //Declares a function for later execution, also does some preparation for LLVM.
    fn functionDeclare(&mut self, ls: List) {
        match ls {
            List::func(f) => {
                match f.clone() {
                    function::parameters_def(na,ar,ty,ele) => {
                        let temp: Vec<llvmhashvariable> = Vec::new();
                        let tempstring: String = unbox(na.clone());
                        self.changeState(tempstring.clone(),llvmhashstate::state(Box::new(llvmfunctionstate::Declared),Box::new(temp),Box::new(f.clone()),-1));           
                        let par_types = self.fetch_func_types_dec(unbox(na.clone()),unbox(ar.clone())); // LLVM LINES HERE
                        let fn_type = self.context.i32_type().fn_type(&par_types, false);
                        let function = self.module.add_function(&na, fn_type, None);
                        //let basic_block = self.context.append_basic_block(function, &na);
                
                        //self.fn_value_opt = Some(function);
                        //self.builder.position_at_end(basic_block);
                    },
                    _ => (),
                };
            },
            _ => (),// Do nothing
        }
    }

    //Runs the main function declared in main.rs by adding all relevant information such as function name, function arguments and 
    //variables to compile_function
    fn runMainFunction(&mut self, ls: List) {
        match ls {
            List::func(f) => {
                match f.clone() {
                    function::parameters_def(na,ar,ty,ele) => {
                        if (unbox(na.clone()) == "main".to_string()) {
                            //println!("Found main in structure");
                            let fc = function::parameters_call(Box::new("main".to_string()),Box::new(function_arguments_call::variable(Box::new(variable::name(Box::new("test".to_string()))))));
                            self.compile_function("FirstCall".to_string(),fc);
                        }
                    },
                    _ => (),
                };
            },
            _ => (),// Do nothing
        }
    }

    //Compiles List enum for IntValue, used in adding values for declares etc.
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

    //Allocates a pointer for later use.
    fn allocate_pointer(&mut self, functionname: String, na: String, is_bool: bool) -> PointerValue<'ctx> {
        //println!("allocate {:?}",na.clone());
        let builder = self.context.create_builder();
        let entry = self.fn_value().get_first_basic_block().unwrap();
        match entry.get_first_instruction() {
           Some(f_ins) => builder.position_before(&f_ins),
           None => builder.position_at_end(entry),
        }
        let pa: PointerValue;

        if is_bool {
            pa = builder.build_alloca(self.context.bool_type(), &na);
        } else {
            pa = builder.build_alloca(self.context.i32_type(), &na);
        }

        //self.insert_var(functionname.clone(),na.clone(),pa);
        return pa;
    }

    //Adds together values based on operand, returns IntValue so is used to combine values at declares etc.
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
            op::equal => {
                let vall = self.compile_list(functionname.clone(), unbox(l));
                let valr = self.compile_list(functionname.clone(), unbox(r));
                return self.builder.build_int_compare(IntPredicate::EQ,vall,valr,"Equal");
            },
            
    
            _ => panic!("Operand not supported: compile_cons")
        };
        return self.cast_int(0)
    }

    //Does initial run of each function, needed for LLVM to "declare" the function line by line. The actual calling is done by compile_function_call.
    //This works a bit different from the interpreter, it is this function that utilizes the stored function in getState.
    fn compile_function(&mut self,functionname: String, func_var: function) -> InstructionValue<'ctx>{
        let (na,args) = match func_var{
            function::parameters_def(n,m,_n,_o)=>{
                return self.builder.build_unreachable();
            }, //Do nothing on define, since this is handled in functionDeclare, except for main.
            function::parameters_call(v,w)=>{
                (v,w)
            },
        };

        //println!("compile_function {:?}",unbox(na.clone()));

        //Fetch function arguments and call fetch_func_types_dec

        let st = self.getState(unbox(na.clone()));

        let fu_st = match st {
            llvmhashstate::state(_fust,_va,fu,_i) => fu,
            _ => panic!("Function not declared: compile_function")
        };

        //println!("Program from state: {:?}",fu_st.clone());

        let (fu_st_args,fu_st_ele) = match unbox(fu_st.clone()) {
            function::parameters_def(_na,args,_ty,ele) => (args,ele),
            _ => panic!("Something went wrong with declaring: compile_function")
        };

        let fu = self.module.get_function(&na).unwrap();

        let basic_block = self.context.append_basic_block(fu, &na);
                
        self.fn_value_opt = Some(fu);
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
        println!("JUNK INPUT na: {:?}, call: {:?}, args: {:?}",unbox(functionname.clone()),unbox(args.clone()),unbox(fuargs.clone()));
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
                    _ => {panic!("Tried to give an assign as argument to function {:?} : function_arguments_call_declare",functionname.clone())},
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
                        // if (self.try_var(*varname.clone())) {
                        //     println!("var {:?} found, not declared",*varname.clone());                
                        //     return;         
                        // }
                        let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),true);
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
                        // if (self.try_var(*varname.clone())) {    
                        //     println!("var {:?} found, not declared",*varname.clone());              
                        //     return;         
                        // }
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
                    _ => {panic!("Tried to give an assign as argument to function {:?} : function_arguments_call_declare",functionname.clone())},
                };
                if (self.try_var(*varname.clone())) {     
                    println!("var {:?} found, not declared",*varname.clone());             
                    return;         
                }
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
                    _ => {panic!("Tried to give an assign as argument to function {:?} : function_arguments_call_declare",functionname.clone())},
                };
                match unbox(va.clone()) {
                    variable::parameters(na,_ty,val) => {
                        match unbox(val) {
                            variable_value::Boolean(b) => {
                                if vartype != Type::boolean {
                                    panic!("Type mismatch in var: function_arguments_call_declare")
                                };
                                // if (self.try_var(*varname.clone())) {    
                                //     println!("var {:?} found, not declared",*varname.clone());              
                                //     return;         
                                // }
                                let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),true);
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
                                // if (self.try_var(*varname.clone())) {   
                                //     println!("var {:?} found, not declared",*varname.clone());               
                                //     return;         
                                // }
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
                        let varname = n;
                        let ptr_val = self.get_var(unbox(varname.clone()));
                        let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();

                        let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false);
                        self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone());
                        self.builder.build_store(ptr.clone(),val);

                        let ptr_val = self.get_var(unbox(varname.clone()));
                        self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
                    },
                    _ => panic!("Type not yet supported: function_arguments_call_declare"),
                };
            },
        }
    }

    //Feches function types in a vector for declaring.
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

    //Fetches the type of a variable.
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

    //Declares a variable in memory by assigning a pointer and inserting to structure and storing it.
    //Now replaces value stored if passing check done by try_var, if this behaviour is incorrect remove lines marked at each switch. -R
    fn declare_var(&mut self, functionname: Box<String>, variable_var: variable) -> InstructionValue<'ctx>{
        match variable_var{
            variable::parameters(na,ty,val) => {
                match unbox(val) {
                    variable_value::Boolean(b) => {
                        if ty != Type::boolean {
                            panic!("Type mismatch in var: declare_var")
                        };
                        let b = self.cast_bool(b);
                        if (self.try_var(*na.clone())) {                                // --------------
                            let ptr = self.get_var(*na.clone());                        // -------------- HERE
                            return self.builder.build_store(ptr.clone(),b);             // --------------
                        }                                                               // --------------
                        let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),true);
                        self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                        return self.builder.build_store(ptr.clone(),b);
                    },
                    variable_value::Number(n) => {
                        if ty != Type::Integer {
                            panic!("Type mismatch in var: declare_var")
                        };
                        let n = self.cast_int(n);
                        if (self.try_var(*na.clone())) {
                            let ptr = self.get_var(*na.clone());
                            return self.builder.build_store(ptr.clone(),n);
                        }
                        let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                        self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                        return self.builder.build_store(ptr.clone(),n);
                    },
                    variable_value::boxs(b) => {
                        let ls = unbox(b);
                        match ls.clone() {
                            List::Num(n) => {
                                if ty != Type::Integer {
                                    panic!("Type mismatch in var: declare_var")
                                };
                                let n = self.cast_int(n);
                                if (self.try_var(*na.clone())) {
                                    let ptr = self.get_var(*na.clone());
                                    return self.builder.build_store(ptr.clone(),n);
                                }
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                return self.builder.build_store(ptr.clone(),n);
                            },
                            List::boolean(b) => {
                                if ty != Type::boolean {
                                    panic!("Type mismatch in var: declare_var")
                                };
                                let b = self.cast_bool(b);
                                if (self.try_var(*na.clone())) {
                                    let ptr = self.get_var(*na.clone());
                                    return self.builder.build_store(ptr.clone(),b);
                                }
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),true);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                return self.builder.build_store(ptr.clone(),b);
                            },
                            List::var(v) => {
                                let varval = self.fetch_var(functionname.clone(),v);
                                if (self.try_var(*na.clone())) {
                                    let ptr = self.get_var(*na.clone());
                                    return self.builder.build_store(ptr.clone(),varval);
                                }
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                return self.builder.build_store(ptr.clone(),varval);
                            },
                            List::Cons(lli,op,rli) => {
                                let val = self.compile_cons(*functionname.clone(), lli, op, rli);
                                if (self.try_var(*na.clone())) {
                                    let ptr = self.get_var(*na.clone());
                                    return self.builder.build_store(ptr.clone(),val);
                                }
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                return self.builder.build_store(ptr.clone(),val);
                            },
                            List::func(fu) => {
                                let val = self.compile_list(unbox(functionname.clone()), ls.clone());
                                if (self.try_var(*na.clone())) {
                                    let ptr = self.get_var(*na.clone());
                                    return self.builder.build_store(ptr.clone(),val);
                                }
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                return self.builder.build_store(ptr.clone(),val);
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
            variable::assign(na,val) => {
                match unbox(val) {
                    variable_value::Boolean(b) => {
                        let b = self.cast_bool(b);                              
                        let ptr = self.get_var(*na.clone());                        
                        return self.builder.build_store(ptr.clone(),b); 
                    },
                    variable_value::Number(n) => {

                        let n = self.cast_int(n);
                        let ptr = self.get_var(*na.clone());
                        return self.builder.build_store(ptr.clone(),n);
                    },
                    variable_value::boxs(b) => {
                        let ls = unbox(b);
                        match ls.clone() {
                            List::Num(n) => {
                                let n = self.cast_int(n);
                                let ptr = self.get_var(*na.clone());
                                return self.builder.build_store(ptr.clone(),n);
                            },
                            List::boolean(b) => {
                                let b = self.cast_bool(b);
                                let ptr = self.get_var(*na.clone());
                                return self.builder.build_store(ptr.clone(),b);
                            },
                            List::var(v) => {
                                let varval = self.fetch_var(functionname.clone(),v);
                                let ptr = self.get_var(*na.clone());
                                return self.builder.build_store(ptr.clone(),varval);
                            },
                            List::Cons(lli,op,rli) => {
                                let val = self.compile_cons(*functionname.clone(), lli, op, rli);
                                let ptr = self.get_var(*na.clone());
                                return self.builder.build_store(ptr.clone(),val);
                            },
                            List::func(fu) => {
                                let val = self.compile_list(unbox(functionname.clone()), ls.clone());
                                let ptr = self.get_var(*na.clone());
                                return self.builder.build_store(ptr.clone(),val);
                            },
                            _ => panic!("Incorrect type: declare_var"),
                        }
                    },
                    _ => panic!("Something failed: declare_var"),
                };
                panic!("Incorrect type: declare_var");
            },
        };
    }

    //Fetches variables depending on the type of the variable, and then adding it to memory
    //Since this returns IntValue it is used when combining values with an operator etc.
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
            variable::assign(na,val) => {
                match unbox(val) {
                    variable_value::Boolean(b) => {
                        let ptr = self.get_var(unbox(na.clone()));
                        let b = self.cast_bool(b);
                        self.builder.build_store(ptr.clone(),b);
                        b
                    },
                    variable_value::Number(n) => {
                        let ptr = self.get_var(unbox(na.clone()));
                        let n = self.cast_int(n);
                        self.builder.build_store(ptr.clone(),n);
                        n
                    },
                    variable_value::boxs(b) => {
                        let ls = unbox(b);
                        match ls.clone() {
                            List::Num(n) => {
                                let ptr = self.get_var(unbox(na.clone()));
                                let n = self.cast_int(n);
                                self.builder.build_store(ptr.clone(),n);
                                n
                            },
                            List::boolean(b) => {
                                let ptr = self.get_var(unbox(na.clone()));
                                let b = self.cast_bool(b);
                                self.builder.build_store(ptr.clone(),b);
                                b
                            },
                            List::var(v) => {
                                let varval = self.fetch_var(functionname.clone(),v);
                                let ptr = self.get_var(unbox(na.clone()));
                                self.builder.build_store(ptr.clone(),varval);
                                varval
                            },
                            List::Cons(lli,op,rli) => {
                                let val = self.compile_cons(*functionname.clone(), lli, op, rli);
                                let ptr = self.get_var(unbox(na.clone()));
                                self.builder.build_store(ptr.clone(),val);
                                val
                            },
                            List::func(fu) => {
                                let val = self.compile_list(unbox(functionname.clone()), ls.clone());
                                let ptr = self.get_var(unbox(na.clone()));
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
        };
    }

    //Compiles an if statement.
    fn compile_if(&mut self, functionname: Box<String>, if_e: if_enum) -> InstructionValue<'ctx> {
        let (ifst, if_body) = match if_e{
            if_enum::condition(v,w)=>(v,w)
        };

        let cond = self.compile_cons(unbox(functionname.clone()), ifst, op::greater, Box::new(List::Num(0))); //Check condition

        let then_block = self.context.append_basic_block(self.fn_value(), "then");
        let cont_block = self.context.append_basic_block(self.fn_value(), "cont");

        self.builder
            .build_conditional_branch(cond, then_block, cont_block);
        self.builder.position_at_end(then_block);

        //Runs actual code in if statement.
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

    //Compiles a while loop.
    fn compile_while(&mut self, functionname: Box<String>, while_e: while_enum) -> InstructionValue<'ctx> {
        let (while_statement, while_body) =  match while_e{
            while_enum::condition(v,w)=>(v,w),
        };

        let do_block = self.context.append_basic_block(self.fn_value(), "do");
        let cont_block = self.context.append_basic_block(self.fn_value(), "cont");

        self.builder
            .build_conditional_branch(self.compile_list(unbox(functionname.clone()), unbox(while_statement.clone())), do_block, cont_block);
        self.builder.position_at_end(do_block);

        //Runs actual code in while loop.
        self.compile_function_elements(functionname.clone(), unbox(while_body));

        self.builder
            .build_conditional_branch(self.compile_list(unbox(functionname.clone()), unbox(while_statement.clone())), do_block, cont_block);
        self.builder.position_at_end(cont_block);


        let phi = self.builder.build_phi(self.context.i32_type(), "while");
        phi.add_incoming(&[
            (&self.cast_int(0), do_block),
            (&self.cast_int(1), do_block),
        ]);
        phi.as_instruction()
    }

    //Builds a return statement for the provided value and returns it.
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
                let val = self.compile_list(unbox(functionname.clone()),unbox(va));
                return self.builder.build_return(Some(&val));
            },
            _ => panic!("Return does not support this type: return_execute"),
        }
    }

    //Compile the elements of a function by checking what the instruction is, and then compiling each instruction accordingly
    fn compile_function_elements(&mut self, functionname: Box<String>, fe: function_elements) -> (InstructionValue<'ctx>, bool){
        match fe {
            function_elements::ele_list(v,w)=>{
                let ele1: function_elements = unbox(v);
                let ele2: function_elements = unbox(w);
                let res1 = self.compile_function_elements(functionname.clone(), ele1);
                if (res1.1) {   //Checks if compiled element is a return statement.
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

    //Compile a function call, no need to fetch from state since it is already "declared" using LLVM in compile_function.
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

        //println!("function call {:?}",fname);

        // let st = self.getState(unbox(funame.clone()));

        // let fu_st = match st {
        //     llvmhashstate::state(_fust,_va,fu,_i) => fu,
        //     _ => panic!("Function not declared: compile_function")
        // };

        // //println!("Program from state: {:?}",fu_st.clone());

        // let (fu_st_args,fu_st_ele) = match unbox(fu_st.clone()) {
        //     function::parameters_def(_na,args,_ty,ele) => (args,ele),
        //     _ => panic!("Something went wrong with declaring: compile_function")
        // };

        let fu = self.module.get_function(&fname).unwrap(); //Fetch function declared in compile_function.
        let args = self.compile_function_arguments_call_declare(funame.clone(), fargs);
        
        let call = self //Build the function call with arguments and function fetched from LLVM.
            .builder
            .build_call(fu, &args, &fname)
            .try_as_basic_value()
            .left()
            .unwrap();
        
        match call {
            value => value.into_int_value(),
        }
    }

    //Send with function_arguments to retrive argument names to send along, should make this better and the same as interpreter. R 22/7
    //Gets function_arguments_call into a vector used in compile_function_call. This vector is then given to the builder to build the call.
    fn compile_function_arguments_call_declare(&mut self, functionname: Box<String>, args: Box<function_arguments_call>) -> Vec<BasicValueEnum<'ctx>> {
        if (unbox(functionname.clone()) == "main") {
            return vec![inkwell::values::BasicValueEnum::IntValue(self.cast_int(0))];
        }
        let temp: function_arguments_call = unbox(args.clone());
        match temp {
            function_arguments_call::arg_call_list(a1,a2) => {
                let mut leftSide = self.compile_function_arguments_call_declare(functionname.clone(),a1);
                let mut rightSide = self.compile_function_arguments_call_declare(functionname.clone(),a2);
                leftSide.extend(rightSide);
                return leftSide;
            },
            function_arguments_call::bx(bo) => {
                let varname = Box::new("empty".to_string());
                let unb = unbox(bo);
                match unb {
                    List::var(v) => {
                        let newargs = Box::new(function_arguments_call::variable(Box::new(v)));
                        return self.compile_function_arguments_call_declare(functionname.clone(), newargs);
                    },
                    List::boolean(b) => {
                        let b = self.cast_bool(b);
                        let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),true);
                        self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone());
                        self.builder.build_store(ptr.clone(),b);

                        let ptr_val = self.get_var(unbox(varname.clone()));
                        let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
                        let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
                        return result;
                    },
                    List::Num(n) => {
                        let n = self.cast_int(n);
                        let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false);
                        self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone());
                        self.builder.build_store(ptr.clone(),n);

                        let ptr_val = self.get_var(unbox(varname.clone()));
                        let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
                        let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
                        return result;
                    },
                    List::Cons(a,o,b) => {
                        let n = self.compile_cons(unbox(functionname.clone()),a,o,b);
                        let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false);
                        self.builder.build_store(ptr.clone(),n);

                        let ptr_val = self.get_var(unbox(varname.clone()));
                        let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
                        let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
                        return result;
                    },
                    _ => panic!("asd"),
                };
            },
            function_arguments_call::function(fu) => {
                let varname = Box::new("empty".to_string());
                let val = self.compile_function_call(functionname.clone(), unbox(fu));
                let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false);
                self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone());
                self.builder.build_store(ptr.clone(),val);

                let ptr_val = self.get_var(unbox(varname.clone()));
                let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
                let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
                return result;
            },
            function_arguments_call::variable(va) => {
                let varname = Box::new("empty".to_string());
                match unbox(va.clone()) {
                    variable::parameters(na,_ty,val) => {
                        match unbox(val) {
                            variable_value::Boolean(b) => {
                                let b = self.cast_bool(b);
                                let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),true);
                                self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),b);

                                let ptr_val = self.get_var(unbox(varname.clone()));
                                let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
                                let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
                                return result;
                            },
                            variable_value::Number(n) => {
                                let n = self.cast_int(n);
                                let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false);
                                self.builder.build_store(ptr.clone(),n);

                                let ptr_val = self.get_var(unbox(varname.clone()));
                                let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
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

//     //Send with function_arguments to retrive argument names to send along, should make this better and the same as interpreter. R 22/7
//     //Gets function_arguments_call into a vector used in compile_function_call. This vector is then given to the builder to build the call.
//     fn compile_function_arguments_call_declare(&mut self, functionname: Box<String>, args: Box<function_arguments_call>, fuargs: Box<function_arguments>) -> Vec<BasicValueEnum<'ctx>> {
//         println!("REAL INPUT na: {:?}, call: {:?}, args: {:?}",unbox(functionname.clone()),unbox(args.clone()),unbox(fuargs.clone()));
//         if (unbox(functionname.clone()) == "main") {
//             return vec![inkwell::values::BasicValueEnum::IntValue(self.cast_int(0))];
//         }
//         let temp: function_arguments_call = unbox(args.clone());
//         match temp {
//             function_arguments_call::arg_call_list(a1,a2) => {
//                 let (varl, far) = match unbox(fuargs) {
//                     function_arguments::arg_list(le,ri) => {(le,ri)},
//                     _ => return panic!("Too many inputs given to function {:?} : compile_function_arguments_call_declare",functionname.clone()),
//                 };
//                 let fal = Box::new(function_arguments::var(varl));
//                 let mut leftSide = self.compile_function_arguments_call_declare(functionname.clone(),a1,fal);
//                 let mut rightSide = self.compile_function_arguments_call_declare(functionname.clone(),a2,far);
//                 leftSide.extend(rightSide);
//                 return leftSide;
//             },
//             function_arguments_call::bx(bo) => {
//                 let functionargs = match unbox(fuargs.clone()) {
//                     function_arguments::arg_list(le,ri) => {return panic!("jada")},
//                     function_arguments::var(v) => {v},
//                 };
//                 let (varname,vartype) = match functionargs {
//                     variable::parameters(n,t,_v) => {(n,t)},
//                     variable::name(n) => {(n,Type::unknown(0))},
//                     _ => {panic!("Tried to give an assign as argument to function {:?} : compile_function_arguments_call_declare",functionname.clone())},
//                 };
//                 let unb = unbox(bo);
//                 match unb {
//                     List::var(v) => {
//                         let newargs = Box::new(function_arguments_call::variable(Box::new(v)));
//                         return self.compile_function_arguments_call_declare(functionname.clone(), newargs, fuargs.clone());
//                     },
//                     List::boolean(b) => {
//                         let b = self.cast_bool(b);
//                         if (self.try_var(*varname.clone())) {                
//                             let ptr = self.get_var(*varname.clone());                        
//                             self.builder.build_store(ptr.clone(),b);
//                             let ptr_val = self.get_var(unbox(varname.clone()));
//                             let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
//                             let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
//                             return result;             
//                         }
//                         let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),true);
//                         self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone());
//                         self.builder.build_store(ptr.clone(),b);

//                         let ptr_val = self.get_var(unbox(varname.clone()));
//                         let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
//                         let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
//                         return result;
//                     },
//                     List::Num(n) => {
//                         let n = self.cast_int(n);
//                         if (self.try_var(*varname.clone())) {                
//                             let ptr = self.get_var(*varname.clone());                        
//                             self.builder.build_store(ptr.clone(),n);
//                             let ptr_val = self.get_var(unbox(varname.clone()));
//                             let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
//                             let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
//                             return result;             
//                         }
//                         let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false);
//                         self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone());
//                         self.builder.build_store(ptr.clone(),n);

//                         let ptr_val = self.get_var(unbox(varname.clone()));
//                         let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
//                         let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
//                         return result;
//                     },
//                     List::Cons(a,o,b) => {
//                         let n = self.compile_cons(unbox(functionname.clone()),a,o,b);
//                         if (self.try_var(*varname.clone())) {                
//                             let ptr = self.get_var(*varname.clone());                        
//                             self.builder.build_store(ptr.clone(),n);
//                             let ptr_val = self.get_var(unbox(varname.clone()));
//                             let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
//                             let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
//                             return result;             
//                         }
//                         let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false);
//                         self.builder.build_store(ptr.clone(),n);

//                         let ptr_val = self.get_var(unbox(varname.clone()));
//                         let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
//                         let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
//                         return result;
//                     },
//                     _ => panic!("asd"),
//                 };
//             },
//             function_arguments_call::function(fu) => {
//                 let functionargs = match unbox(fuargs.clone()) {
//                     function_arguments::arg_list(le,ri) => {return panic!("jada")},
//                     function_arguments::var(v) => {v},
//                 };
//                 let (varname,vartype) = match functionargs {
//                     variable::parameters(n,t,_v) => {(n,t)},
//                     variable::name(n) => {(n,Type::unknown(0))},
//                     _ => {panic!("Tried to give an assign as argument to function {:?} : compile_function_arguments_call_declare",functionname.clone())},
//                 };
//                 let val = self.compile_function_call(functionname.clone(), unbox(fu));
//                 if (self.try_var(*varname.clone())) {                
//                     let ptr = self.get_var(*varname.clone());                        
//                     self.builder.build_store(ptr.clone(),val);
//                     let ptr_val = self.get_var(unbox(varname.clone()));
//                     let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
//                     let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
//                     return result;             
//                 }
//                 let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false);
//                 self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone());
//                 self.builder.build_store(ptr.clone(),val);

//                 let ptr_val = self.get_var(unbox(varname.clone()));
//                 let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
//                 let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
//                 return result;
//             },
//             function_arguments_call::variable(va) => {
//                 let functionargs = match unbox(fuargs.clone()) {
//                     function_arguments::arg_list(le,ri) => {return panic!("jada")},
//                     function_arguments::var(v) => {v},
//                 };
//                 let (varname,vartype) = match functionargs {
//                     variable::parameters(n,t,_v) => {(n,t)},
//                     variable::name(n) => {(n,Type::unknown(0))},
//                     _ => {panic!("Tried to give an assign as argument to function {:?} : compile_function_arguments_call_declare",functionname.clone())},
//                 };
//                 match unbox(va.clone()) {
//                     variable::parameters(na,_ty,val) => {
//                         match unbox(val) {
//                             variable_value::Boolean(b) => {
//                                 let b = self.cast_bool(b);
//                                 if (self.try_var(*varname.clone())) {                
//                                     let ptr = self.get_var(*varname.clone());                        
//                                     self.builder.build_store(ptr.clone(),b);
//                                     let ptr_val = self.get_var(unbox(varname.clone()));
//                                     let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
//                                     let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
//                                     return result;             
//                                 }
//                                 let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),true);
//                                 self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone());
//                                 self.builder.build_store(ptr.clone(),b);

//                                 let ptr_val = self.get_var(unbox(varname.clone()));
//                                 let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
//                                 let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
//                                 return result;
//                             },
//                             variable_value::Number(n) => {
//                                 let n = self.cast_int(n);
//                                 if (self.try_var(*varname.clone())) {                
//                                     let ptr = self.get_var(*varname.clone());                        
//                                     self.builder.build_store(ptr.clone(),n);
//                                     let ptr_val = self.get_var(unbox(varname.clone()));
//                                     let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
//                                     let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
//                                     return result;             
//                                 }
//                                 let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false);
//                                 self.builder.build_store(ptr.clone(),n);

//                                 let ptr_val = self.get_var(unbox(varname.clone()));
//                                 let val = self.builder.build_load(*ptr_val, &(unbox(varname.clone()))).into_int_value();
//                                 let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
//                                 return result;
//                             },
//                             _ => panic!("temp"), //Might have to include more cases for variable_value here if needed.
//                         };
//                     },
//                     variable::name(n) => {
//                         let vnam = unbox(n);
//                         let ptr = self.get_var(vnam.clone());
//                         let val = self.builder.build_load(*ptr,&vnam.clone()).into_int_value();

//                         //Here the variable should be added to the current functions local variables. 2021-04-25
//                         self.insert_var(*functionname.clone(), vnam.clone(), ptr.clone());
//                         let ptr_val = self.get_var(vnam.clone());
//                         let val = self.builder.build_load(*ptr_val, &vnam.clone()).into_int_value();
//                         let result: Vec<BasicValueEnum> = vec![inkwell::values::BasicValueEnum::IntValue(val)];
//                         return result;
//                     },
//                     _ => panic!("Type not yet supported: function_arguments_call_declare"),
//                 };
//             },
//         }
//     }
// }

fn buildFunctionCall(fu: function) -> function {
    match fu {
        function::parameters_def(na,ar,ty,ele) => { //When we find a define, run a "call" to that function. Specifically needed for LLVM since it relies on this to "make" the function for calling later.
            let args = getJunkInput(unbox(ar));
            let fc = function::parameters_call(na,Box::new(args));
            return fc;
        },
        _ => panic!(),
    }
    
}

fn getJunkInput(fuargs: function_arguments) -> function_arguments_call {
    match fuargs {
        function_arguments::arg_list(va,re) => {
            let ty = match va {
                variable::parameters(na,ty,val) => {
                    ty
                }
                _ => panic!(),
            };
            let junkval = match ty {
                Type::Integer => {function_arguments_call::bx(Box::new(List::Num(0)))},
                _ => {function_arguments_call::bx(Box::new(List::boolean(false)))},
            };
            let resval = getJunkInput(unbox(re));
            return function_arguments_call::arg_call_list(Box::new(junkval),Box::new(resval));
        },
        function_arguments::var(va) => {
            let ty = match va {
                variable::parameters(na,ty,val) => {
                    ty
                }
                _ => panic!(),
            };
            let junkval = match ty {
                Type::Integer => {function_arguments_call::bx(Box::new(List::Num(0)))},
                _ => {function_arguments_call::bx(Box::new(List::boolean(false)))},
            };
            return junkval;
        }
    }
}



fn main() {}