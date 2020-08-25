extern crate nom;
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_till, take_until, take_while1, take_while_m_n},
    //sequence::tuple,
    character::complete::{char, digit1, multispace0},
    character::is_alphanumeric,
    combinator::map,
    combinator::map_res,
    multi::many0,
    // complete::take,
    sequence::{delimited, preceded, terminated, tuple},
    //take_until,
    //alt,
    switch,
    IResult,
};

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

//Specific enum for typechecker, used to check return values and types for input.
#[derive(Debug, Clone, PartialEq)]
enum hashchecker {
    st(Box<Type>, Box<function_arguments>),
    Nil,
}

//Sets return type of a function in hashmap.
fn setReturnType(na: String, ty: Box<Type>, check: &mut HashMap<String, hashchecker>) {
    let check2 = &mut check.clone();
    let ovar = match getType(na.clone(), check2) {
        hashchecker::st(ty,ve) => {ve},
        _ => panic!("Type incorrect: setReturnType"),
    };
    check.insert(na,hashchecker::st(ty,ovar.clone()));
}

//Sets return type of a function in hashmap.
fn setVarType(na: String, ty: Box<function_arguments>, check: &mut HashMap<String, hashchecker>) {
    let check2 = &mut check.clone();
    let oret = match getType(na.clone(), check2) {
        hashchecker::st(ty,ve) => {ty},
        _ => panic!("Type incorrect: setVarType"),
    };
    check.insert(na,hashchecker::st(oret.clone(),ty));
}

//Gets return type of a function in hashmap.
fn getType(na: String, check: &mut HashMap<String, hashchecker>) -> &hashchecker {
    if check.contains_key(&na) {
        let result = check.get(&na);
        match result {
            Some(val) => return val,
            None => panic!("Get type failed!: getType"),
        }
    } else {
        panic!("No such type exists!: getType");
    }
}

//Declares types for function input.
fn functionVarDeclare(ls: List, check: &mut HashMap<String, hashchecker>) {
    match ls {
        List::func(f) => {
            match f {
                function::parameters_def(na,args,ty,ele) => {
                    setVarType(unbox(na), args, check);
                },
                _ => (), 
            }
        },
        _ => (), 
    } 
}

pub fn typechecker(pg : Program) {
    let (nm, statements) = match pg {
        Program::pgr(v,w) => (v,w),
    };

    let mut check: HashMap<String, hashchecker> = HashMap::new();

    let mut deciter = statements.iter(); 

    for stmt in deciter {
        functionVarDeclare(stmt.clone(), &mut check); //Loop through and declares the functions and only sets types for call.
    }                                                 //Return is set later, not needed to have it here.

    let mut iter = statements.iter(); 

    for stmt in iter {
        listChecker("".to_string(),stmt.clone(), &mut check); //Loop through and check all types.
    }
}

//Takes a struct and checks if the operand matches the left and right hand side. That is, if you try to add a integer and bool it will panic.
//Also checks for variables to see that the assignment is the correct type.
fn listChecker(na: String, ls: List, check: &mut HashMap<String, hashchecker>) -> Type {
    match ls {
        List::Num(n) => Type::Integer,
        List::boolean(b) => Type::boolean,
        List::Cons(l,o,r) => {
            return consChecker(na, l, o, r, check);               
        },
        List::var(v) => {
            return varChecker(na.clone(), v, check);    
        },
        List::func(f) => {
            return functionChecker(na.clone(), f, check);
        },
        List::paran(p) => {
            return listChecker(na.clone(), unbox(p), check);
        },
        _ => Type::unknown(0), //Added to be able to test 1/11-19, should skip here /Rickard
    }    
}

//Checks that variable declarations are correct type. Need to add ability to check for variable names.
fn varChecker(na: String, v: variable, check: &mut HashMap<String, hashchecker>) -> Type {
    match v {
        variable::parameters(_n,ty,value) => {
            match ty {
                Type::Integer => {
                    match *value {
                        variable_value::Number(n) => {
                            return Type::Integer;
                        },
                        variable_value::boxs(b) => {
                            let typ = listChecker(na.clone(), unbox(b), check);
                            if typ != Type::Integer {
                                return panic!("Incorrect assignment: typechecker");
                            }
                        },
                        _ => return panic!("Incorrect assignment: typechecker"),
                    };
                    return Type::unknown(0);
                },
                Type::boolean => {
                    match *value {
                        variable_value::Boolean(b) => {
                            return Type::boolean;
                        },
                        variable_value::boxs(b) => {
                            let typ = listChecker(na.clone(),unbox(b), check);
                            if typ != Type::Integer {
                                return panic!("Incorrect assignment: typechecker")
                            }
                        },
                        _ => return panic!("Incorrect assignment: typechecker"),
                    };
                    return Type::unknown(0);
                },
                _ => Type::unknown(0),
            };
            return Type::unknown(0);
        },
        _ => Type::unknown(0), //Added to be able to test 1/11-19 /Rickard
    }
}

//Checks that the type is correct and matches operand.
fn consChecker(na: String,l: Box<List>, o: op, r: Box<List>, check: &mut HashMap<String, hashchecker>) -> Type {
    let ls = listChecker(na.clone(),*l, check);
    let rs = listChecker(na.clone(),*r, check);
    match o {
        op::add => {
            match ls {
                Type::Integer => {
                    match rs {
                        Type::Integer => return Type::Integer,
                        _ => panic!("Incorrect types: typechecker"),
                    };
                },
                _ => panic!("Incorrect types: typechecker"),

            };
        },
        op::sub => {
            match ls {
                Type::Integer => {
                    match rs {
                        Type::Integer => return Type::Integer,
                        _ => panic!("Incorrect types: typechecker"),
                    };
                },
                _ => panic!("Incorrect types: typechecker"),

            };
        },
        op::div => {
            match ls {
                Type::Integer => {
                    match rs {
                        Type::Integer => return Type::Integer,
                        _ => panic!("Incorrect types: typechecker"),
                    };
                },
                _ => panic!("Incorrect types: typechecker"),

            };
        },
        op::mult => {
            match ls {
                Type::Integer => {
                    match rs {
                        Type::Integer => return Type::Integer,
                        _ => panic!("Incorrect types: typechecker"),
                    };
                },
                _ => panic!("Incorrect types: typechecker"),

            };
        },
        op::res => {
            match ls {
                Type::Integer => {
                    match rs {
                        Type::Integer => return Type::Integer,
                        _ => panic!("Incorrect types: typechecker"),
                    };
                },
                _ => panic!("Incorrect types: typechecker"),

            };
        },
        op::and => {
            match ls {
                Type::boolean => {
                    match rs {
                        Type::boolean => return Type::boolean,
                        _ => panic!("Incorrect types: typechecker"),
                    };
                },
                _ => panic!("Incorrect types: typechecker"),

            };
        },
        op::or => {
            match ls {
                Type::boolean => {
                    match rs {
                        Type::boolean => return Type::boolean,
                        _ => panic!("Incorrect types: typechecker"),
                    };
                },
                _ => panic!("Incorrect types: typechecker"),

            };
        },
        _ => panic!("Incorrect operand : typechecker"),
    }; 
}

//Checks functions.
fn functionChecker(na: String, fu: function, check: &mut HashMap<String, hashchecker>) -> Type {
    match fu {
        function::parameters_call(_na,args) => { 
            let typearg = match getType(na.clone(), check) {
                hashchecker::st(ty,ve) => {ve},
                _ => panic!(),
            };
            return function_a_callChecker(na, unbox(args), unbox(typearg.clone()), check);
        },
        function::parameters_def(funame,args,ty,ele) => {
            setReturnType(unbox(funame.clone()), Box::new(ty.clone()), check); //Add type to hashmap for later use in returnChecker
            function_eChecker(unbox(funame.clone()), unbox(ele), check);
            return ty;
        },
    }
}

//Checks the different function elements. Main function for function elements.
fn function_eChecker(na: String, fe: function_elements, check: &mut HashMap<String, hashchecker>) {
    match fe {
        function_elements::ele_list(v,w)=>{
            let _res1 = function_eChecker(na.clone(), unbox(v.clone()), check);
            let _res2 = function_eChecker(na.clone(), unbox(w.clone()), check);
        },
        function_elements::boxs(v)=>{
            varChecker(na.clone(), unbox(v), check); 
        },
        function_elements::if_box(v)=>{
            ifChecker(na.clone(), unbox(v), check);
        },
        function_elements::List(v)=>{
            listChecker(na.clone(), v, check);
        },
        function_elements::function(v)=>{
            functionChecker(na.clone(), v, check);
        },
        function_elements::variable(v)=>{
            varChecker(na.clone(), v, check); 
        },
        function_elements::if_enum(v)=>{
            ifChecker(na.clone(), v, check);
        },
        function_elements::while_enum(v) => {
            whileChecker(na.clone(), v, check)
        },
        function_elements::return_val(v) => {
            returnChecker(na.clone(), v, check);
        },
    }
}

//Empty for now, could be used later to check that the inputs to functions have the correct type. But to accomplish this, a new function
//will be needed to "declare" the function to a hashmap like in the interpreter.
fn function_a_Checker(na: String, fa: function_arguments, check: &mut HashMap<String, hashchecker>) {
    
}

//Empty for now, could be used later to check that the inputs to functions have the correct type. But to accomplish this, a new function
//will be needed to "declare" the function to a hashmap like in the interpreter.
fn function_a_callChecker(na: String, fa: function_arguments_call, ve: function_arguments, check: &mut HashMap<String, hashchecker>) -> Type {
    match fa {
        function_arguments_call::arg_call_list(lfa, rfa) => {

        }
        function_arguments_call::variable(va) => {

        }
        function_arguments_call::bx(b) => {

        }
        function_arguments_call::function(fu) => {

        }
    }
    return Type::unknown(0);
}

//Checks if enums, including the condition.
 fn ifChecker(na: String, i: if_enum, check: &mut HashMap<String, hashchecker>) {
    match i {
        if_enum::condition(v ,w) =>{
            listChecker(na.clone(), unbox(v), check);
            function_eChecker(na.clone(), unbox(w), check); 
        }
    }
 }

//Checks while enums, including the condition.
fn whileChecker(na: String, wh: while_enum, check: &mut HashMap<String, hashchecker>) {
    match wh {
        while_enum::condition(v ,w) =>{
            listChecker(na.clone(), unbox(v), check);
            function_eChecker(na.clone(), unbox(w), check); 
        }
    }
}

//Checks that the return type matches the function.
fn returnChecker(na: String, v: variable_value, check: &mut HashMap<String, hashchecker>) {
    let hcheck = getType(na.clone(), check);
    let rettype = match hcheck.clone() {
        hashchecker::st(bo,_) => {unbox(bo.clone())},
        _ => {Type::unknown(0)},
    };
    let realtype = match v {
        variable_value::variable(v) => {varChecker(na.clone(), unbox(v), check)},
        variable_value::boxs(l) => {listChecker(na.clone(), unbox(l), check)},
        variable_value::Number(n) => {Type::Integer},
        variable_value::Boolean(n) => {Type::boolean},
        _ => {panic!("returnChecker")},
    };
    if rettype != realtype {
        panic!("Type mismatch returnChecker");
    };
}

fn unbox<T>(value: Box<T>) -> T {
    *value
}


fn main() {}