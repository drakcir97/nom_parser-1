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
    };

    //let mut state: HashMap<String, llvmhashstate> = HashMap::new();

    //let mut mem: HashMap<String, HashMap<String, PointerValue>> = HashMap::new();

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

    fn insert_var(&self, functionname: String, na: String, pa: PointerValue<'ctx>) {
        self.var.insert(na,pa);
    }

    fn compile_list(&mut self, functionname: String, ls: List) -> IntValue<'ctx> {
        match ls{
            List::paran(v) => {return self.compile_list(functionname.clone(), unbox(v))},
            List::Cons(v,w,x) => {return self.compile_cons(functionname.clone(),v,w,x)},
            List::Num(v) => {return self.cast_int(v)},
            List::boolean(v)=>{return self.cast_bool(v)},
            List::func(fu) => {return self.compile_function_call(Box::new(functionname.clone()),fu)},
            List::var(v) => {return self.compile_var(Box::new(functionname.clone()), v)},
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
        match func_var{
            function::parameters_def(na,_m,_n,_o)=>{
                if unbox(na.clone()) == "main" { //Special case for main. Calls in when we find define in code. Ensures that it is always called.
                    let w = function_arguments_call::variable(Box::new(variable::name(Box::new("".to_string()))));
                    //return self.compile_function_call(na.clone(), na.clone(), Box::new(w));
                }
            }, //Do nothing on define, since this is handled in functionDeclare, except for main.
            function::parameters_call(v,w)=>{
                //return self.compile_function_call(v.clone(), Box::new(functionname.clone()), w);
            },
        };
    }

    //Removed check in memory so now only one match for everything, simplify memory to just include addressmap. Should make fetching easier too.
    fn compile_var(&mut self, functionname: Box<String>, variable_var: variable) -> IntValue<'ctx> {
        match variable_var{
            variable::parameters(na,ty,val) => {
                match unbox(val) {
                    variable_value::Boolean(b) => {
                        if ty != Type::boolean {
                            panic!("Type mismatch in var: compile_var")
                        };
                        let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),true);
                        let b = self.cast_bool(b);
                        self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                        self.builder.build_store(ptr.clone(),b)
                    },
                    variable_value::Number(n) => {
                        if ty != Type::Integer {
                            panic!("Type mismatch in var: compile_var")
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
                                    panic!("Type mismatch in var: compile_var")
                                };
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),false);
                                let n = self.cast_int(n);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),n)
                            },
                            List::boolean(b) => {
                                if ty != Type::boolean {
                                    panic!("Type mismatch in var: compile_var")
                                };
                                let ptr = self.allocate_pointer(*functionname.clone(),*na.clone(),true);
                                let b = self.cast_bool(b);
                                self.insert_var(*functionname.clone(), *na.clone(), ptr.clone());
                                self.builder.build_store(ptr.clone(),b)
                            },
                            List::var(v) => {
                                let varval = self.compile_var(functionname.clone(),v);
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
                            _ => return self.cast_int(0),
                        }
                    },
                };
                return self.cast_int(0);
            },
            variable::name(v)=>{ // Access local var with same name and return it.
                let ptr = self.get_var(*v.clone());
                return self.builder.build_load(*ptr,&v.clone()).into_int_value();
            },
        };
    }

    fn compile_if(&mut self, functionname: Box<String>, if_e: if_enum) {
        let (ifst, if_body) = match if_e{
            if_enum::condition(v,w)=>(v,w)
        };
        let temp = self.compile_cons(unbox(functionname.clone()), ifst, op::greater, Box::new(List::Num(0)));
        if (temp == (self.cast_int(1))) {
            self.compile_function_elements(functionname.clone(), unbox(if_body));
        }
    }

    fn compile_while(&mut self, functionname: Box<String>, while_e: while_enum) {
        let (while_statement, while_body) =  match while_e{
            while_enum::condition(v,w)=>(v,w),
        };
        let temp = self.compile_cons(unbox(functionname.clone()), while_statement.clone(), op::greater, Box::new(List::Num(0)));
        while (temp == (self.cast_int(1))) {
            self.compile_function_elements(functionname.clone(), unbox(while_body.clone()));
        }
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

    fn compile_function_elements(&mut self, functionname: Box<String>, fe: function_elements) -> InstructionValue<'ctx>{
        match fe {
            function_elements::ele_list(v,w)=>{
                let ele1: function_elements = unbox(v);
                let ele2: function_elements = unbox(w);
                let res1 = self.compile_function_elements(functionname.clone(), ele1);
                let res2 = self.compile_function_elements(functionname.clone(), ele2);
            },
            function_elements::boxs(v)=>{
                let box_cont: variable = unbox(v);
                self.compile_var(functionname, box_cont);
            },
            function_elements::if_box(v)=>{
                let box_cont= unbox(v);
                self.compile_if(functionname, box_cont);
            },
            function_elements::List(v)=>{
                self.compile_list(unbox(functionname), v);
            },
            function_elements::function(v)=>{
                self.compile_function(unbox(functionname), v)
            },
            function_elements::variable(v)=>{
                self.compile_var(functionname, v);
            },
            function_elements::if_enum(v)=>{
                self.compile_if(functionname, v);
            },
            function_elements::while_enum(v) => {
                self.compile_while(functionname, v);
            },
            function_elements::return_val(v) => {
                return self.compile_return(functionname,v);
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
                let leftSide = self.compile_function_arguments_call_declare(functionname.clone(),a1);
                let rightSide = self.compile_function_arguments_call_declare(functionname.clone(),a2);
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