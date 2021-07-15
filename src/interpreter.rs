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

pub fn execute(pg: Program) -> Vec<(String, hashstate)> {
    let (nm, statements) = match pg {
        Program::pgr(v,w) => (v,w),
    };

    //State will contain all functions, they are stored using functionDeclare.
    //Each state contains the function name, local variables, copy of function define and the line currently running.
    let mut state: HashMap<String, hashstate> = HashMap::new(); 

    //These are used to store data in memory. As shown to the right, addressmap stores the real values,
    let mut idmap: HashMap<i32,i32> = HashMap::new();               //  id      -> address
    let mut addressmap: HashMap<i32, hashdata> = HashMap::new();    //  address -> hashdata::valuei32,valuebool
                                                                    //             hashdata::address

    let mut currentid: i32 = 1; //Incrementer for idmap, a needed but not good solution.

    let mut dec_iter = statements.iter(); 

    for stmt in dec_iter {
        functionDeclare(stmt.clone(), &mut state); //Loop through and declare all functions into state.
    }

    let mut iter = statements.iter(); 

    for stmt in iter { //Run program
        execute_List(unbox(nm.clone()),stmt.clone(),&mut state,&mut idmap,&mut addressmap,&mut currentid);
    }
    let result = getAllStates(&mut state, &mut idmap, &mut addressmap);
    return result.clone();
    //assert!(iter.is_ok());
}

// For each different type, such as loop or if statements, just hardcode that behavior. 
// Such as if statement in code using the given condition.
fn execute_List(functionname: String, ls: List, state: &mut HashMap<String, hashstate>, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>, currentid: &mut i32) -> List{
    match ls{
        List::paran(v) => {return execute_List(functionname.clone(), unbox(v), state, idmap, addressmap, currentid)},
        List::Cons(v,w,x) => {return cons_execute(functionname.clone(),v,w,x,state,idmap,addressmap,currentid)},
        List::Num(v) => {return ls},
        List::boolean(v)=>{return ls},
        List::func(fu) => {
            function_execute(functionname.clone(), fu.clone(), state, idmap, addressmap, currentid);
            let _func_var = match fu.clone(){
                function::parameters_def(_l,_m,_n,_o)=>{return List::Num(0)}, 
                function::parameters_call(v,_w)=>{
                    let fnst = getState(unbox(v),state);
                    match fnst {
                        hashstate::state(ft,_hv,_fu,_ln) => {
                            match unbox(ft.clone()) {
                                functionstate::Returned(v) => {
                                    let data = getFromAddressHashdata(unbox(v), addressmap);
                                    match data {
                                        hashdata::valuei32(a) => {return List::Num(a)},
                                        hashdata::valuebool(a) => {return List::boolean(a)},
                                        _ => {return List::Num(0)},
                                    };
                                },
                                _ => {return List::Num(0)}
                            }
                        },
                        _ => {return List::Num(0)}
                    }
                },
                _ => {return List::Num(0)}
            };
            return List::Num(0);
        },
        List::var(v) => {return var_execute(Box::new(functionname.clone()), v, state, idmap, addressmap, currentid)},
        _ => panic!("Something went wrong: execute_List"),
    };
}


//Changes current state, st is function name, line is the current line. Subject to massive changes.
fn changeState(function: String, st: hashstate, state: &mut HashMap<String, hashstate>) {
    state.insert(function,st); //Adds state if it does not exists, updates value 'st' if it does.
}

//Changes a state of a function, can be used to change status, modify local variables or to change the function.
fn changeFunctionState(function: String, st: functionstate, state: &mut HashMap<String, hashstate>) {
    let oldstate = getState(function.clone(),state);
    match oldstate.clone() {
        hashstate::state(fs, hv, fu, ln) => {
            match unbox(fs.clone()) {
                functionstate::Returned(_) => (),
                _ => changeState(function.clone(), hashstate::state(Box::new(st),hv,fu,ln),state),
            };
        },
        hashstate::Nil => {panic!("Not supposed to happen: changeFunctionState")}
    }
}

//Removes state and returns true if it existed, otherwise returns false.
fn removeState(function: String, state: &mut HashMap<String, hashstate>) -> bool{
    if state.contains_key(&function) {
        state.remove(&function);
        return true;
    } else {
        return false;
    }
}

// //Returns state, returns an error if it does not exist.
fn getState(function: String, state: &mut HashMap<String, hashstate>) -> &hashstate {
    if state.contains_key(&function) {
        let result = state.get(&function);
        match result {
            Some(val) => return val,
            None => panic!("Get state failed!: getState"),
        }
    } else {
        panic!("No such state exists!: getState");
    }
}

//Returns all states currently stored. Changes id in local variables to real values for displaying.
fn getAllStates(state: &mut HashMap<String, hashstate>, idmap: &mut HashMap<i32,i32>, addressmap: &mut HashMap<i32, hashdata>) -> Vec<(String, hashstate)> {
    let mut result: Vec<(String, hashstate)> = Vec::new();
    for (nm, val) in state.iter() {
        let newst = match val {
            hashstate::state(fst,ha,fu,ln) => {
                let temp: &mut Vec<hashvariable> = &mut Vec::new();
                let iter = ha.iter();
                for varib in iter {
                    let (na,id) = match varib {
                        hashvariable::var(na,i) => (na,i),
                        _ => panic!(),
                    };
                    let val = getFromId(*id, idmap, addressmap);
                    let rval = getFromAddressHashdata(val, addressmap);
                    let fval = match rval {
                        hashdata::valuei32(v) => v,
                        hashdata::valuebool(v) => {
                            if v == true {
                                1
                            } else {
                                0
                            }
                        },
                        _ => panic!(),
                    };
                    let hvar = hashvariable::var(na.to_string(),fval);
                    temp.push(hvar);
                };
                hashstate::state(fst.clone(),Box::new(temp.to_vec()),fu.clone(),ln.clone())
            },
            _ => panic!("Error: getAllStates"),
        };
        result.push((nm.clone(),newst.clone()));
    }
    return result;
}

//Increases line for function by 1, returns the current line executing.
fn increaseLineForFunction(function: String, state: &mut HashMap<String, hashstate>) -> i32 {
    let stfn = getState(function.clone(), state);
    match stfn.clone() {
        hashstate::state(fst, ha, fu, ln) => {
            let nst = hashstate::state(fst, ha, fu, ln+1);
            changeState(function.clone(), nst, state);
            return ln+1;     
        }
        _ => panic!("State not found: increaseLineForFunc")
    }
    return 0;
}

// //Adds var to vector of local variables in function. Returns true if it succeeded. 
fn addLocalVariable(function: String, inp: hashvariable, state: &mut HashMap<String, hashstate>) -> bool {
    if state.contains_key(&function) {
        let prevstate = getState(function.clone(),state);
        //println!("prevstate: {:?}", prevstate.clone());
        match prevstate {
            hashstate::state(fnstate, va, fu, linenum) => {
                //println!("hashstate::state{:?}", (fnstate.clone(), va.clone(), fu.clone(),linenum.clone()));
                let mut temp: Vec<hashvariable> = unbox(va.clone());
                temp.push(inp);
                changeState(function.clone(),hashstate::state(Box::new(unbox(fnstate.clone())),Box::new(temp), fu.clone(),linenum.clone()),state);
                //println!("{:?}",unbox(va.clone()));
                return true;
            },
            _ => panic!("State is incorrect: addLocalVariable")
        }
    } else {
        return false;
    }
}

//Gets local variable from function, returns hashvariable::Nil if none is found.
fn getLocalVariable(function: String, inp: String, state: &mut HashMap<String, hashstate>) -> hashvariable {
    if state.contains_key(&function) { 
        let prevstate = getState(function,state);
        match prevstate {
            hashstate::state(_fnstate,va,_fu,_linenum) => {
                let mut temp: Vec<hashvariable> = *va.clone();
                let mut iterator = temp.iter();
                for ha in iterator {
                    match ha.clone() {
                        hashvariable::var(na,_address) => {
                            if inp == na {
                                return ha.clone();
                            }
                        },
                        _ => (),
                    }
                };
            },
            _ => panic!("State is incorrect: getLocalVariable"),
        };
    }
    return hashvariable::Nil;
}

//Remove local variable if there is a match found.
fn removeLocalVariable(function: String, inp: hashvariable, state: &mut HashMap<String, hashstate>) -> hashvariable {
    if state.contains_key(&function) {
        let a = function.clone();
        let prevstate = getState(function,state);
        match prevstate {
            hashstate::state(fnstate,va,fu,linenum) => {
                let mut temp: Vec<hashvariable> = *va.clone();
                let fu2: function = *fu.clone();
                //temp.remove_item(&inp);
                let mut iterator = temp.iter().cloned();
                let mut i = 0;
                for ha in iterator {
                    if ha == inp {
                        break;
                    }
                    i+=1;
                }
                temp.remove(i);
                changeState(a,hashstate::state(Box::new(*fnstate.clone()),Box::new(temp),Box::new(fu2),*linenum),state);
                return inp;
            },
            _ => panic!("State is incorrect, removelocalvariable"),
        }
    } else {
        return hashvariable::Nil;
    }
}

//Removes all local variables for input function, this does not remove them from memory.
fn removeAllLocalVariables(function: String, state: &mut HashMap<String, hashstate>)-> bool{
    if state.contains_key(&function){
        let a = function.clone();
        let hashstate = getState(function, state);
        match hashstate{
            hashstate::state(fsb,_hashbox,fu,integer)=> {
                let mut temp: Vec<hashvariable> = Vec::new();
                let fu2: function = *fu.clone();
                changeState(a, hashstate::state(Box::new(*fsb.clone()),Box::new(temp),Box::new(fu2),*integer),state);
                return true;
            },
            _=>panic!("remove all variables"),
        }
        
    }
    panic!("state does not contain function: removealllocalvariables");
    
}

//Adds data to memory, address is optional, set to 0 to auto. Returns address added.
fn addToMemory(address: i32, data: hashdata, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>, currentid: &mut i32) -> i32 {
    if !idmap.contains_key(&currentid) {
        let oldid = *currentid;
        if address != 0 {
            idmap.insert(*currentid,address);
            addressmap.insert(address,data);
            *currentid += 1;
            return address;
        }else{
            idmap.insert(*currentid,*currentid);
            addressmap.insert(*currentid,data);
            *currentid += 1;
            return oldid;
        }
    }
    panic!("idmap already contains id!");
}

//Replace value at address with given hashdata.
fn replaceAtMemory(address: i32, data: hashdata, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>) -> bool {
    if addressmap.contains_key(&address) {
        let _toret = addressmap.remove(&address);
        addressmap.insert(address,data);
        return true;
    }
    return false;
}

//Adds pointer to memory, returns address of pointer added.
fn addPointer(address: i32, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>, currentid: &mut i32) -> i32 {
    if !idmap.contains_key(currentid) {
        let oldid = *currentid;
        idmap.insert(*currentid,*currentid);
        addressmap.insert(*currentid,hashdata::address(address));
        *currentid += 1;
        return oldid;
    }
    panic!("idmap already contains id!")
}

//Removes value at specified address, note that this is only in addressmap.
fn removeFromMemory(address: i32, idmap: &mut HashMap<i32,i32>, addressmap: &mut HashMap<i32, hashdata>, currentid: &mut i32) -> hashdata {
    if addressmap.contains_key(&address) {
        if idmap.contains_key(&address) {
            let temp = idmap.get(&address);
            match temp {
                Some(v) => {
                    if *v == address {
                        let a = address.clone();
                        idmap.remove(&address);
                        let res_rem = addressmap.remove(&a);
                        match res_rem {
                            Some(v) => return v,
                            _ => panic!("Nothing was removed: removeFromMemory"),
                        }
                    }
                },
                _ => (),
            }
        }
        let res_rem2 = addressmap.remove(&address);
        match res_rem2 {
            Some(v) => return v,
            _ => panic!("Nothing was removed: removeFromMemory"),
        }
    }
    panic!("Address does not exist in addressmap: removeFromMemory");
}

//Gets data from memory, address specifies where in addressmap.
fn getFromMemory(address: i32,addressmap: &mut HashMap<i32, hashdata>) -> hashdata {
    if addressmap.contains_key(&address) {
        let result = addressmap.get(&address);
        match result {
            Some(val) => return val.clone(),
            None => panic!("Not found in memory"),
        }
    } else {
        panic!("Not found in memory");
    }
}
// Gets data from memory, in this case from id. Id is different from address and is stored in idmap.                    // Still untested
fn getFromId(id: i32, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>) -> hashdata {
    if idmap.contains_key(&id) {
        let result = idmap.get(&id);
        match result {
            Some(val) => {
                let result2 = addressmap.get(val);
                match result2 {
                    Some(val2) => return val2.clone(),
                    None => panic!("Not found in idmap"),
                }
            }
            None => panic!("Not found in idmap"),
        }
    } else {
        panic!("Not found in idmap");
    }
}

//Untangles data by recurson to get real data from pointers.
fn getFromAddressHashdata(inp: hashdata, addressmap: &mut HashMap<i32, hashdata>) -> hashdata {
    match inp {
        hashdata::address(a) => {
            let fmem = getFromMemory(a,addressmap);
            match fmem {
                hashdata::address(_a) => {
                    return getFromAddressHashdata(fmem, addressmap);
                },
                hashdata::valuebool(_v) => {
                    return fmem;
                },
                hashdata::valuei32(_v) => {
                    return fmem;
                }
            }
        },
        hashdata::valuebool(_v) => {
            return inp;
        },
        hashdata::valuei32(_v) => {
            return inp;
        }
    }
}

//Loops through the parse tree to find and declare the different functions to the hashmap.
fn functionDeclare(ls: List, state: &mut HashMap<String, hashstate>) {
    match ls {
        List::func(f) => {
            match f.clone() {
                function::parameters_def(na,ar,ty,ele) => {
                    let temp: Vec<hashvariable> = Vec::new();
                    let tempstring: String = unbox(na.clone());
                    changeState(tempstring,hashstate::state(Box::new(functionstate::Declared),Box::new(temp),Box::new(f.clone()),-1),state);
                },
                _ => (),
            };
        },
        _ => (),// Do nothing
    }
}

//Takes a deconstructed List::Cons, both sides and operator are different parameters. 
//Executes both sides independantly and summarizes them according to operand. Returns List to be able to both return i32 and bool.
fn cons_execute(functionname: String, l: Box<List>, oper: op ,r:  Box<List>, state: &mut HashMap<String, hashstate>, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>, currentid: &mut i32)-> List{
    let expr = match oper{
        op::add => {
            let llist: List = execute_List(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
            let rlist: List = execute_List(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
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
            let llist: List = execute_List(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
            let rlist: List = execute_List(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
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
            let llist: List = execute_List(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
            let rlist: List = execute_List(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
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
            let llist: List = execute_List(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
            let rlist: List = execute_List(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
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
            let llist: List = execute_List(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
            let rlist: List = execute_List(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
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
            let llist: List = execute_List(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
            let rlist: List = execute_List(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
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
            let llist: List = execute_List(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
            let rlist: List = execute_List(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
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
            let llist: List = execute_List(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
            let rlist: List = execute_List(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
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
            let llist: List = execute_List(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
            let rlist: List = execute_List(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
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
            let llist: List = execute_List(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
            let rlist: List = execute_List(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
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
            let llist: List = execute_List(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
            let rlist: List = execute_List(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
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
        op::equal => {
            let llist: List = execute_List(functionname.clone(), unbox(l), state, idmap, addressmap, currentid);
            let rlist: List = execute_List(functionname.clone(), unbox(r), state, idmap, addressmap, currentid);
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
            if (vall == valr) {
                return List::boolean(true);
            }
            return List::boolean(false);
        },
        

        _ => panic!("Operand not supported: cons_execute")
    };

}

fn unbox<T>(value: Box<T>) -> T {
    *value
}

//Executes a function, includes changing the status of prev function to Calling.
fn function_execute(functionname: String, func_var: function, state: &mut HashMap<String, hashstate>, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>, currentid: &mut i32){
    match func_var{
        function::parameters_def(na,_m,_n,_o)=>{
            if unbox(na.clone()) == "main" { //Special case for main. Calls in when we find define in code. Ensures that it is always called.
                changeFunctionState(unbox(na.clone()), functionstate::Running, state); //Set new as running

                let w = function_arguments_call::variable(Box::new(variable::name(Box::new("".to_string()))));
                paramcall_execute(na.clone(), na.clone(), Box::new(w), state, idmap, addressmap, currentid);

            }
        }, //Do nothing on define, since this is handled in functionDeclare, except for main.
        function::parameters_call(v,w)=>{
            changeFunctionState(functionname.clone(), functionstate::Calling, state); //Set old function as calling
            changeFunctionState(unbox(v.clone()), functionstate::Running, state); //Set new as running
            paramcall_execute(v.clone(), Box::new(functionname.clone()), w, state, idmap, addressmap, currentid);
            changeFunctionState(functionname.clone(), functionstate::Running, state); //Set old function as running again
        },
    };
}

//Executes variables, if it doesn't exist, add to mem and to local vars. Otherwise replace value at address.
//Checks so that types are correct.
fn var_execute(functionname: Box<String>, variable_var: variable, state: &mut HashMap<String, hashstate>, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>, currentid: &mut i32) -> List{
    match variable_var{
        variable::parameters(na,ty,val) => {
            let testIfExist = getLocalVariable(unbox(functionname.clone()), unbox(na.clone()), state);
            match testIfExist {
                hashvariable::var(_oldname,oldaddress) => {  //If we get this it means a local var exists with same name, replace val in memory.
                    match unbox(val) {                      //This means we don't have to touch local var, since it just contains address.
                        variable_value::Boolean(b) => {     //And we replaced value at address
                            if ty != Type::boolean {
                                panic!("Type mismatch in var: var_execute")
                            };
                            let temp = hashdata::valuebool(b);
                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                        },
                        variable_value::Number(n) => {
                            if ty != Type::Integer {
                                panic!("Type mismatch in var: var_execute")
                            };
                            let temp = hashdata::valuei32(n);
                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                        },
                        variable_value::boxs(b) => {
                            let ls = unbox(b);
                            match ls.clone() {
                                List::Num(n) => {
                                    if ty != Type::Integer {
                                        panic!("Type mismatch in var: var_execute")
                                    };
                                    let temp = hashdata::valuei32(n);
                                    let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                                },
                                List::boolean(b) => {
                                    if ty != Type::boolean {
                                        panic!("Type mismatch in var: var_execute")
                                    };
                                    let temp = hashdata::valuebool(b);
                                    let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                                },
                                List::var(v) => {
                                    let varval = var_execute(functionname.clone(),v , state, idmap, addressmap, currentid);
                                    match varval.clone() {
                                        List::Num(n) => {
                                            if ty != Type::Integer {
                                                panic!("Type mismatch in var: var_execute")
                                            };
                                            let temp = hashdata::valuei32(n);
                                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                                        },
                                        List::boolean(b) => {
                                            if ty != Type::boolean {
                                                panic!("Type mismatch in var: var_execute")
                                            };
                                            let temp = hashdata::valuebool(b);
                                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
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
                                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                                        },
                                        List::boolean(b) => {
                                            if ty != Type::boolean {
                                                panic!("Type mismatch in var: var_execute")
                                            };
                                            let temp = hashdata::valuebool(b);
                                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
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
                                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                                        },
                                        List::boolean(b) => {
                                            if ty != Type::boolean {
                                                panic!("Type mismatch in var: var_execute")
                                            };
                                            let temp = hashdata::valuebool(b);
                                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                                        },
                                        _ => (),
                                    };
                                },
                                _ => (),
                            }
                        },
                        variable_value::variable(b) => {},
                        _ => panic!("Incorrect value: var_execute"),
                    };
                },
                hashvariable::Nil => { //No match, add as usual.
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
                            }
                        },
                        variable_value::variable(b) => {},
                        _ => panic!("Incorrect value: var_execute"),
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
        variable::assign(na,val) => {
            let testIfExist = getLocalVariable(unbox(functionname.clone()), unbox(na.clone()), state);
            match testIfExist {
                hashvariable::var(_oldname,oldaddress) => {
                    let undata = getFromMemory(oldaddress.clone(),addressmap);
                    let tatypedata = getFromAddressHashdata(undata, addressmap);
                    let dtype = match tatypedata {
                        hashdata::valuei32(va) => {Type::Integer},
                        hashdata::valuebool(va) => {Type::boolean},
                        _ => {panic!("Type incorrect from fetched local variable {:?}",unbox(na.clone()))},
                    };
                    match unbox(val) {                      //This means we don't have to touch local var, since it just contains address.
                        variable_value::Boolean(b) => {     //And we replaced value at address
                            if dtype != Type::boolean {
                                panic!("Type mismatch in var: var_execute")
                            };
                            let temp = hashdata::valuebool(b);
                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                        },
                        variable_value::Number(n) => {
                            if dtype != Type::Integer {
                                panic!("Type mismatch in var: var_execute")
                            };
                            let temp = hashdata::valuei32(n);
                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                        },
                        variable_value::boxs(b) => {
                            let ls = unbox(b);
                            match ls.clone() {
                                List::Num(n) => {
                                    if dtype != Type::Integer {
                                        panic!("Type mismatch in var: var_execute")
                                    };
                                    let temp = hashdata::valuei32(n);
                                    let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                                },
                                List::boolean(b) => {
                                    if dtype != Type::boolean {
                                        panic!("Type mismatch in var: var_execute")
                                    };
                                    let temp = hashdata::valuebool(b);
                                    let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                                },
                                List::var(v) => {
                                    let varval = var_execute(functionname.clone(),v , state, idmap, addressmap, currentid);
                                    match varval.clone() {
                                        List::Num(n) => {
                                            if dtype != Type::Integer {
                                                panic!("Type mismatch in var: var_execute")
                                            };
                                            let temp = hashdata::valuei32(n);
                                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                                        },
                                        List::boolean(b) => {
                                            if dtype != Type::boolean {
                                                panic!("Type mismatch in var: var_execute")
                                            };
                                            let temp = hashdata::valuebool(b);
                                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                                        },
                                        _ => (),
                                    };
                                },
                                List::Cons(lli,op,rli) => {
                                    let val = cons_execute(unbox(functionname.clone()), lli, op, rli, state, idmap, addressmap, currentid);
                                    match val.clone() {
                                        List::Num(n) => {
                                            if dtype != Type::Integer {
                                                panic!("Type mismatch in var: var_execute")
                                            };
                                            let temp = hashdata::valuei32(n);
                                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                                        },
                                        List::boolean(b) => {
                                            if dtype != Type::boolean {
                                                panic!("Type mismatch in var: var_execute")
                                            };
                                            let temp = hashdata::valuebool(b);
                                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                                        },
                                        _ => (),
                                    };
                                },
                                List::func(fu) => {
                                    let val = execute_List(unbox(functionname.clone()), ls.clone(), state, idmap, addressmap, currentid);
                                    match val.clone() {
                                        List::Num(n) => {
                                            if dtype != Type::Integer {
                                                panic!("Type mismatch in var: var_execute")
                                            };
                                            let temp = hashdata::valuei32(n);
                                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                                        },
                                        List::boolean(b) => {
                                            if dtype != Type::boolean {
                                                panic!("Type mismatch in var: var_execute")
                                            };
                                            let temp = hashdata::valuebool(b);
                                            let _addressOfTemp = replaceAtMemory(oldaddress, temp, idmap, addressmap);
                                        },
                                        _ => (),
                                    };
                                },
                                _ => (),
                            }
                        },
                        variable_value::variable(b) => {},
                        _ => panic!("Incorrect value: var_execute"),
                    };
                },
                hashvariable::Nil => { panic!("No local variable found for name {:?}: var_execute", unbox(na.clone())) },
            };
            return List::Num(0);
        },
    };
}

//Execute if statements, uses cons_execute to check statement.
fn if_execute(functionname: Box<String>, if_e: if_enum, state: &mut HashMap<String, hashstate>, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>, currentid: &mut i32){
    let (ifst, if_body) = match if_e{
        if_enum::condition(v,w)=>(v,w)
    };
    if cons_execute(unbox(functionname.clone()), ifst, op::greater, Box::new(List::Num(0)), state, idmap, addressmap, currentid)  != List::boolean(false) {
        function_elements_execute(functionname.clone(), unbox(if_body), state, idmap, addressmap, currentid);
        
    }
}


//Execute while statements, uses cons_execute to check statement.
fn while_execute(functionname: Box<String>, while_e: while_enum, state: &mut HashMap<String, hashstate>, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>, currentid: &mut i32){
    let (while_statement, while_body) =  match while_e{
        while_enum::condition(v,w)=>(v,w),
    };
    changeFunctionState(unbox(functionname.clone()), functionstate::Looping, state);
    while cons_execute(unbox(functionname.clone()), while_statement.clone(), op::greater, Box::new(List::Num(0)), state, idmap, addressmap, currentid) != List::boolean(false) {
        function_elements_execute(functionname.clone(), unbox(while_body.clone()), state, idmap, addressmap, currentid);
        let fnstate = getState(*functionname.clone(),state);
        match fnstate {
            hashstate::state(st,vars,fu,line) => {
                match unbox(st.clone()) {
                    functionstate::Returned(_) => {
                        break;
                    },
                    _ => (),
                };
            },
            _ => (),
        }
    }
    changeFunctionState(unbox(functionname), functionstate::Running, state);
}

//Execute return statements
fn return_execute(functionname: Box<String>, var_val: variable_value, state: &mut HashMap<String, hashstate>, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>, currentid: &mut i32) {
    let st2: &mut HashMap<String, hashstate> = &mut state.clone();
    let fnstate = getState(*functionname.clone(),st2);
    match fnstate {
        hashstate::state(st,vars,fu,line) => {
            match unbox(st.clone()) {
                functionstate::Returned(_) => {
                    return;
                },
                _ => (),
            };
            match var_val {
                variable_value::Boolean(b) => {
                    let temp = hashdata::valuebool(b);
                    let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                    let hashdata_ret = hashdata::address(addressOfTemp);
                    let temp = hashstate::state(Box::new(functionstate::Returned(Box::new(hashdata_ret))),vars.clone(),fu.clone(),line.clone());
                    changeState(unbox(functionname.clone()),temp,state);
                },
                variable_value::Number(n) => {
                    let temp = hashdata::valuei32(n);
                    let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                    let hashdata_ret = hashdata::address(addressOfTemp);
                    let temp = hashstate::state(Box::new(functionstate::Returned(Box::new(hashdata_ret))),vars.clone(),fu.clone(),line.clone());
                    changeState(unbox(functionname.clone()),temp,state);
                },
                variable_value::variable(v) => {
                    match unbox(v) {
                        variable::name(n) => {
                            let varname = unbox(n);
                            let variab = getLocalVariable(unbox(functionname.clone()),varname,state);
                            match variab {
                                hashvariable::var(_na,ad) => {
                                    let hdata = getFromMemory(ad,addressmap);
                                    match hdata {
                                        hashdata::address(_a) => {
                                            let untangledData = getFromAddressHashdata(hdata,addressmap);
                                            let temp = hashstate::state(Box::new(functionstate::Returned(Box::new(untangledData))),vars.clone(),fu.clone(),line.clone());
                                            changeState(unbox(functionname.clone()),temp,state);
                                        },
                                        hashdata::valuebool(_va__b) => {
                                            let temp = hashstate::state(Box::new(functionstate::Returned(Box::new(hdata))),vars.clone(),fu.clone(),line.clone());
                                            changeState(unbox(functionname.clone()),temp,state);
                                        },
                                        hashdata::valuei32(_va__i) => {
                                            let temp = hashstate::state(Box::new(functionstate::Returned(Box::new(hdata))),vars.clone(),fu.clone(),line.clone());
                                            changeState(unbox(functionname.clone()),temp,state);
                                        },
                                        _ => panic!("This is not supposed to happen.... return_execute"),
                                    };
                                },
                                _ => panic!("Local variable does not exist: return_execute"),
                            };
                        },
                        _ => panic!("Return does not support this type: return_execute"),
                    };
                },
                variable_value::boxs(va) => {
                    match unbox(va) {
                        List::var(v) => {
                            let varval = variable_value::variable(Box::new(v));
                            return_execute(functionname.clone(), varval, state, idmap, addressmap, currentid);
                        },
                        List::Cons(bl, opr, br) => {
                            let consval = cons_execute(unbox(functionname.clone()), bl, opr, br, state, idmap, addressmap, currentid);
                            let hashdata_ret = match consval {
                                List::Num(n) => {
                                    hashdata::valuei32(n)
                                },
                                List::boolean(n) => {
                                    hashdata::valuebool(n)
                                },
                                _ => panic!(""),
                            }; 
                            let temp = hashstate::state(Box::new(functionstate::Returned(Box::new(hashdata_ret))),vars.clone(),fu.clone(),line.clone());
                            changeState(unbox(functionname.clone()),temp,state);
                        },
                        List::boolean(b) => {
                            let hashdata_ret = hashdata::valuebool(b);
                            let temp = hashstate::state(Box::new(functionstate::Returned(Box::new(hashdata_ret))),vars.clone(),fu.clone(),line.clone());
                            changeState(unbox(functionname.clone()),temp,state);
                        },
                        List::Num(n) => {
                            let hashdata_ret = hashdata::valuei32(n);
                            let temp = hashstate::state(Box::new(functionstate::Returned(Box::new(hashdata_ret))),vars.clone(),fu.clone(),line.clone());
                            changeState(unbox(functionname.clone()),temp,state);
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

//Execute function elements
fn function_elements_execute(functionname: Box<String>, fe: function_elements, state: &mut HashMap<String, hashstate>, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>, currentid: &mut i32){
    let _ln = increaseLineForFunction(unbox(functionname.clone()),state);
    match fe {
        function_elements::ele_list(v,w)=>{
            let ele1: function_elements = unbox(v);
            let ele2: function_elements = unbox(w);
            let res1 = function_elements_execute(functionname.clone(), ele1, state, idmap, addressmap, currentid);
            let res2 = function_elements_execute(functionname.clone(), ele2, state, idmap, addressmap, currentid);
        },
        function_elements::boxs(v)=>{
            let box_cont: variable = unbox(v);
            var_execute(functionname, box_cont, state, idmap, addressmap, currentid); 
        },
        function_elements::if_box(v)=>{
            let box_cont= unbox(v);
            if_execute(functionname, box_cont, state, idmap, addressmap, currentid);
        },
        function_elements::List(v)=>{
            execute_List(unbox(functionname), v, state, idmap, addressmap, currentid);
        },
        function_elements::function(v)=>{
            function_execute(unbox(functionname), v, state, idmap, addressmap, currentid);
        },
        function_elements::variable(v)=>{
            var_execute(functionname, v, state, idmap, addressmap, currentid);
        },
        function_elements::if_enum(v)=>{
            if_execute(functionname, v, state, idmap, addressmap, currentid);
        },
        function_elements::while_enum(v) => {
            while_execute(functionname, v, state, idmap, addressmap, currentid);
        },
        function_elements::return_val(v) => {
            return_execute(functionname,v,state,idmap,addressmap,currentid);
        },
    }
}

//Execute a function call with args.
fn paramcall_execute(functionname: Box<String>, oldfunctionname: Box<String>, args: Box<function_arguments_call>, state: &mut HashMap<String, hashstate>, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>, currentid: &mut i32){
    //Volvo 740 med svetsad diff e gött
    //Böttad 240 me basdunk och dåliga bakdäck slår allt
    let fnstate = getState(unbox(functionname.clone()), state);
    match fnstate {
        hashstate::state(st,_v,_func,_line) => {
            match unbox(st.clone()) {
                functionstate::Declared => {function_arguments_call_execute(functionname.clone(), oldfunctionname.clone(), args, state, idmap, addressmap, currentid)},
                functionstate::Running => {function_arguments_call_execute(functionname.clone(), oldfunctionname.clone(), args, state, idmap, addressmap, currentid)},
                functionstate::Calling => {function_arguments_call_execute(functionname.clone(), oldfunctionname.clone(), args, state, idmap, addressmap, currentid)},
                functionstate::Stopped => {function_arguments_call_execute(functionname.clone(), oldfunctionname.clone(), args, state, idmap, addressmap, currentid)},
                functionstate::Looping => {function_arguments_call_execute(functionname.clone(), oldfunctionname.clone(), args, state, idmap, addressmap, currentid)},
                functionstate::Returned(v) => {function_arguments_call_execute(functionname.clone(), oldfunctionname.clone(), args, state, idmap, addressmap, currentid)},
            }
        },
        _ => panic!("Function does not exist"),
    }
}

//Declares variables using function_arguments_call_declare and sends function further to be executed. Also changes state to Running.
fn function_arguments_call_execute(functionname: Box<String>, oldfunctionname: Box<String>, args: Box<function_arguments_call>, state: &mut HashMap<String, hashstate>, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>, currentid: &mut i32) {
    let st2: &mut HashMap<String, hashstate> = &mut state.clone();
    let fnstate = getState(*functionname.clone(), st2);
    match fnstate {
        hashstate::state(_st,v,fu,line) => {
            let temp = hashstate::state(Box::new(functionstate::Running),v.clone(),fu.clone(),line.clone());
            changeState(unbox(functionname.clone()),temp,state);
            match unbox(fu.clone()) {
                function::parameters_def(_na,fuag,_ty,ele) => {
                    let functionargs = fuag.clone();
                    function_arguments_call_declare(functionname.clone(), oldfunctionname.clone(), args, functionargs, state, idmap, addressmap, currentid);
                    function_elements_execute(functionname.clone(),unbox(ele.clone()),state,idmap,addressmap,currentid);
                },
                _ => panic!("Function stored incorrectly in mem: function_arguments_call_execute"),
            };
        },
        _ => panic!("Function is not declared!: function_arguments_call_execute"),
    }
}

//Declares variables sent to a function through call and adds them to memeory. Handles nestled function calls.
//Takes function args to ensure that names and order is correct when declaring variables.
fn function_arguments_call_declare(functionname: Box<String>, oldfunctionname: Box<String>, args: Box<function_arguments_call>, fuargs: Box<function_arguments>, state: &mut HashMap<String, hashstate>, idmap: &mut HashMap<i32,i32>,addressmap: &mut HashMap<i32, hashdata>, currentid: &mut i32) {
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
            let _leftSide = function_arguments_call_declare(functionname.clone(),oldfunctionname.clone(),a1,fal,state,idmap,addressmap,currentid);
            let _rightSide = function_arguments_call_declare(functionname.clone(),oldfunctionname.clone(),a2,far,state,idmap,addressmap,currentid);
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
                    function_arguments_call_declare(functionname.clone(), oldfunctionname.clone(), newargs, fuargs.clone(), state, idmap, addressmap, currentid);
                },
                List::boolean(b) => {
                    if vartype != Type::boolean {
                        panic!("Type mismatch in var: function_arguments_call_declare")
                    };
                    let temp = hashdata::valuebool(b);
                    let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                    let temp2 = hashvariable::var(unbox(varname),addressOfTemp);
                    addLocalVariable(unbox(functionname.clone()), temp2, state);
                },
                List::Num(n) => {
                    if vartype != Type::Integer {
                        panic!("Type mismatch in var: function_arguments_call_declare")
                    };
                    let temp = hashdata::valuei32(n);
                    let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                    let temp2 = hashvariable::var(unbox(varname),addressOfTemp);
                    addLocalVariable(unbox(functionname.clone()), temp2, state);
                },
                List::Cons(a,t,b) => {
                    let val = cons_execute(unbox(oldfunctionname.clone()),a,t,b,state,idmap,addressmap,currentid);
                    let temp = function_arguments_call::bx(Box::new(val));
                    return function_arguments_call_declare(functionname.clone(),oldfunctionname.clone(),Box::new(temp),fuargs,state,idmap,addressmap,currentid);
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
            match unbox(fu) {
                function::parameters_call(na, ar) => {
                    //let st2: &mut HashMap<String, hashstate> = &mut state.clone();
                    let fnstate = getState(unbox(functionname.clone()), state);
                    match fnstate {
                        hashstate::state(st,v,ele,line) => {
                            let newstate = hashstate::state(Box::new(functionstate::Calling),v.clone(),ele.clone(),line.clone());
                            changeState(unbox(functionname.clone()),newstate,state);
                            function_arguments_call_execute(na.clone(), functionname.clone(), ar, state, idmap, addressmap, currentid);
                            //Wait for state to change to Returned, and get value from memory. (functionstate::Returned(Box<hashdata::address/value>))
                            let returnedState = getState(unbox(na.clone()),state);
                            match returnedState {
                                hashstate::state(st,_v,fu,_line) => {
                                    match unbox(st.clone()) {
                                        functionstate::Returned(v) => {
                                            match unbox(v.clone()) {
                                                hashdata::address(a) => {
                                                    let ffrommem = getFromMemory(a, addressmap);
                                                    let hdata = getFromAddressHashdata(ffrommem, addressmap);
                                                    let ty = match hdata {hashdata::valuei32(_) => Type::Integer, _ => Type::boolean};
                                                    if vartype != ty {
                                                        panic!("Type mismatch in function call: function_arguments_call_declare")
                                                    };
                                                    let ad = addToMemory(0, hdata, idmap, addressmap, currentid);
                                                    let varToAdd = hashvariable::var(unbox(varname),ad);
                                                    addLocalVariable(unbox(functionname.clone()), varToAdd, state);
                                                },
                                                hashdata::valuei32(v) => {
                                                    if vartype != Type::Integer {
                                                        panic!("Type mismatch in function call: function_arguments_call_declare")
                                                    };
                                                    let hdata = hashdata::valuei32(v);
                                                    let ad = addToMemory(0, hdata, idmap, addressmap, currentid);
                                                    let varToAdd = hashvariable::var(unbox(varname),ad);
                                                    addLocalVariable(unbox(functionname.clone()), varToAdd, state);
                                                },
                                                hashdata::valuebool(v) => {
                                                    if vartype != Type::boolean {
                                                        panic!("Type mismatch in function call: function_arguments_call_declare")
                                                    };
                                                    let hdata = hashdata::valuebool(v);
                                                    let ad = addToMemory(0, hdata, idmap, addressmap, currentid);
                                                    let varToAdd = hashvariable::var(unbox(varname),ad);
                                                    addLocalVariable(unbox(functionname.clone()), varToAdd, state);
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
                        _ => panic!("Function attempting call does not exist!: function_arguments_call_declare"),
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
                _ => {panic!("Tried to give an assign as argument to function {:?} : function_arguments_call_declare",functionname.clone())},
            };
            match unbox(va.clone()) {
                variable::parameters(na,_ty,val) => {
                    match unbox(val) {
                        variable_value::Boolean(b) => {
                            if vartype != Type::boolean {
                                panic!("Type mismatch in var: function_arguments_call_declare")
                            };
                            let temp = hashdata::valuebool(b);
                            let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                            let temp2 = hashvariable::var(unbox(varname),addressOfTemp);
                            addLocalVariable(unbox(functionname.clone()), temp2, state);
                        },
                        variable_value::Number(n) => {
                            if vartype != Type::Integer {
                                panic!("Type mismatch in var: function_arguments_call_declare")
                            };
                            let temp = hashdata::valuei32(n);
                            let addressOfTemp = addToMemory(0, temp, idmap, addressmap, currentid);
                            let temp2 = hashvariable::var(unbox(varname),addressOfTemp);
                            addLocalVariable(unbox(functionname.clone()), temp2, state);
                        },
                        _ => panic!("temp"), //Might have to include more cases for variable_value here if needed.
                    };
                },
                variable::name(n) => {
                    let vnam = unbox(n);
                    let hvar = getLocalVariable(unbox(oldfunctionname.clone()), vnam.clone(), state);
                    let oldaddress = match hvar {
                        hashvariable::var(oldna, oldad) => {oldad},
                        _ => return panic!("No variable named {:?} in function {:?} found: function_arguments_call_declare",vnam.clone(),oldfunctionname.clone()),
                    };
                    let ffrommem = getFromMemory(oldaddress, addressmap);
                    let hdata = getFromAddressHashdata(ffrommem, addressmap);
                    let ty = match hdata {hashdata::valuei32(_) => Type::Integer, _ => Type::boolean};
                    if vartype != ty {
                        panic!("Type mismatch in prev local var: function_arguments_call_declare")
                    };
                    let ad = addToMemory(0, hdata, idmap, addressmap, currentid);
                    let varToAdd = hashvariable::var(unbox(varname),ad);
                    addLocalVariable(unbox(functionname.clone()), varToAdd, state);
                },
                _ => panic!("Type not yet supported: function_arguments_call_declare"),
            };
        },
    }
}

fn main() {}