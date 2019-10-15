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
use std::borrow::Cow;
use std::process;
use std::result;
use std::str;

#[derive(Debug, PartialEq, Eq)]
enum List {
    Cons(Box<List>, op, Box<List>),
    Num(i32),
    func(function),
    var(variable),
    paran(Box<List>),
}
use crate::List::{func, var, Cons, Num};

#[derive(Debug, PartialEq, Eq)]
enum op {
    add,
    sub,
    div,
    mult,
    res,
    less,
    greater,
    lessEqual,
    greatEqual,
    and,
    or,
    wrong,
    unknown(usize),
    // &&,
    //||,
}
use crate::op::{add, div, mult, res, sub, wrong};

#[derive(Debug, PartialEq, Eq)]
enum variable_value {
    variable,
    boxs(Box<List>),
    Number(i32),
    Boolean(Box<String>),
    Nil(i32),
}
use crate::variable_value::{boxs, Boolean, Nil, Number};

#[derive(Debug, PartialEq, Eq)]
enum variable {
    parameters(Box<String>, Type, Box<variable_value>),
    name(Box<String>),
}
use crate::variable::{name, parameters};

#[derive(Debug, PartialEq, Eq)]
enum Type {
    Integer,
    boolean,
    unknown(i32),
}

#[derive(Debug, PartialEq, Eq)]
enum function {
    parameters_def(
        Box<String>,
        Box<function_arguments>,
        Type,
        Box<function_elements>,
    ),
    parameters_call(Box<String>, Box<function_arguments_call>),
}

#[derive(Debug, PartialEq, Eq)]
enum function_arguments {
    arg_list(variable, Box<function_arguments>),
    var(variable),
}

#[derive(Debug, PartialEq, Eq)]
enum function_arguments_call {
    arg_call_list(Box<function_arguments_call>, Box<function_arguments_call>),
    variable,
    bx(Box<List>),
    function,
}

#[derive(Debug, PartialEq, Eq)]
enum function_elements {
    ele_list(Box<function_elements>, Box<function_elements>),
    boxs(Box<variable>),
    if_box(Box<if_enum>),
    List,
    function,
}

#[derive(Debug, PartialEq, Eq)]
enum if_enum{
    condition(Box<List>,Box<function_elements>)
}

fn main() {
    //let k = put_in_box("-3");
    //let list = Box::leak(k);
    //  let  strinf = "abcd";

    //    println!("{:?}", varib);
    // let z: u8 = 0;
    //
    // let varib = "ab,ab,ab,ef";
    // let x: IResult<&str,Vec<&str>> = many0(
    // delimited(
    // take(z),
    // take_until(","),
    // tag(","),
    // )val.pop()
    // )(varib);

    // let x = put_in_box("1+2+func(1+2)+3+a");
    //let input = "(a+b)";
    //let x: IResult<&str, &str> = delimited(tag("("), take_until(")"), tag(")"))(input);

    // let x = function_parser(
    // "getfunkbody(input: i32) -> i32{
    //  let x: i32 = 5;
    //  let b: i32 = 3+6+7;
    // }",
    // );

    let x = get_curl_brack_body(
        "{
        let x = 5;
        let k = 9;
        if true {
            if jdad {
                let banna = false;
                let apple = true;
            };
            let i = 89;
            let ifthing = 7;
        };
    }",
    );

    // let x = get_curl_brack_body("{
    // let x = 5;
    // let b = 3+6+7;
    // }");

    //let x =  variable_parser("x: i32 = 2+3+4;");
    // let x: IResult<&str, &str> = take(z)(varib);'a
    println!("{:?}", &x);
    let (rest, mut val)= match x{
        Ok(v)=>v,
        Err(_) => panic!("jada")
    };
    val.pop();
    println!("{:?}", val.pop())
}

fn variable_parser(input: &str) -> Box<variable> {
    let (input, varname) = match name_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", "error"),
    };
    println!("input and varname: {:?}", (&input, &varname));
    let (pibval, vartype) = match variable_type_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", Type::unknown(0)),
    };
    // println!("input and vartype: {:?}", (&input, &vartype));
    // let (input, pibval) = match variable_expression_parser(input) {
    // Ok(v) => v,
    // Err(q) => ("error", "error"),
    // };
    // println!("input and pibval {:?}", (&input, &pibval));

    if vartype == Type::boolean {
        let box_4_varname = Box::new(String::from(varname));
        let box_4_value = Box::new(String::from(pibval));
        let param = parameters(box_4_varname, vartype, Box::new(Boolean(box_4_value)));

        return Box::new(param);
    }
    println!("put in box thing {:?}", pibval);
    let x = put_in_box(pibval);
    let box_4_varname = Box::new(String::from(varname));
    let param = parameters(box_4_varname, vartype, Box::new(boxs(x)));

    return Box::new(param);
}

fn name_parser(input: &str) -> IResult<&str, &str> {
    let (input, varname) = preceded(
        multispace0,
        // preceded(
        //alt((tag("let"), tag("fn"))),
        preceded(multispace0, take_while1(char::is_alphanumeric)),
        // ),
    )(input)?;

    Ok((input, varname))
}

fn variable_type_parser(input: &str) -> IResult<&str, Type> {
    let (input, vartype) = preceded(
        tag(":"),
        preceded(
            multispace0,
            alt((
                alt((
                    terminated(
                        map(tag("i32"), |_| Type::Integer),
                        preceded(multispace0, tag("=")),
                    ),
                    terminated(
                        map(tag("bool"), |_| Type::boolean),
                        preceded(multispace0, tag("=")),
                    ),
                )),
                alt((
                    map(tag("i32"), |_| Type::Integer),
                    map(tag("bool"), |_| Type::boolean),
                )),
            )),
        ),
    )(input)?;
    Ok((input, vartype))
}

fn variable_expression_parser(input: &str) -> IResult<&str, &str> {
    let (input, pibval) =
        preceded(multispace0, delimited(tag("="), take_until(";"), tag(";")))(input)?;

    //let x = put_in_box(pibval);
    //let box_4_varname = Box::new(varname)
    //let param = parameters(box_4_varname,vartype,x)
    //Box::new(param)

    Ok((input, pibval))
}

fn parser2(input: &str) -> IResult<&str, &str> {
    digit1(input)
}

fn put_in_box(input: &str) -> Box<List> {
    let (restvalue, value) = finalparser(input);
    //let value = value.as_bytes();
    let test: Box<List> = match parser2(value) {
        Ok(v) => {
            let value: i32 = value.parse().unwrap();
            let box_var = Box::new(Num(value));
            if restvalue == "" {
                return box_var;
            }
            let (restvalue, operator) = operator(restvalue);

            let list = Cons(Box::new(Num(value)), operator, put_in_box(restvalue));
            return Box::new(list);
        }
        Err(q) => match get_parentheses_content(restvalue) {
            Ok(v) => {
                let value = String::from(value);
                let func_box_var = function_call_parentheses_parser_final(v.1);
                let funcpar = function::parameters_call(Box::new(value), func_box_var);
                let box_var = Box::new(func(funcpar));

                if v.0 == "" {
                    return box_var;
                }
                let (restvalue, operator) = operator(v.0);
                let list = Cons(box_var, operator, put_in_box(restvalue));
                return Box::new(list);
            }
            Err(q) => {
                let value = String::from(value);
                let box_var = Box::new(var(variable::name(Box::new(value))));

                if restvalue == "" {
                    return box_var;
                }
                let (restvalue, operator) = operator(restvalue);

                let list = Cons(box_var, operator, put_in_box(restvalue));

                return Box::new(list);
            }
        },
    };
    test
}

fn parser(input: &str) -> IResult<&str, &str> {
    preceded(
        multispace0,
        alt((
            digit1,
            take_while1(char::is_alphanumeric),
            //get_parentheses_content,
        )),
    )(input)
}

fn finalparser(input: &str) -> (&str, &str) {
    match parser(input) {
        Ok(v) => v,
        Err(q) => ("error", "error"),
    }
}

use std;
fn operator(input: &str) -> (&str, op) {
    let value: IResult<&str, op> = preceded(
        multispace0,
        alt((
            map(tag("+"), |_| op::add),
            map(tag("/"), |_| op::div),
            map(tag("-"), |_| op::sub),
            map(tag("%"), |_| op::res),
            map(tag("<"), |_| op::less),
            map(tag("*"), |_| op::mult),
            map(tag(">"), |_| op::greater),
            map(tag("||"), |_| op::or),
            map(tag("&&"), |_| op::and),
            map(tag("<="), |_| op::lessEqual),
            map(tag(">="), |_| op::greatEqual),
            //map(take_till(is_alphanumeric), |r: &[str]| op::unknown(r.len())),
        )),
    )(input);

    match value {
        Ok(v) => v,
        Err(q) => ("error", op::unknown(0)),
    }
    //{
    //      Ok(v) => v,
    //      Err(q) => ("error", "error")
    // }

    // if value == "+"{
    //     return (reststring,op::add)
    // }else if value == "*"{
    //     return (reststring, op::mult)
    // }else{
    //     process::exit(1);
    // }
}

fn get_parentheses_content(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(tag("("), take_until(")"), tag(")")),
        delimited(tag("{"), take_until("}"), tag("}")),
    ))(input)
}

fn function_call_parentheses_parser(input: &str) -> IResult<&str, Vec<&str>> {
    let z: u8 = 0;
    let x: IResult<&str, Vec<&str>> = many0(delimited(take(z), take_until(","), tag(",")))(input);

    x
}

fn function_call_parentheses_parser_final(input: &str) -> Box<function_arguments_call> {
    let (reststring, values) = match function_call_parentheses_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", vec!["error"]),
    };

    func_var(values, reststring)
}

fn function_def_parentheses_parser_final(input: &str) -> Box<function_arguments> {
    let (reststring, values) = match function_call_parentheses_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", vec!["error"]),
    };

    func_variable_defin(values, reststring)
}

fn func_var(mut input: Vec<&str>, reststring: &str) -> Box<function_arguments_call> {
    if input.len() == 0 {
        //return Box::new(function_arguments_call::bx(put_in_box(input.pop().unwrap())))
        return Box::new(function_arguments_call::bx(put_in_box(reststring)));
    }
    let x = put_in_box(input.pop().unwrap());
    let list = function_arguments_call::arg_call_list(
        Box::new(function_arguments_call::bx(x)),
        func_var(input, reststring),
    );
    return Box::new(list);
}

fn func_variable_defin(mut input_vec: Vec<&str>, reststring: &str) -> Box<function_arguments> {
    if input_vec.len() == 0 {
        let (reststring, varname) = match name_parser(reststring) {
            Ok(v) => v,
            Err(q) => ("error", "error"),
        };
        let (reststring, var_type) = match variable_type_parser(reststring) {
            Ok(v) => v,
            Err(q) => ("error", Type::unknown(0)),
        };
        let variable = parameters(Box::new(String::from(varname)), var_type, Box::new(Nil(0)));

        return Box::new(function_arguments::var(variable));
    }

    let input = input_vec.pop().unwrap();

    let (input, varname) = match name_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", "error"),
    };
    let (input, var_type) = match variable_type_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", Type::unknown(0)),
    };
    let variable = parameters(Box::new(String::from(varname)), var_type, Box::new(Nil(0)));

    return Box::new(function_arguments::arg_list(
        variable,
        func_variable_defin(input_vec, reststring),
    ));
}
//needs fixing for neseted ";" and "{}"
fn get_curl_brack_body(input: &str) -> IResult<&str, Vec<&str>> {
    let z: u8 = 0;
    delimited(
        preceded(multispace0, tag("{")),
        many0(preceded(
            multispace0,
           // alt((test_loop, terminated(take_until(";"), tag(";")))),
            terminated(take_until(";"), tag(";")),
        )),
        preceded(multispace0, tag("}")),
    )(input)
}

// fn test<'a>(input: &str) -> IResult<&str,&str>{
// let z: u8 = 0;
// let delimited_var: IResult<&str, &str> = delimited(
// tag("if"),
// take_until("{"),
// take(z),
// )(input);
// let (input, value) = match delimited_var {
// Ok (v) => v,
//Err(e) => panic!("asd"),
// _ => panic!("asd"),
// };
//
// let (input, curl_body) = match get_curl_brack_body(input){
// Ok(v)=>v,
// Err(q)=>panic!("jaifda"),
// };
//
// let mut fullstring: String = "if".to_string();
// fullstring.push_str(&value);
// let mut loopstring: Cow<String> = Cow::Borrowed(&fullstring);
//let fullstring1 = format!("if {}",value);
// for string in curl_body{
// string.to_string().push_str(";");
// loopstring.push_str(string);
// let tempstr = concat_str(&loopstring.to_string(),string);
// &loopstring.push_str(string);
// let loopstring = Cow::Owned(loopstring);
// }
//
// let loopstring: Cow<String> = Cow::Owned(loopstring);
// let s_slice: &str = &loopstring[..];
//
// return Ok((input, s_slice));
// }
//
// fn concat_str(inp1: &'static str, inp2: &'static str) -> &'static str {
// inp1.to_string().push_str(inp2);
// return &inp1;
// }
//
//
//https://doc.rust-lang.org/rust-by-example/flow_control/loop/return.html
// 
// fn test_loop(input: &str) -> IResult<&str, &str> {
    // let z: u8 = 0;
    // let delimited_var: IResult<&str, &str> = delimited(tag("if"), take_until("{"), take(z))(input);
    // let (input, value) = match delimited_var {
        // Ok(v) => v,
       //// Err(e) => panic!("asd"),
        // _ => panic!("asd"),
    // };
// 
    // let (input, mut curl_body) = match get_curl_brack_body(input) {
        // Ok(v) => v,
        // Err(q) => panic!("jada"),
    // };
    // let mut fullstring = "if".to_string();
    // fullstring.push_str(&value);
    // let mut counter = curl_body.len();
// 
    // let fullstr: String = loop {
        // if counter == 0 {
            // break fullstring;
        // }
        // fullstring.push_str(curl_body.pop().unwrap());
        // fullstring.to_owned();
        // counter = curl_body.len();
    // };
    // fullstr.to_owned();
    ////let testvar = test3(fullstring);
    // let testvar: &str = fullstr.as_str();
    // return Ok((input, testvar));
// }
// 
// fn test3(input: String) -> &str{
//     input.to_owned();
//     let fullstr: &str = &input[..];
//     return fullstr
// }

// fn test2(input: Vec<&str>) -> String{
// if input.len() == 0{
// let string = input.pop().unwrap();
// let string: String = string.to_string();
// return string
// }
//
// let string = input.pop().unwrap();
// let string: String = string.to_string();
// let string2 = test2(input);
// string.push_str(&string2)
//
// }

fn return_parser(input: &str) -> IResult<&str, Type> {
    preceded(
        multispace0,
        preceded(
            tag("->"),
            preceded(
                multispace0,
                alt((
                    map(tag("i32"), |_| Type::Integer),
                    map(tag("bool"), |_| Type::boolean),
                )),
            ),
        ),
    )(input)
}

fn function_body_elements(mut input_Vec: Vec<&str>) -> (Box<function_elements>, Vec<&str>) {
    println!("input_vec: {:?}", input_Vec);
    let input_str: &str = input_Vec.swap_remove(0);
    let (input, value) = match thing(input_str) {
        Ok(v) => v,
        Err(_q) => ("error", "error"),
    };
    let whtrem = input_str.trim_start_matches(" ");
    println!("checking call number {:?}", whtrem);
    if whtrem.starts_with("let") {
        println!("rest: {:?}", input);
        let x = variable_parser(input);
        println!("x variable parser result: {:?}", &x);
        if input_Vec.len() == 0 {
            return Box::new(function_elements::boxs(x));
        }
        let list = function_elements::ele_list(
            Box::new(function_elements::boxs(x)),
            function_body_elements(input_Vec),
        );

        return Box::new(list);
    } else if whtrem.starts_with("return") {
        panic!("Fix later return");
    //Do stuff
    } else if whtrem.starts_with("while") {
        panic!("fix later while");
    //while_parser(whtrem)
    } else if whtrem.starts_with("if") {
        let (if_box, input_vec) = if_while_parser(input, input_Vec);
        if input_vec.len = 0{
            let list = function_elements::if_box(if_box);
            return Box::new(list)
        }


        panic!("fuuck");

    } else {
        panic!("Fix later elsek");
    }
}

fn thing(input: &str) -> IResult<&str, &str> {
    preceded(
        multispace0,
        alt((
            tag("let"),
            tag("return"),
            take_while1(char::is_alphanumeric),
        )),
    )(input)
}

fn function_parser(input: &str) -> Box<function> {
    //Gets function name
    let (input, varname) = match name_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", "error"),
    };
    println!("varname: {:?}", varname);
    //Gets functions arguments
    let (input, paren_cont) = match get_parentheses_content(input) {
        Ok(v) => v,
        Err(q) => ("error", "error"),
    };
    println!("paran_cont: {:?}", paren_cont);
    //Gets function return type
    let (input2, return_type) = match return_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", Type::unknown(0)),
    };
    println!("input2, returntype: {:?}", (input2, &return_type));

    //checks if function has returntype
    if return_type == Type::unknown(0) {
        //gets everything in curly brackets
        let (input, curl_para_cont) = match get_curl_brack_body(input) {
            Ok(v) => v,
            Err(q) => ("error", vec!["error"]),
        };
        println!("curl_para_cont in if: {:?}", curl_para_cont);
        //puts and returns function arguments in tree
        let function_arg = function_def_parentheses_parser_final(paren_cont);
        // puts and returns function elemenst in tree
        let function_elements = function_body_elements(curl_para_cont);
        let box_name = Box::new(String::from(varname));
        let function =
            function::parameters_def(box_name, function_arg, return_type, function_elements);

        return Box::new(function);
    }
    let (input, curl_para_cont) = match get_curl_brack_body(input2) {
        Ok(v) => v,
        Err(q) => ("error", vec!["error"]),
    };

    println!("curl_para_cont out if: {:?}", curl_para_cont);
    //Puts and returns function arguments in tree
    let function_arg = function_def_parentheses_parser_final(paren_cont);
    //puts and returns function elements(function body) in tree
    let function_elements = function_body_elements(curl_para_cont);
    let box_name = Box::new(String::from(varname));
    let function = function::parameters_def(box_name, function_arg, return_type, function_elements);

    return Box::new(function);
}

//Parses the condition, and the body of if statements and while loops and adds them to the tree

fn if_while_parser(input: &str, input_vec: Vec<&'static str>) -> (Box<if_enum>, Vec<&'static str>) {
    let condition: IResult<&str, &str> = get_parentheses_content(input);
    let (input, value) = match condition {
        Ok(v) => v,
        Err(e) => ("Error", "if_while_parser failed"),
    };
    let expression_box = put_in_box(value);

    let deleted_ws_brac: IResult<&str,&str>  = preceded(multispace0, tag("{"),)(input);
    let (input,_) = match deleted_ws_brac{
        Ok(v) => v,
        Err(q) => panic!("if expression wrong") 
    };
    
    let vector: Vec<&str> = vec!(input); 
    let (body_box2, vector) = function_body_elements(vector);
    let (body_box, input_vec) = function_body_elements(input_vec);
    let input_vec: Vec<&str> = input_vec;
    let arglist = Box::new(function_elements::ele_list(body_box2,body_box));

    let list = (if_enum::condition(expression_box, arglist));
    return (Box::new(list), input_vec)

}
