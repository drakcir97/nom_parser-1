<<<<<<< HEAD
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
use std::collections::HashMap;

use crate::enums::*;

use crate::enums::List::{func, var, Cons, Num};
use crate::enums::op::{add, div, mult, res, sub, wrong};
use crate::enums::variable_value::{boxs, Boolean, Nil, Number};
use crate::enums::variable::{name, parameters};

struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,

    mut state: HashMap<String, hashstate> = HashMap::new(),

    mut idmap: HashMap<i32,i32> = HashMap::new(),
    mut addressmap: HashMap<i32, hashdata> = HashMap::new(), 

    mut currentid: i32 = 1,
}

impl<'a, 'ctx> CodeGen<'a, 'ctx> {
    compile_list(&mut self, na: String, ls: List) -> List<'ctx> {
        match ls{
            List::paran(v) => {self.compile_list(functionname.clone(), unbox(v))},
            List::Cons(v,w,x) => {self.compile_cons(functionname.clone(),v,w,x)},
            List::Num(v) => {return ls},
            List::boolean(v)=>{return ls},
            List::func(fu) => {},
            List::var(v) => {self.compile_var(Box::new(functionname.clone()), v)},
            _ => panic!("Something went wrong: execute_List"),
        };
    }

    functionDeclare(&mut self, ls: List) {
        match ls {
            List::func(f) => {
                match f.clone() {
                    function::parameters_def(na,ar,ty,ele) => {
                        let temp: Vec<hashvariable> = Vec::new();
                        let tempstring: String = unbox(na.clone());
                        self.state.insert(tempstring,hashstate::state(Box::new(functionstate::Declared),Box::new(temp),Box::new(f.clone()),-1),state);
                    },
                    _ => (),
                };
            },
            _ => (),// Do nothing
        }
    }

    //CHange return statements to use builder istead, not needed to chech for logic ourselves.
    compile_cons(&mut self, functionname: String, l: Box<List>, oper: op ,r:  Box<List>) -> List<'ctx> {
        let expr = match oper{
            op::add => {
                let llist: List = self.compile_list(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
                let rlist: List = self.compile_list(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
                let vall = match llist {
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                let valr = match rlist {
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                return List::Num(vall+valr);
            },
    
            op::sub => {
                let llist: List = self.compile_list(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
                let rlist: List = self.compile_list(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
                let vall = match llist {
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                let valr = match rlist {
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                return List::Num(vall-valr);
            },
    
            op::div => {
                let llist: List = self.compile_list(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
                let rlist: List = self.compile_list(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
                let vall = match llist {
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                let valr = match rlist {
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                return List::Num(vall/valr);
            },
    
            op::res => {
                let llist: List = self.compile_list(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
                let rlist: List = self.compile_list(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
                let vall = match llist {
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                let valr = match rlist {
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                return List::Num(vall%valr);
            },
            op::mult => {
                let llist: List = self.compile_list(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
                let rlist: List = self.compile_list(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
                let vall = match llist {
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                let valr = match rlist {
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                return List::Num(vall*valr);
            },
            op::less => {
                let llist: List = self.compile_list(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
                let rlist: List = self.compile_list(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
                let vall = match llist {
                    List::boolean(n) => {
                        if n == true {
                            1
                        } else {
                            0
                        }
                    },
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                let valr = match rlist {
                    List::boolean(n) => {
                        if n == true {
                            1
                        } else {
                            0
                        }
                    },
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                if vall < valr {
                    return List::boolean(true);
                }
                return List::boolean(false);
            },
            op::greater => {
                let llist: List = self.compile_list(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
                let rlist: List = self.compile_list(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
                let vall = match llist {
                    List::boolean(n) => {
                        if n == true {
                            1
                        } else {
                            0
                        }
                    },
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                let valr = match rlist {
                    List::boolean(n) => {
                        if n == true {
                            1
                        } else {
                            0
                        }
                    },
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                if vall > valr {
                    return List::boolean(true);
                }
                return List::boolean(false);
            },
            op::lessEqual => {
                let llist: List = self.compile_list(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
                let rlist: List = self.compile_list(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
                let vall = match llist {
                    List::boolean(n) => {
                        if n == true {
                            1
                        } else {
                            0
                        }
                    },
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                let valr = match rlist {
                    List::boolean(n) => {
                        if n == true {
                            1
                        } else {
                            0
                        }
                    },
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                if vall <= valr {
                    return List::boolean(true);
                }
                return List::boolean(false);
            },
            op::greatEqual => {
                let llist: List = self.compile_list(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
                let rlist: List = self.compile_list(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
                let vall = match llist {
                    List::boolean(n) => {
                        if n == true {
                            1
                        } else {
                            0
                        }
                    },
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                let valr = match rlist {
                    List::boolean(n) => {
                        if n == true {
                            1
                        } else {
                            0
                        }
                    },
                    List::Num(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                if vall >= valr {
                    return List::boolean(true);
                }
                return List::boolean(false);
            },
            op::and => {
                let llist: List = self.compile_list(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
                let rlist: List = self.compile_list(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
                let vall = match llist {
                    List::boolean(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                let valr = match rlist {
                    List::boolean(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                return List::boolean(vall && valr);
            },
            op::or => {
                let llist: List = self.compile_list(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
                let rlist: List = self.compile_list(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
                let vall = match llist {
                    List::boolean(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                let valr = match rlist {
                    List::boolean(n) => {n},
                    _ => {panic!("Type mismatch : cons_execute")},
                };
                return List::boolean(vall || valr);
            },
            
    
            _ => panic!("Operand not supported: cons_execute")
        };
    }

    compile_function(&mut self,functionname: String, func_var: function) {
        match func_var{
            function::parameters_def(na,_m,_n,_o)=>{
                if unbox(na.clone()) == "main" { //Special case for main. Calls in when we find define in code. Ensures that it is always called.
                    let w = function_arguments_call::variable(Box::new(variable::name(Box::new("".to_string()))));
                    self.compile_function_arguments_call_execute(na.clone(), na.clone(), Box::new(w));
                }
            }, //Do nothing on define, since this is handled in functionDeclare, except for main.
            function::parameters_call(v,w)=>{
                self.compile_function_arguments_call_execute(v.clone(), Box::new(functionname.clone()), w);
            },
        };
    }

    //Removed check in memory so now only one match for everything, simplify memory to just include addressmap. Should make fetching easier too.
    compile_var(&mut self, functionname: Box<String>, variable_var: variable) -> List<'ctx> {
        match variable_var{
            variable::parameters(na,ty,val) => {
                match unbox(val) {
                    variable_value::Boolean(b) => {
                        if ty != Type::boolean {
                            panic!("Type mismatch in var: var_execute")
                        };
                        let temp = hashdata::valuebool(b);
                        let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                        let temp2 = hashvariable::var(unbox(na),addressOfTemp);
                        let resadd = addLocalVariable(unbox(functionname), temp2, state);
                        if resadd == false {
                            return panic!("Adding local variable failed: var_execute");
                        }
                    },
                    variable_value::Number(n) => {
                        if ty != Type::Integer {
                            panic!("Type mismatch in var: var_execute")
                        };
                        let temp = hashdata::valuei32(n);
                        let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                        let temp2 = hashvariable::var(unbox(na),addressOfTemp);
                        let resadd = addLocalVariable(unbox(functionname), temp2, state);
                        if resadd == false {
                            return panic!("Adding local variable failed: var_execute");
                        }
                    },
                    variable_value::boxs(b) => {
                        let ls = unbox(b);
                        match ls.clone() {
                            List::Num(n) => {
                                if ty != Type::Integer {
                                    panic!("Type mismatch in var: var_execute")
                                };
                                let temp = hashdata::valuei32(n);
                                let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                                let temp2 = hashvariable::var(unbox(na),addressOfTemp);
                                let resadd = addLocalVariable(unbox(functionname), temp2, state);
                                if resadd == false {
                                    return panic!("Adding local variable failed: var_execute");
                                }
                            },
                            List::boolean(b) => {
                                if ty != Type::boolean {
                                    panic!("Type mismatch in var: var_execute")
                                };
                                let temp = hashdata::valuebool(b);
                                let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                                let temp2 = hashvariable::var(unbox(na),addressOfTemp);
                                let resadd = addLocalVariable(unbox(functionname), temp2, state);
                                if resadd == false {
                                    return panic!("Adding local variable failed: var_execute");
                                }
                            },
                            List::var(v) => {
                                let varval = var_execute(functionname.clone(),v , state, idmap, addressmap, currentid);
                                match varval.clone() {
                                    List::Num(n) => {
                                        if ty != Type::Integer {
                                            panic!("Type mismatch in var: var_execute")
                                        };
                                        let temp = hashdata::valuei32(n);
                                        let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                                        let temp2 = hashvariable::var(unbox(na),addressOfTemp);
                                        let resadd = addLocalVariable(unbox(functionname), temp2, state);
                                        if resadd == false {
                                            return panic!("Adding local variable failed: var_execute");
                                        }
                                    },
                                    List::boolean(b) => {
                                        if ty != Type::boolean {
                                            panic!("Type mismatch in var: var_execute")
                                        };
                                        let temp = hashdata::valuebool(b);
                                        let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                                        let temp2 = hashvariable::var(unbox(na),addressOfTemp);
                                        let resadd = addLocalVariable(unbox(functionname), temp2, state);
                                        if resadd == false {
                                            return panic!("Adding local variable failed: var_execute");
                                        }
                                    },
                                    _ => (),
                                };
                            },
                            List::Cons(lli,op,rli) => {
                                let val = cons_execute(unbox(functionname.clone()), lli, op, rli, state, idmap, addressmap, currentid);
                                match val.clone() {
                                    List::Num(n) => {
                                        if ty != Type::Integer {
                                            panic!("Type mismatch in var: var_execute")
                                        };
                                        let temp = hashdata::valuei32(n);
                                        let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                                        let temp2 = hashvariable::var(unbox(na),addressOfTemp);
                                        let resadd = addLocalVariable(unbox(functionname), temp2, state);
                                        if resadd == false {
                                            return panic!("Adding local variable failed: var_execute");
                                        }
                                    },
                                    List::boolean(b) => {
                                        if ty != Type::boolean {
                                            panic!("Type mismatch in var: var_execute")
                                        };
                                        let temp = hashdata::valuebool(b);
                                        let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                                        let temp2 = hashvariable::var(unbox(na),addressOfTemp);
                                        let resadd = addLocalVariable(unbox(functionname), temp2, state);
                                        if resadd == false {
                                            return panic!("Adding local variable failed: var_execute");
                                        }
                                    },
                                    _ => (),
                                };
                            },
                            List::func(fu) => {
                                let val = execute_List(unbox(functionname.clone()), ls.clone(), state, idmap, addressmap, currentid);
                                match val.clone() {
                                    List::Num(n) => {
                                        if ty != Type::Integer {
                                            panic!("Type mismatch in var: var_execute")
                                        };
                                        let temp = hashdata::valuei32(n);
                                        let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                                        let temp2 = hashvariable::var(unbox(na),addressOfTemp);
                                        let resadd = addLocalVariable(unbox(functionname), temp2, state);
                                        if resadd == false {
                                            return panic!("Adding local variable failed: var_execute");
                                        }
                                    },
                                    List::boolean(b) => {
                                        if ty != Type::boolean {
                                            panic!("Type mismatch in var: var_execute")
                                        };
                                        let temp = hashdata::valuebool(b);
                                        let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                                        let temp2 = hashvariable::var(unbox(na),addressOfTemp);
                                        let resadd = addLocalVariable(unbox(functionname), temp2, state);
                                        if resadd == false {
                                            return panic!("Adding local variable failed: var_execute");
                                        }
                                    },
                                    _ => (),
                                };
                            },
                            _ => (),
                        };
                    },
                };
                return List::Num(0);
            },
            variable::name(v)=>{ // Access local var with same name and return it.
                let lvar = getLocalVariable(unbox(functionname), unbox(v.clone()), state);
                match lvar {
                    hashvariable::var(n,a) => {
                        let mem = getFromMemory(a,addressmap);
                        let value = getFromAddressHashdata(mem, addressmap);
                        match value {
                            hashdata::valuei32(va) => {return List::Num(va);},
                            hashdata::valuebool(va) => {return List::boolean(va);},
                            _ => {return List::Num(0);},
                        }
    
                    },
                    _ => panic!("Name given {:?} is incorrect, not in local variables: to var_execute", unbox(v.clone())),
                };
                
                
            },
        };
    }

    compile_if(&mut self, functionname: Box<String>, if_e: if_enum) {

    }

    compile_while(&mut self, functionname: Box<String>, while_e: while_enum) {

    }

    compile_return(&mut self, functionname: Box<String>, var_val: variable_value) {

    }

    compile_function_elements(&mut self, functionname: Box<String>, fe: function_elements) {

    }

    compile_function_arguments_call_execute(&mut self, functionname: Box<String>, oldfunctionname: Box<String>, args: Box<function_arguments_call>) {

    }

    compile_function_arguments_call_declare(&mut self, functionname: Box<String>, oldfunctionname: Box<String>, args: Box<function_arguments_call>, fuargs: Box<function_arguments>) {

    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let context = Context::create();
    let module = context.create_module("sum");
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;
    let codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
        execution_engine,
    };

    let sum = codegen.jit_compile_sum().ok_or("Unable to JIT compile `sum`")?;

    let x = 1u64;
    let y = 2u64;
    let z = 3u64;

    unsafe {
        println!("{} + {} + {} = {}", x, y, z, sum.call(x, y, z));
        assert_eq!(sum.call(x, y, z), x + y + z);
    }

    Ok(())
}