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
    //Mem should be sorted on function name and contain a second hashmap that actually stores the variables under name with the pointer. 2021-04-25

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
        execution_engine: &execution_engine
    };

    let mut state: HashMap<String, llvmhashstate> = HashMap::new();

    let mut mem: HashMap<String, HashMap<String, PointerValue>> = HashMap::new();

    let mut dec_iter = statements.iter();

    for stmt in dec_iter {
        codegen.functionDeclare(stmt.clone(),state,mem); //Loop through and declare all functions into state.
    }

    let mut iter = statements.iter(); 

    for stmt in iter { //Run program
        codegen.compile_list(unbox(nm.clone()),stmt.clone(), &mut state, &mut mem);
    }
    //assert!(iter.is_ok());
}

impl<'a, 'ctx> CodeGen<'a, 'ctx> {
    fn cast_int(&self, int: i32, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) -> IntValue<'ctx> {
        self.context.i32_type().const_int(int as u64, false)
    }

    fn cast_bool(&self, b: bool, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) -> IntValue<'ctx> {
        match b {
            true => self.context.bool_type().const_int(1, false),
            false => self.context.bool_type().const_int(0, false),
        }
    }

    fn get_var(&self, functionname: String, na: String, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) -> &'ctx PointerValue<'ctx> {
        let mut hmapp: &HashMap<String, PointerValue<'ctx>> = match mem.get(&functionname) {
            Some(v) => v,
            None => panic!("Not found in memory: get_var"),
        };
        match hmapp.get(&na) {
            Some(v) => return v,
            None => panic!("Not found in memory: get_var"),
        }
    }

    fn get_hashmap(&self, na: String, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) -> &'ctx HashMap<String, PointerValue<'ctx>> {
        match mem.get(&na) {
            Some(v) => return v,
            None => panic!("Not found in memory: get_hashmap")
        }
    }

    fn getState(&self, function: String, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) -> &llvmhashstate<'ctx> {
        let result = state.get(&function);
        let temp = match result {
            Some(val) => val,
            None => panic!("Get state failed!: getState"),
        };
        return temp;
    }

    fn insert_var(&mut self, functionname: String, na: String, po: PointerValue<'ctx>, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) {
        let hmapp = self.get_hashmap(functionname.clone(),state,mem);
        let mut nhmapp = hmapp.clone();
        nhmapp.insert(na,po);
        mem.insert(functionname.clone(),nhmapp.clone());
    }

    //Used to create a new hashmap for local variables inside memory.
    fn create_func(&mut self, functionname: String, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) {
        let mut temp: HashMap<String, PointerValue<'ctx>> = HashMap::new();
        mem.insert(functionname,temp);
    }

    fn compile_list(&mut self, functionname: String, ls: List, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) -> IntValue<'ctx> {
        match ls{
            List::paran(v) => {return self.compile_list(functionname.clone(), unbox(v),state,mem)},
            List::Cons(v,w,x) => {return self.compile_cons(functionname.clone(),v,w,x,state,mem)},
            List::Num(v) => {return self.cast_int(v,state,mem)},
            List::boolean(v)=>{return self.cast_bool(v,state,mem)},
            List::func(fu) => {return self.cast_int(0,state,mem)},
            List::var(v) => {return self.compile_var(Box::new(functionname.clone()), v,state,mem)},
            _ => panic!("Something went wrong: execute_List"),
        };
    }

    //TODO
    fn allocate_pointer(&mut self, functionname: String, na: String, is_bool: bool, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) -> PointerValue<'ctx> {
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

        self.insert_var(functionname.clone(),na.clone(),pa,state,mem);
        return pa;
    }

    fn functionDeclare(&mut self, ls: List, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) {
        match ls {
            List::func(f) => {
                match f.clone() {
                    function::parameters_def(na,ar,ty,ele) => {
                        let temp: Vec<llvmhashvariable> = Vec::new();
                        let tempstring: String = unbox(na.clone());
                        state.insert(tempstring.clone(),llvmhashstate::state(Box::new(llvmfunctionstate::Declared),Box::new(temp),Box::new(f.clone()),-1));
                        self.create_func(tempstring.clone(),state,mem);
                    },
                    _ => (),
                };
            },
            _ => (),// Do nothing
        }
    }

    //CHange return statements to use builder istead, not needed to chech for logic ourselves.
    fn compile_cons(&mut self, functionname: String, l: Box<List>, oper: op ,r:  Box<List>, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) -> IntValue<'ctx> {
        let expr = match oper{
            op::add => {
                let vall = self.compile_list(functionname.clone(), unbox(l),state,mem);
                let valr = self.compile_list(functionname.clone(), unbox(r),state,mem);
                return self.builder.build_int_add(vall,valr,"add");
            },
    
            op::sub => {
                let vall = self.compile_list(functionname.clone(), unbox(l),state,mem);
                let valr = self.compile_list(functionname.clone(), unbox(r),state,mem);
                return self.builder.build_int_sub(vall, valr, "sub");
            },
    
            op::div => {
                let vall = self.compile_list(functionname.clone(), unbox(l),state,mem);
                let valr = self.compile_list(functionname.clone(), unbox(r),state,mem);
                return self.builder.build_int_unsigned_div(vall,valr,"div");
            },
    
            op::res => {
                let vall= self.compile_list(functionname.clone(), unbox(l),state,mem);
                let valr = self.compile_list(functionname.clone(), unbox(r),state,mem);
                return self.builder.build_int_add(vall,valr,"NOT IMPLEMENTED");
            },
            op::mult => {
                let vall = self.compile_list(functionname.clone(), unbox(l),state,mem);
                let valr = self.compile_list(functionname.clone(), unbox(r),state,mem);
                return self.builder.build_int_mul(vall,valr,"mul");
            },
            op::less => {
                let vall = self.compile_list(functionname.clone(), unbox(l),state,mem);
                let valr = self.compile_list(functionname.clone(), unbox(r),state,mem);
                return self.builder.build_int_compare(IntPredicate::ULT,vall,valr,"Lesser than");
            },
            op::greater => {
                let vall = self.compile_list(functionname.clone(), unbox(l),state,mem);
                let valr = self.compile_list(functionname.clone(), unbox(r),state,mem);
                return self.builder.build_int_compare(IntPredicate::UGT, vall, valr, "Greater than")
            },
            op::lessEqual => {
                let vall = self.compile_list(functionname.clone(), unbox(l),state,mem);
                let valr = self.compile_list(functionname.clone(), unbox(r),state,mem);
                return self.builder.build_int_compare(IntPredicate::ULE,vall,valr,"Lesser or equal");
            },
            op::greatEqual => {
                let vall = self.compile_list(functionname.clone(), unbox(l),state,mem);
                let valr = self.compile_list(functionname.clone(), unbox(r),state,mem);
                return self.builder.build_int_compare(IntPredicate::UGE,vall,valr,"Greater or equal");
            },
            op::and => {
                let vall = self.compile_list(functionname.clone(), unbox(l),state,mem);
                let valr = self.compile_list(functionname.clone(), unbox(r),state,mem);
                return self.builder.build_and(vall, valr, "and")
            },
            op::or => {
                let vall = self.compile_list(functionname.clone(), unbox(l),state,mem);
                let valr = self.compile_list(functionname.clone(), unbox(r),state,mem);
                return self.builder.build_or(vall, valr, "or");
            },
            
    
            _ => panic!("Operand not supported: compile_cons")
        };
        return self.cast_int(0,state,mem)
    }

    fn compile_function(&mut self,functionname: String, func_var: function, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) {
        match func_var{
            function::parameters_def(na,_m,_n,_o)=>{
                if unbox(na.clone()) == "main" { //Special case for main. Calls in when we find define in code. Ensures that it is always called.
                    let w = function_arguments_call::variable(Box::new(variable::name(Box::new("".to_string()))));
                    self.compile_function_arguments_call_execute(na.clone(), na.clone(), Box::new(w),state,mem);
                }
            }, //Do nothing on define, since this is handled in functionDeclare, except for main.
            function::parameters_call(v,w)=>{
                self.compile_function_arguments_call_execute(v.clone(), Box::new(functionname.clone()), w,state,mem);
            },
        };
    }

    //Removed check in memory so now only one match for everything, simplify memory to just include addressmap. Should make fetching easier too.
    fn compile_var(&mut self, functionname: Box<String>, variable_var: variable, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) -> IntValue<'ctx> {
        match variable_var{
            variable::parameters(na,ty,val) => {
                match unbox(val) {
                    variable_value::Boolean(b) => {
                        if ty != Type::boolean {
                            panic!("Type mismatch in var: compile_var")
                        };
                        let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),true,state,mem);
                        let b = self.cast_bool(b,state,mem);
                        self.builder.build_store(ptr.clone(),b);
                        self.insert_var(*functionname.clone(), *na.clone(), ptr.clone(),state,mem);
                    },
                    variable_value::Number(n) => {
                        if ty != Type::Integer {
                            panic!("Type mismatch in var: compile_var")
                        };
                        let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false,state,mem);
                        let n = self.cast_int(n,state,mem);
                        self.builder.build_store(ptr.clone(),n);
                        self.insert_var(*functionname.clone(), *na.clone(), ptr.clone(),state,mem);
                    },
                    variable_value::boxs(b) => {
                        let ls = unbox(b);
                        match ls.clone() {
                            List::Num(n) => {
                                if ty != Type::Integer {
                                    panic!("Type mismatch in var: compile_var")
                                };
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false,state,mem);
                                let n = self.cast_int(n,state,mem);
                                self.builder.build_store(ptr.clone(),n);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone(),state,mem);
                            },
                            List::boolean(b) => {
                                if ty != Type::boolean {
                                    panic!("Type mismatch in var: compile_var")
                                };
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),true,state,mem);
                                let b = self.cast_bool(b,state,mem);
                                self.builder.build_store(ptr.clone(),b);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone(),state,mem);
                            },
                            List::var(v) => {
                                let varval = self.compile_var(functionname.clone(),v,state,mem);
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false,state,mem);
                                self.builder.build_store(ptr.clone(),varval);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone(),state,mem);
                            },
                            List::Cons(lli,op,rli) => {
                                let val = self.compile_cons(*functionname.clone(), lli, op, rli,state,mem);
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false,state,mem);
                                self.builder.build_store(ptr.clone(),val);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone(),state,mem);
                            },
                            List::func(fu) => {
                                let val = self.compile_list(unbox(functionname.clone()), ls.clone(),state,mem);
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false,state,mem);
                                self.builder.build_store(ptr.clone(),val);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone(),state,mem);
                            },
                            _ => (),
                        };
                    },
                };
                return self.cast_int(0,state,mem);
            },
            variable::name(v)=>{ // Access local var with same name and return it.
                let ptr = self.get_var(*functionname.clone(), *v.clone(),state,mem);
                return self.builder.build_load(*ptr,&v.clone()).into_int_value();
            },
        };
    }

    fn compile_if(&mut self, functionname: Box<String>, if_e: if_enum, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) {
        let (ifst, if_body) = match if_e{
            if_enum::condition(v,w)=>(v,w)
        };
        let temp = self.compile_cons(unbox(functionname.clone()), ifst, op::greater, Box::new(List::Num(0)),state,mem);
        if (temp == (self.cast_int(1,state,mem))) {
            self.compile_function_elements(functionname.clone(), unbox(if_body),state,mem);
        }
    }

    fn compile_while(&mut self, functionname: Box<String>, while_e: while_enum, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) {
        let (while_statement, while_body) =  match while_e{
            while_enum::condition(v,w)=>(v,w),
        };
        let temp = self.compile_cons(unbox(functionname.clone()), while_statement.clone(), op::greater, Box::new(List::Num(0)),state,mem);
        while (temp == (self.cast_int(1,state,mem))) {
            self.compile_function_elements(functionname.clone(), unbox(while_body.clone()),state,mem);
        }
    }

    //Used to return  -> (InstructionValue<'ctx>, bool)
    //Use self.builder.build_return(Some(&var)) to return only value, no need to memory and local vars and state. Should be a lot smaller after everything is removed.
    fn compile_return(&mut self, functionname: Box<String>, var_val: variable_value, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) {
        let fnstate = self.getState(*functionname.clone(),state,mem);
        match fnstate {
            llvmhashstate::state(_st,vars,fu,line) => {
                match var_val {
                    variable_value::Boolean(b) => {
                        let b = self.cast_bool(b,state,mem);
                        let temp = llvmhashdata::value(b);
                        let st = llvmhashstate::state(Box::new(llvmfunctionstate::Returned(Box::new(temp))),vars.clone(),fu.clone(),line.clone());
                        state.insert(*functionname.clone(),st);
                        //return self.builder.build_return(Some(b),true);
                    },
                    variable_value::Number(n) => {
                        let n = self.cast_int(n,state,mem);
                        let temp = llvmhashdata::value(n);
                        let st = llvmhashstate::state(Box::new(llvmfunctionstate::Returned(Box::new(temp))),vars.clone(),fu.clone(),line.clone());
                        state.insert(*functionname.clone(),st);
                    },
                    variable_value::variable(v) => {
                        match unbox(v) {
                            variable::name(n) => {
                                let varname = unbox(n);
                                let ptr = self.get_var(*functionname.clone(), varname.clone(),state,mem);
                                let val = self.builder.build_load(*ptr,&varname.clone()).into_int_value();
                                let temp = llvmhashdata::value(val);
                                let st = llvmhashstate::state(Box::new(llvmfunctionstate::Returned(Box::new(temp))),vars.clone(),fu.clone(),line.clone());
                                state.insert(*functionname.clone(),st);
                            },
                            _ => panic!("Return does not support this type: return_execute"),
                        };
                    },
                    variable_value::boxs(va) => {
                        match unbox(va) {
                            List::var(v) => {
                                let varval = variable_value::variable(Box::new(v));
                                self.compile_return(functionname.clone(), varval,state,mem);
                            },
                            List::Cons(bl, opr, br) => {
                                let consval = self.compile_cons(unbox(functionname.clone()), bl, opr, br,state,mem);
                                let hashdata_ret = llvmhashdata::value(consval);
                                let temp = llvmhashstate::state(Box::new(llvmfunctionstate::Returned(Box::new(hashdata_ret))),vars.clone(),fu.clone(),line.clone());
                                state.insert(*functionname.clone(),temp);
                            },
                            List::boolean(b) => {
                                let b = self.cast_bool(b,state,mem);
                                let hashdata_ret = llvmhashdata::value(b);
                                let temp = llvmhashstate::state(Box::new(llvmfunctionstate::Returned(Box::new(hashdata_ret))),vars.clone(),fu.clone(),line.clone());
                                state.insert(*functionname.clone(),temp);
                            },
                            List::Num(n) => {
                                let n = self.cast_int(n,state,mem);
                                let hashdata_ret = llvmhashdata::value(n);
                                let temp = llvmhashstate::state(Box::new(llvmfunctionstate::Returned(Box::new(hashdata_ret))),vars.clone(),fu.clone(),line.clone());
                                state.insert(*functionname.clone(),temp);
                            },
                            _ => panic!("Return does not support this type: return_execute"),
                        }
                    }
                    _ => panic!("Return does not support this type: return_execute"),
                }
            },
            _ => panic!("Function does not exist: return execute"),
        };
    }

    fn compile_function_elements(&mut self, functionname: Box<String>, fe: function_elements, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) {
        match fe {
            function_elements::ele_list(v,w)=>{
                let ele1: function_elements = unbox(v);
                let ele2: function_elements = unbox(w);
                let res1 = self.compile_function_elements(functionname.clone(), ele1,state,mem);
                let res2 = self.compile_function_elements(functionname.clone(), ele2,state,mem);
            },
            function_elements::boxs(v)=>{
                let box_cont: variable = unbox(v);
                self.compile_var(functionname, box_cont,state,mem); 
            },
            function_elements::if_box(v)=>{
                let box_cont= unbox(v);
                self.compile_if(functionname, box_cont,state,mem);
            },
            function_elements::List(v)=>{
                self.compile_list(unbox(functionname), v,state,mem);
            },
            function_elements::function(v)=>{
                self.compile_function(unbox(functionname), v,state,mem);
            },
            function_elements::variable(v)=>{
                self.compile_var(functionname, v,state,mem);
            },
            function_elements::if_enum(v)=>{
                self.compile_if(functionname, v,state,mem);
            },
            function_elements::while_enum(v) => {
                self.compile_while(functionname, v,state,mem);
            },
            function_elements::return_val(v) => {
                self.compile_return(functionname,v,state,mem);
            },
        }
    }

    fn compile_function_arguments_call_execute(&mut self, functionname: Box<String>, oldfunctionname: Box<String>, args: Box<function_arguments_call>, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) {
        let st = self.getState(*functionname.clone(),state,mem);
        match st {
            llvmhashstate::state(_st,v,fu,line) => {
                let temp = llvmhashstate::state(Box::new(llvmfunctionstate::Running),v.clone(),fu.clone(),line.clone());
                state.insert(*functionname.clone(),temp);
                match unbox(fu.clone()) {
                    function::parameters_def(_na,fuag,_ty,ele) => {
                        let functionargs = fuag.clone();
                        self.compile_function_arguments_call_declare(functionname.clone(), oldfunctionname.clone(), args, functionargs,state,mem);
                        self.compile_function_elements(functionname.clone(),unbox(ele.clone()),state,mem);
                    },
                    _ => panic!("Function stored incorrectly in mem: function_arguments_call_execute"),
                };
            },
            _ => panic!("Function is not declared!: function_arguments_call_execute"),
        }
    }

    fn compile_function_arguments_call_declare(&mut self, functionname: Box<String>, oldfunctionname: Box<String>, args: Box<function_arguments_call>, fuargs: Box<function_arguments>, state: &'ctx mut HashMap<String, llvmhashstate<'ctx>>, mem: &'ctx mut HashMap<String, HashMap<String, PointerValue<'ctx>>>) {
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
                let _leftSide = self.compile_function_arguments_call_declare(functionname.clone(),oldfunctionname.clone(),a1,fal,state,mem);
                let _rightSide = self.compile_function_arguments_call_declare(functionname.clone(),oldfunctionname.clone(),a2,far,state,mem);
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
                        self.compile_function_arguments_call_declare(functionname.clone(), oldfunctionname.clone(), newargs, fuargs.clone(),state,mem);
                    },
                    List::boolean(b) => {
                        if vartype != Type::boolean {
                            panic!("Type mismatch in var: function_arguments_call_declare")
                        };
                        let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),true,state,mem);
                        let b = self.cast_bool(b,state,mem);
                        self.builder.build_store(ptr.clone(),b);
                        self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone(),state,mem);
                    },
                    List::Num(n) => {
                        if vartype != Type::Integer {
                            panic!("Type mismatch in var: function_arguments_call_declare")
                        };
                        let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false,state,mem);
                        let n = self.cast_int(n,state,mem);
                        self.builder.build_store(ptr.clone(),n);
                        self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone(),state,mem);
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
                match unbox(fu) {
                    function::parameters_call(na, ar) => {
                        //let st2: &mut HashMap<String, hashstate> = &mut state.clone();
                        self.compile_function_arguments_call_execute(na.clone(), functionname.clone(), ar,state,mem);
                        //Wait for state to change to Returned, and get value from memory. (functionstate::Returned(Box<hashdata::address/value>))
                        let returnedState = self.getState(*na.clone(),state,mem);
                        match returnedState {
                            llvmhashstate::state(st,_v,fu,_line) => {
                                match unbox(st.clone()) {
                                    llvmfunctionstate::Returned(v) => {
                                        match unbox(v.clone()) {
                                            llvmhashdata::value(v) => {
                                                let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false,state,mem);
                                                self.builder.build_store(ptr.clone(),v);
                                                self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone(),state,mem);
                                            },
                                            _ => panic!("Previous function call returned nothing!: function_arguments_call_declare"),
                                        }
                                    },
                                    _ => panic!("Previous function call returned nothing!: function_arguments_call_declare"),
                                };

                            },
                            _ => panic!("State does not exist!: function_arguments_call_declare"),
                        };
    
                    },
                    _ => panic!("Not a function call: function_arguments_call_declare"),
                }
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
                                let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),true,state,mem);
                                let b = self.cast_bool(b,state,mem);
                                self.builder.build_store(ptr.clone(),b);
                                self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone(),state,mem);
                            },
                            variable_value::Number(n) => {
                                if vartype != Type::Integer {
                                    panic!("Type mismatch in var: function_arguments_call_declare")
                                };
                                let ptr = self.allocate_pointer(*functionname.clone(),*varname.clone(),false,state,mem);
                                let n = self.cast_int(n,state,mem);
                                self.builder.build_store(ptr.clone(),n);
                                self.insert_var(*functionname.clone(), *varname.clone(), ptr.clone(),state,mem);
                            },
                            _ => panic!("temp"), //Might have to include more cases for variable_value here if needed.
                        };
                    },
                    variable::name(n) => {
                        let vnam = unbox(n);
                        let ptr = self.get_var(*functionname.clone(), vnam.clone(),state,mem);
                        let val = self.builder.build_load(*ptr,&vnam.clone()).into_int_value();

                        //Here the variable should be added to the current functions local variables. 2021-04-25
                        self.insert_var(*functionname.clone(), vnam.clone(), ptr.clone(),state,mem);
                    },
                    _ => panic!("Type not yet supported: function_arguments_call_declare"),
                };
            },
        }
    }
}


fn main() {}