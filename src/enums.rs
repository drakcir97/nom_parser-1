extern crate nom;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum expr {
    list(List),
    function(function),
    variable(variable),
    if_enum(if_enum),
    while_enum(while_enum),
    return_val(variable_value),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum List {
    Cons(Box<List>, op, Box<List>),
    Num(i32),
    boolean(bool),
    func(function),
    var(variable),
    paran(Box<List>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum op {
    add,
    sub,
    div,
    mult,
    res,
    less,
    greater,
    equal,
    lessEqual,
    greatEqual,
    and,
    or,
    wrong,
    unknown(usize),
    // &&,
    //||,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum variable_value {
    variable(Box<variable>),
    boxs(Box<List>),
    Number(i32),
    Boolean(bool),
    Nil(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum variable {
    parameters(Box<String>, Type, Box<variable_value>),
    name(Box<String>),
    assign(Box<String>, Box<variable_value>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Integer,
    boolean,
    unknown(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum function {
    parameters_def(
        Box<String>,
        Box<function_arguments>,
        Type,
        Box<function_elements>,
    ),
    parameters_call(Box<String>, Box<function_arguments_call>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum function_arguments {
    arg_list(variable, Box<function_arguments>),
    var(variable),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum function_arguments_call {
    arg_call_list(Box<function_arguments_call>, Box<function_arguments_call>),
    variable(Box<variable>),
    bx(Box<List>),
    function(Box<function>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum function_elements {
    ele_list(Box<function_elements>, Box<function_elements>),
    boxs(Box<variable>),
    if_box(Box<if_enum>),
    List(List),
    function(function),
    variable(variable),
    if_enum(if_enum),
    while_enum(while_enum),
    return_val(variable_value),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum if_enum {
    condition(Box<List>, Box<function_elements>),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum while_enum {
    condition(Box<List>, Box<function_elements>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Program {
    pgr(Box<String>, Box<Vec<List>>),
}

// -------------------------------------------------------------------------------------------- \\
//Hashmap for containing the state we are currently in.
//static mut state: HashMap<&str, hashstate> = HashMap::new(); //Internal types are just a guess.

// Example from lession
//static mut idmap: HashMap<i32,i32> = HashMap::new();               //  id      -> address
//static mut addressmap: HashMap<i32, hashdata> = HashMap::new();    //  address -> hashdata::valuei32,valuebool
//    hashdata::address

//static mut currentid: i32 = 0; //Incrementer for idmap, bad fix for now
// Intepreter enums
// State for function, contains whether it's running or simply declared. Contains a vector of hashvariable. Last i32 is for line currently running.
#[derive(Debug, Clone, PartialEq)]
pub enum hashstate {
    state(
        Box<functionstate>,
        Box<Vec<hashvariable>>,
        Box<function>,
        i32,
    ),
    Nil,
}

//First one is variable name, last one is the address in addressmap and idmap
#[derive(Debug, Clone, PartialEq)]
pub enum hashvariable {
    var(String, i32),
    Nil,
}

//The data located in addressmap. Value refers to a real value, address just points to another address.
#[derive(Debug, Clone, PartialEq)]
pub enum hashdata {
    valuei32(i32),
    valuebool(bool),
    address(i32),
}

//The different states a function can be in.
#[derive(Debug, Clone, PartialEq)]
pub enum functionstate {
    Running,
    Stopped,
    Declared,
    Looping,
    Calling,
    Returned(Box<hashdata>),
}