#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unreachable_code)]
#![allow(non_camel_case)]
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
    st(Box<Type>, Box<Vec<hashvariable>>, Box<function_arguments>),
    Nil,
}

enum hashvarchecker {
    va(Box<Type>),
    Nil,
}

//Sets return type of a function in hashmap.
fn setReturnType(na: String, ty: Box<Type>, check: &mut HashMap<String, hashchecker>) {
    let check2 = &mut check.clone();
    let (ve,fa) = match getType(na.clone(), check2) {
        hashchecker::st(ty,ve,fa) => {(ve,fa)},
        _ => panic!("Type incorrect for function {:?} {:?}: setReturnType", na.clone(),unbox(ty.clone())),
    };
    check.insert(na,hashchecker::st(ty,ve.clone(),fa.clone()));
}

//Sets local var types of a function in hashmap.
fn setVarType(na: String, ve: Box<Vec<hashvariable>>, check: &mut HashMap<String, hashchecker>) {
    let check2 = &mut check.clone();
    let (ty,fa) = match getType(na.clone(), check2) {
        hashchecker::st(ty,ve,fa) => {(ty,fa)},
        _ => panic!("Type incorrect for function {:?}: setVarType", na.clone()),
    };
    check.insert(na,hashchecker::st(ty.clone(),ve,fa.clone()));
}

//Sets function_arguments of a function in hashmap.
fn setInpType(na: String, fa: Box<function_arguments>, check: &mut HashMap<String, hashchecker>) {
    let check2 = &mut check.clone();
    let (ty,ve) = match getType(na.clone(), check2) {
        hashchecker::st(ty,ve,fa) => {(ty,ve)},
        _ => panic!("Type incorrect for function {:?}: setInpType", na.clone()),
    };
    check.insert(na,hashchecker::st(ty.clone(),ve.clone(),fa));
}

//Gets return type of a function in hashmap.
fn getType(na: String, check: &mut HashMap<String, hashchecker>) -> &hashchecker {
    if check.contains_key(&na) {
        let result = check.get(&na);
        match result {
            Some(val) => return val,
            None => panic!("Get return type failed for function {:?}: getType", na.clone()),
        }
    } else {
        panic!("No such return type exists for function {:?}: getType", na.clone());
    }
}

fn setVar(id: i32, inp: Box<Type>, varcheck: &mut HashMap<i32, hashvarchecker>, currentid: &mut i32) -> i32 {
    if id == 0 {
        let retid = *currentid;
        varcheck.insert(retid, hashvarchecker::va(inp));
        *currentid += 1;
        return retid;
    } else {
        varcheck.insert(id, hashvarchecker::va(inp));
        return id;
    }
    
}

fn getVar(id: i32, varcheck: &mut HashMap<i32, hashvarchecker>) -> &hashvarchecker {
    if varcheck.contains_key(&id) {
        let result = varcheck.get(&id);
        match result {
            Some(val) => return val,
            None => panic!("Get type failed for id {:?}: getVar", id.clone()),
        }
    } else {
        panic!("No such type exists for id {:?}: getVar", id.clone());
    }
}

fn createLocalVar(functionname: String, varname: String, vartype: Type, check: &mut HashMap<String, hashchecker>, varcheck: &mut HashMap<i32, hashvarchecker>, currentid: &mut i32) {
    let st = getType(functionname.clone(), check);
    let varvec = match st.clone() {
        hashchecker::st(ty,ve,fa) => {unbox(ve)},
        _ => panic!(),
    };
    let varid = setVar(0, Box::new(vartype.clone()),varcheck,currentid);
    let mut variablevector = varvec.clone();
    variablevector.push(hashvariable::var(varname.clone(),varid.clone()));
    setVarType(functionname.clone(),Box::new(variablevector),check);
    
}

//Declares types for function input.
fn functionVarDeclare(ls: List, check: &mut HashMap<String, hashchecker>) {
    match ls {
        List::func(f) => {
            match f {
                function::parameters_def(na,args,ty,ele) => {
                    let tevec: Vec<hashvariable> = Vec::new();
                    check.insert(unbox(na.clone()),hashchecker::st(Box::new(ty),Box::new(tevec.clone()),args.clone()));
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
    let mut varcheck: HashMap<i32, hashvarchecker> = HashMap::new();
    let mut currentid: i32 = 1;

    let mut deciter = statements.iter(); 

    for stmt in deciter {
        functionVarDeclare(stmt.clone(), &mut check); //Loop through and declares the functions and only sets types for call.
    }                                                 //Return is set later, not needed to have it here.

    let mut iter = statements.iter(); 

    for stmt in iter {
        listChecker("".to_string(),stmt.clone(), &mut check, &mut varcheck, &mut currentid); //Loop through and check all types.
    }
    println!("{}", unbox(nm)+" passed typechecker!")
}

//Takes a struct and checks if the operand matches the left and right hand side. That is, if you try to add a integer and bool it will panic.
//Also checks for variables to see that the assignment is the correct type.
fn listChecker(na: String, ls: List, check: &mut HashMap<String, hashchecker>, varcheck: &mut HashMap<i32, hashvarchecker>, currentid: &mut i32) -> Type {
    match ls {
        List::Num(n) => Type::Integer,
        List::boolean(b) => Type::boolean,
        List::Cons(l,o,r) => {
            return consChecker(na, l, o, r, check, varcheck, currentid);               
        },
        List::var(v) => {
            return varChecker(na.clone(), v, check, varcheck, currentid);    
        },
        List::func(f) => {
            return functionChecker(na.clone(), f, check, varcheck, currentid);
        },
        List::paran(p) => {
            return listChecker(na.clone(), unbox(p), check, varcheck, currentid);
        },
        _ => Type::unknown(0), //Added to be able to test 1/11-19, should skip here /Rickard
    }    
}

//Checks that variable declarations are correct type. Need to add ability to check for variable names.
fn varChecker(na: String, v: variable, check: &mut HashMap<String, hashchecker>, varcheck: &mut HashMap<i32, hashvarchecker>, currentid: &mut i32) -> Type {
    match v {
        variable::parameters(varname,ty,value) => {
            match ty {
                Type::Integer => {
                    let valtype = varValueChecker(na.clone(),unbox(value),check,varcheck,currentid);
                    if valtype != Type::Integer {
                        return panic!("Incorrect assignment {:?}: typechecker",na.clone())
                    }
                    createLocalVar(na.clone(), unbox(varname), Type::Integer, check, varcheck, currentid);
                    return Type::Integer;
                },
                Type::boolean => {
                    let valtype = varValueChecker(na.clone(),unbox(value),check,varcheck,currentid);
                    if valtype != Type::boolean {
                        return panic!("Incorrect assignment {:?}: typechecker",na.clone())
                    }
                    createLocalVar(na.clone(), unbox(varname), Type::boolean, check, varcheck, currentid);
                    return Type::Integer;
                },
                _ => Type::unknown(0),
            };
            return Type::unknown(0);
        },
        variable::name(fna) => {
            match getType(na.clone(), check).clone() {
                hashchecker::st(ty,ve,fa) => {
                    let ubve = unbox(ve);
                    let iter = ubve.iter();
                    for lv in iter {
                        let (lona,loid) = match lv.clone() {
                            hashvariable::var(n,id) => {(n,id)},
                            _ => panic!(),
                        };
                        if lona == unbox(fna.clone()) {
                            let lovar = getVar(loid, varcheck);
                            match lovar.clone() {
                                hashvarchecker::va(ty) => {
                                    return unbox(ty.clone());
                                },
                                _ => panic!(),
                            };
                        }
                    };
                    panic!("Local var not found {:?} for func {:?}: typechecker",unbox(fna.clone()),na.clone());
                },
                _ => panic!(),
            }
            return Type::unknown(0);
        },
        variable::assign(varname,val) => {
            match getType(na.clone(), check).clone() {
                hashchecker::st(ty,ve,fa) => {
                    let ubve = unbox(ve);
                    let iter = ubve.iter();
                    for lv in iter {
                        let (lona,loid) = match lv.clone() {
                            hashvariable::var(n,id) => {(n,id)},
                            _ => panic!(),
                        };
                        if lona == unbox(varname.clone()) {
                            let lovar = getVar(loid, varcheck);
                            let natype = match lovar.clone() {
                                hashvarchecker::va(ty) => {
                                    unbox(ty.clone())
                                },
                                _ => panic!(),
                            };
                            let valtype = varValueChecker(na.clone(),unbox(val),check,varcheck,currentid);
                            if natype != valtype {
                                return panic!("Incorrect type in assignment {:?} {:?}: typechecker",unbox(varname.clone()),na.clone())
                            }
                            return natype;

                        }
                    };
                    panic!("Local var not found {:?} {:?}: typechecker",unbox(varname.clone()),na.clone());
                },
                _ => panic!(),
            }
        }
        _ => Type::unknown(0), //Added to be able to test 1/11-19 /Rickard
    }
}

fn varValueChecker(na: String, v: variable_value, check: &mut HashMap<String, hashchecker>, varcheck: &mut HashMap<i32, hashvarchecker>, currentid: &mut i32) -> Type {
    match v {
        variable_value::variable(va) => {
            return varChecker(na,unbox(va),check,varcheck,currentid);
        },
        variable_value::boxs(boli) => {
            return listChecker(na,unbox(boli),check,varcheck,currentid);
        },
        variable_value::Number(nu) => {
            return Type::Integer;
        },
        variable_value::Boolean(bo) => {
            return Type::boolean;
        },
        _ => panic!("Incorrect type: varValueChecker"),
    }
}

//Checks that the type is correct and matches operand.
fn consChecker(na: String,l: Box<List>, o: op, r: Box<List>, check: &mut HashMap<String, hashchecker>, varcheck: &mut HashMap<i32, hashvarchecker>, currentid: &mut i32) -> Type {
    let ls = listChecker(na.clone(),*l, check, varcheck, currentid);
    let rs = listChecker(na.clone(),*r, check, varcheck, currentid);
    match o.clone() {
        op::add => {
            match ls.clone() {
                Type::Integer => {
                    match rs.clone() {
                        Type::Integer => return Type::Integer,
                        _ => panic!("Incorrect types {:?}, {:?}, {:?} for func {:?}: consChecker",ls.clone(),o.clone(),rs.clone(),na.clone()),
                    };
                },
                _ => panic!("Incorrect types {:?}, {:?}, {:?} for func {:?}: consChecker",ls.clone(),o.clone(),rs.clone(),na.clone()),

            };
        },
        op::sub => {
            match ls.clone() {
                Type::Integer => {
                    match rs.clone() {
                        Type::Integer => return Type::Integer,
                        _ => panic!("Incorrect types {:?}, {:?}, {:?} for func {:?}: consChecker",ls.clone(),o.clone(),rs.clone(),na.clone()),
                    };
                },
                _ => panic!("Incorrect types {:?}, {:?}, {:?} for func {:?}: consChecker",ls.clone(),o.clone(),rs.clone(),na.clone()),

            };
        },
        op::div => {
            match ls.clone() {
                Type::Integer => {
                    match rs.clone() {
                        Type::Integer => return Type::Integer,
                        _ => panic!("Incorrect types {:?}, {:?}, {:?} for func {:?}: consChecker",ls.clone(),o.clone(),rs.clone(),na.clone()),
                    };
                },
                _ => panic!("Incorrect types {:?}, {:?}, {:?} for func {:?}: consChecker",ls.clone(),o.clone(),rs.clone(),na.clone()),

            };
        },
        op::mult => {
            match ls.clone() {
                Type::Integer => {
                    match rs.clone() {
                        Type::Integer => return Type::Integer,
                        _ => panic!("Incorrect types {:?}, {:?}, {:?} for func {:?}: consChecker",ls.clone(),o.clone(),rs.clone(),na.clone()),
                    };
                },
                _ => panic!("Incorrect types {:?}, {:?}, {:?} for func {:?}: consChecker",ls.clone(),o.clone(),rs.clone(),na.clone()),

            };
        },
        op::res => {
            match ls.clone() {
                Type::Integer => {
                    match rs.clone() {
                        Type::Integer => return Type::Integer,
                        _ => panic!("Incorrect types {:?}, {:?}, {:?} for func {:?}: consChecker",ls.clone(),o.clone(),rs.clone(),na.clone()),
                    };
                },
                _ => panic!("Incorrect types {:?}, {:?}, {:?} for func {:?}: consChecker",ls.clone(),o.clone(),rs.clone(),na.clone()),

            };
        },
        op::and => {
            match ls.clone() {
                Type::boolean => {
                    match rs.clone() {
                        Type::boolean => return Type::boolean,
                        _ => panic!("Incorrect types {:?}, {:?}, {:?} for func {:?}: consChecker",ls.clone(),o.clone(),rs.clone(),na.clone()),
                    };
                },
                _ => panic!("Incorrect types {:?}, {:?}, {:?} for func {:?}: consChecker",ls.clone(),o.clone(),rs.clone(),na.clone()),

            };
        },
        op::or => {
            match ls.clone() {
                Type::boolean => {
                    match rs.clone() {
                        Type::boolean => return Type::boolean,
                        _ => panic!("Incorrect types {:?}, {:?}, {:?} for func {:?}: consChecker",ls.clone(),o.clone(),rs.clone(),na.clone()),
                    };
                },
                _ => panic!("Incorrect types {:?}, {:?}, {:?} for func {:?}: consChecker",ls.clone(),o.clone(),rs.clone(),na.clone()),

            };
        },
        op::less => {
            return Type::boolean; //No need to check types since interpreter casts them anyway.
        },
        op::greater => {
            return Type::boolean;
        },
        op::equal => {
            return Type::boolean;
        },
        op::lessEqual => {
            return Type::boolean;
        },
        op::greatEqual => {
            return Type::boolean;
        },
        _ => panic!("Incorrect operand : typechecker"),
    }; 
}

//Checks functions.
fn functionChecker(na: String, fu: function, check: &mut HashMap<String, hashchecker>, varcheck: &mut HashMap<i32, hashvarchecker>, currentid: &mut i32) -> Type {
    match fu {
        function::parameters_call(nna,args) => { 
            let typearg = match getType(unbox(nna.clone()), check) {
                hashchecker::st(ty,_,ve) => {ve},
                _ => panic!(),
            };
            function_a_callChecker(unbox(nna.clone()), na.clone(), unbox(args), unbox(typearg.clone()), check, varcheck, currentid);
            let ty = getType(unbox(nna.clone()), check);
            let ret = match ty.clone() {
                hashchecker::st(ty,_,_) => {
                    unbox(ty)
                },
                _ => panic!(),
            };
            return ret;
        },
        function::parameters_def(funame,args,ty,ele) => {
            setReturnType(unbox(funame.clone()), Box::new(ty.clone()), check); //Add type to hashmap for later use in returnChecker
            function_eChecker(unbox(funame.clone()), unbox(ele), check, varcheck, currentid);
            return ty;
        },
    }
}

//Checks the different function elements. Main function for function elements.
fn function_eChecker(na: String, fe: function_elements, check: &mut HashMap<String, hashchecker>, varcheck: &mut HashMap<i32, hashvarchecker>, currentid: &mut i32) {
    match fe {
        function_elements::ele_list(v,w)=>{
            let _res1 = function_eChecker(na.clone(), unbox(v.clone()), check, varcheck, currentid);
            let _res2 = function_eChecker(na.clone(), unbox(w.clone()), check, varcheck, currentid);
        },
        function_elements::boxs(v)=>{
            varChecker(na.clone(), unbox(v), check, varcheck, currentid); 
        },
        function_elements::if_box(v)=>{
            ifChecker(na.clone(), unbox(v), check, varcheck, currentid);
        },
        function_elements::List(v)=>{
            listChecker(na.clone(), v, check, varcheck, currentid);
        },
        function_elements::function(v)=>{
            functionChecker(na.clone(), v, check, varcheck, currentid);
        },
        function_elements::variable(v)=>{
            varChecker(na.clone(), v, check, varcheck, currentid); 
        },
        function_elements::if_enum(v)=>{
            ifChecker(na.clone(), v, check, varcheck, currentid);
        },
        function_elements::while_enum(v) => {
            whileChecker(na.clone(), v, check, varcheck, currentid)
        },
        function_elements::return_val(v) => {
            returnChecker(na.clone(), v, check, varcheck, currentid);
        },
    }
}

//Empty for now, could be used later to check that the inputs to functions have the correct type. But to accomplish this, a new function
//will be needed to "declare" the function to a hashmap like in the interpreter.
fn function_a_Checker(na: String, fa: function_arguments, check: &mut HashMap<String, hashchecker>, varcheck: &mut HashMap<i32, hashvarchecker>, currentid: &mut i32) {
    
}

//Checks that the inputs to a function are the correct type. Uses function_arguments stored in a hashmap to accomplish this. 
//Steps through each enum to verify correct number of arguments and correct type.
fn function_a_callChecker(na: String, oldna: String, fa: function_arguments_call, ve: function_arguments, check: &mut HashMap<String, hashchecker>, varcheck: &mut HashMap<i32, hashvarchecker>, currentid: &mut i32) -> Type {
    if na == "main" {
        return Type::unknown(0);
    }
    match fa {
        function_arguments_call::arg_call_list(lfac, rfac) => {
            let (varl, far) = match ve.clone() {
                function_arguments::arg_list(le,ri) => {(le,ri)},
                _ => panic!("Too many inputs to func {:?}: function_a_callChecker",na.clone()),
            };
            let fal = function_arguments::var(varl);
            let _left = function_a_callChecker(na.clone(), oldna.clone(), unbox(lfac), fal, check, varcheck, currentid);
            let _right = function_a_callChecker(na.clone(), oldna.clone(), unbox(rfac), unbox(far), check, varcheck, currentid);
        }
        function_arguments_call::variable(va) => {
            let functionargs = match ve {
                function_arguments::arg_list(le,ri) => {return panic!("Too few inputs to func {:?}: function_a_callChecker",na.clone())},
                function_arguments::var(v) => {v},
            };
            let (varname,vartype) = match functionargs {
                variable::parameters(n,t,_v) => {(n,t)},
                variable::name(n) => {(n,Type::unknown(0))},
                _ => {panic!("Tried to give an assign as argument to function {:?} : function_a_callChecker",na.clone())},
            };
            let vaTy = varChecker(oldna.clone(), unbox(va), check, varcheck, currentid);
            if vaTy.clone() != vartype.clone() {
                panic!("Incorrect type {:?}, {:?} for func {:?}: function_a_callChecker",vaTy.clone(),vartype.clone(),na.clone());
            }
            createLocalVar(na.clone(),unbox(varname.clone()),vaTy.clone(),check,varcheck,currentid);
            return vaTy.clone();
        }
        function_arguments_call::bx(b) => {
            let functionargs = match ve {
                function_arguments::arg_list(le,ri) => {return panic!("Too few inputs to func {:?}: function_a_callChecker",na.clone())},
                function_arguments::var(v) => {v},
            };
            let (varname,vartype) = match functionargs {
                variable::parameters(n,t,_v) => {(n,t)},
                variable::name(n) => {(n,Type::unknown(0))},
                _ => {panic!("Tried to give an assign as argument to function {:?} : function_a_callChecker",na.clone())},
            };
            let ty = listChecker(oldna.clone(), unbox(b), check, varcheck, currentid);
            if ty.clone() != vartype.clone() {
                panic!("Incorrect type {:?}, {:?} for func {:?}: function_a_callChecker",ty.clone(),vartype.clone(),na.clone());
            }
            createLocalVar(na.clone(),unbox(varname.clone()),ty.clone(),check,varcheck,currentid);
            return ty;
        }
        function_arguments_call::function(fu) => {
            let functionargs = match ve {
                function_arguments::arg_list(le,ri) => {return panic!("Too few inputs to func {:?}: function_a_callChecker",na.clone())},
                function_arguments::var(v) => {v},
            };
            let (varname,vartype) = match functionargs {
                variable::parameters(n,t,_v) => {(n,t)},
                variable::name(n) => {(n,Type::unknown(0))},
                _ => {panic!("Tried to give an assign as argument to function {:?} : function_a_callChecker",na.clone())},
            };
            let futy = match unbox(fu.clone()) {
                function::parameters_call(_,_) => {
                    functionChecker(na.clone(), unbox(fu.clone()), check, varcheck, currentid)
                },
                _ => panic!("A declare was placed in a function call {:?}: function_a_callChecker",na.clone()),    
            };
            if futy.clone() != vartype.clone() {
                panic!("Incorrect type {:?}, {:?} for func {:?}: function_a_callChecker",futy.clone(),vartype.clone(),na.clone());
            }
            createLocalVar(na.clone(),unbox(varname.clone()),futy.clone(),check,varcheck,currentid);
            return futy;
        }
    }
    return Type::unknown(0);
}

//Checks if enums, including the condition.
 fn ifChecker(na: String, i: if_enum, check: &mut HashMap<String, hashchecker>, varcheck: &mut HashMap<i32, hashvarchecker>, currentid: &mut i32) {
    match i {
        if_enum::condition(v ,w) =>{
            listChecker(na.clone(), unbox(v), check, varcheck, currentid);
            function_eChecker(na.clone(), unbox(w), check, varcheck, currentid); 
        }
    }
 }

//Checks while enums, including the condition.
fn whileChecker(na: String, wh: while_enum, check: &mut HashMap<String, hashchecker>, varcheck: &mut HashMap<i32, hashvarchecker>, currentid: &mut i32) {
    match wh {
        while_enum::condition(v ,w) =>{
            listChecker(na.clone(), unbox(v), check, varcheck, currentid);
            function_eChecker(na.clone(), unbox(w), check, varcheck, currentid); 
        }
    }
}

//Checks that the return type matches the function.
fn returnChecker(na: String, v: variable_value, check: &mut HashMap<String, hashchecker>, varcheck: &mut HashMap<i32, hashvarchecker>, currentid: &mut i32) {
    let hcheck = getType(na.clone(), check);
    let rettype = match hcheck.clone() {
        hashchecker::st(bo,_,_) => {unbox(bo.clone())},
        _ => {Type::unknown(0)},
    };
    let realtype = match v {
        variable_value::variable(v) => {varChecker(na.clone(), unbox(v), check, varcheck, currentid)},
        variable_value::boxs(l) => {listChecker(na.clone(), unbox(l), check, varcheck, currentid)},
        variable_value::Number(n) => {Type::Integer},
        variable_value::Boolean(n) => {Type::boolean},
        _ => {panic!("Incorrect type {:?}: returnChecker",na.clone())},
    };
    if rettype != realtype {
        panic!("Type mismatch {:?}, {:?} for func {:?} returnChecker",rettype.clone(),realtype.clone(),na.clone());
    };
}

fn unbox<T>(value: Box<T>) -> T {
    *value
}


fn main() {}