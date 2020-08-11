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

pub fn typechecker(pg : Program) {
    let (nm, statements) = match pg {
        Program::pgr(v,w) => (v,w),
    };

    let mut iter = statements.iter(); 

    for stmt in iter {
        listchecker(stmt.clone()); //Loop through and check all types.
    }
}

//Takes a struct and checks if the operand matches the left and right hand side. That is, if you try to add a integer and bool it will panic.
//Also checks for variables to see that the assignment is the correct type.
fn listchecker(ls: List) -> Type {
    match ls {
        List::Num(n) => Type::Integer,
        List::boolean(b) => Type::boolean,
        List::Cons(l,o,r) => {
            let ls = listchecker(*l);
            let rs = listchecker(*r);
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
        },
        List::var(v) => {
            match v {
                variable::parameters(na,ty,value) => {
                    match ty {
                        Type::Integer => {
                            match *value {
                                variable_value::Number(n) => {
                                    return Type::Integer;
                                },
                                variable_value::boxs(b) => {
                                    let typ = listchecker(unbox(b));
                                },
                                _ => panic!("Incorrect assignment: typechecker"),
                            };
                            return Type::unknown(0);
                        },
                        Type::boolean => {
                            match *value {
                                variable_value::Boolean(b) => {
                                    return Type::boolean;
                                },
                                variable_value::boxs(b) => {
                                    let typ = listchecker(unbox(b));
                                },
                                _ => panic!("Incorrect assignment: typechecker"),
                            };
                            return Type::unknown(0);
                        },
                        _ => Type::unknown(0),
                    };
                    return Type::unknown(0);
                },
                _ => Type::unknown(0), //Added to be able to test 1/11-19 /Rickard
            }
        },
        _ => Type::unknown(0), //Added to be able to test 1/11-19, should skip here /Rickard
    }    
}

fn unbox<T>(value: Box<T>) -> T {
    *value
}


fn main() {
    // let y = typechecker(List::Cons(Box::new(List::boolean(true)), op::and, Box::new(List::boolean(true))));
    // println!("res boolean: {:?}", y);
    // let x = typechecker(List::Cons(Box::new(Num(4)), mult, Box::new((Cons(Box::new(Num(10)), mult, Box::new(Cons(Box::new(Num(1000)), div, Box::new(Cons(Box::new(Num(3)), mult, Box::new(Cons(Box::new(Num(8)), add, Box::new(Num(7)))))))))))));
    // println!("res Integer: {:?}", x);
    // let z = typechecker(List::var(variable::parameters(Box::new("VariableName".to_string()),Type::boolean,Box::new(variable_value::Boolean(Box::new("true".to_string()))))));
    // println!("res variable assign boolean: {:?}", z);
}