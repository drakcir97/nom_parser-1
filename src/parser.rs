#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate nom;
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_till, take_until, take_while1, take_while_m_n},
    character::complete::{char, digit1, multispace0},
    character::is_alphanumeric,
    combinator::map,
    combinator::map_res,
    multi::many0,
    sequence::{delimited, preceded, terminated, tuple},
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

pub fn program_parser(input: &str)-> Program{
    let (string, result) = match many0(
    preceded(
        multispace0,
        function_parser,
    )
    )(input){
        Ok(v)=>v,
        Err(q)=>panic!(),
    };
    Program::pgr(Box::new(string.to_string()), Box::new(result))
    
}

fn variable_parser(input: &str) -> IResult<&str, expr> {

    let (input, varname) = name_parser(input)?; 
    let (pibval, vartype) = variable_type_parser(input)?;



    if vartype == Type::boolean {
        let box_4_varname = Box::new(String::from(varname));
        let (input, box_4_value) = put_in_box(pibval)?; 
        let z = match box_4_value {
            expr::list(z) => z,
            _ => panic!(),
        };
        let param = parameters(box_4_varname, vartype, Box::new(boxs(Box::new(z))));

        return Ok((input, expr::variable(param)));
    }
    let (input, x) = put_in_box(pibval)?; 
    let x = match x {
        expr::list(v) => v,
        _ => panic!(),
    };
    println!("variabel parser after pib: {:?}", (input, &x));
    let box_4_varname = Box::new(String::from(varname));
    let param = parameters(box_4_varname, vartype, Box::new(boxs(Box::new(x))));

    Ok((input, expr::variable(param)))
}

fn name_parser(input: &str) -> IResult<&str, &str> {
    let (input, varname) = preceded(
        multispace0,
        preceded(
            alt((tag("let"), tag("fn"))),
            preceded(multispace0, take_while1(char::is_alphanumeric)),
        ),
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


    Ok((input, pibval))
}

fn parser2(input: &str) -> IResult<&str, &str> {
    digit1(input)
}

fn tag_semi_col(input: &str) -> IResult<&str, &str> {
    preceded(
        multispace0,
         alt((
            tag(";"),
            preceded(tag(")"), preceded(multispace0, tag(";"))),
            tag(")"),
        )),
    )(input)
}


// Parses everything right of a "=" so in "x:i32 = 3+x+(y(1)+8)" it parses the "3+x+(y(1)+8)" part
pub fn put_in_box(input: &str) -> IResult<&str, expr> {

    println!("niput to put in box {:?}", input);
    // if input == ""{
        // return Ok(("", ))
    // }
    let resu = match get_reg_brack_cont(input) {
        Ok(v) => {
            cons_paran_parser(v)
        },
        Err(q) => {
            let (restvalue, value) = finalparser(input);
            println!("value: {:?}", value);
            let test = match parser2(value) {
                Ok(v) => {
                    integer_cons_parser(v, value, restvalue)
                },
                Err(q) => match get_reg_brack_cont(restvalue) {
                    Ok(v) => {
                        function_call_parser(v, value)
                    },
                    Err(q) => {
                        variable_cons_parser(value, restvalue)
                    }
                },
            };
            return test;
        }
    };
    return resu;
}

fn cons_paran_parser(mut v: (&str, Vec<expr>) ) -> IResult<&str, expr> {
    let express = v.1.pop().unwrap(); 
    let nextchar = v.0.chars().next().unwrap();
    let if_var = if nextchar == ';'  {
        let (restvalue,_) = tag(";")(v.0)?;
        return Ok((restvalue, express));
    }else if v.0 == "" {
        Ok(("", express))    
    } else {
        match operator(v.0){
            Ok(val)=>{
            let (input, value2) = put_in_box(val.0)?; 
            let value2 = match value2 {
                expr::list(value2) => value2,
                _ => panic!(),
            };   
            let value = match express {
                expr::list(value) => value,
                _ => panic!(),
            };
            let list = Cons(Box::new(value), val.1, Box::new(value2));
            Ok((input, expr::list(list)))
            },
            Err(q)=>{Ok((v.0, express))} 
        }
    };
    return if_var;
}

fn integer_cons_parser<'v>(v:(&str, &str), value: &'v str, restvalue: &'v str) -> IResult<&'v str, expr> {
    println!("integer cons parser input: {:?}", (value, restvalue));
    let value: i32 = value.parse().unwrap();
    let list_var = Num(value);
    // checks if rest value = "" or ;
    let nextchar = restvalue.chars().next().unwrap();
    let if_var = if nextchar == ';'  {
         let (restvalue,_) = tag(";")(restvalue)?;
        return Ok((restvalue, expr::list(list_var)));
    }
    else {
        let test1 = match operator(restvalue){
            Ok(v) =>{
                let (input, value2) = put_in_box(v.0)?;
                let mbox = match value2 {
                    expr::list(value2) => value2,
                    _ => panic!(),
                };
                let list = Cons(Box::new(Num(value)), v.1, Box::new(mbox));
                return Ok((input, expr::list(list)));
            },
            Err(q) => {
                //HERE NEEDS FIXING 
                // let (reststring,_) = tag_semi_col(restvalue)?;
                let nextchar = restvalue.chars().next().unwrap();
                if nextchar == ';' || nextchar == ')' {
                    return Ok((restvalue, expr::list(list_var)));
                }
                return Err(q);
            },
        };
        return test1;
    };
    if_var
}

fn function_call_parser<'v>(v: (&'v str, Vec<expr>), value: &'v str) -> IResult<&'v str, expr> {
    let value = String::from(value);
    //let func_box_var = function_call_parentheses_parser_final(v.1);
    let func_box_var = func_var(v.1);
    let funcpar = function::parameters_call(Box::new(value), func_box_var);
    let list_var = func(funcpar);
    println!("list_var: {:?}", list_var);
    println!("v.0: {:?}", &v.0);
    let nextchar = v.0.chars().next().unwrap();
    let if_var = if nextchar == ';'  {
        let (restvalue,_) = tag(";")(v.0)?;
        return Ok((restvalue, expr::list(list_var)));
    }else if v.0 == "" {
        return Ok(("", expr::list(list_var)));
    } else {
        // if operator in str
        match operator(v.0){
            Ok(val) =>{

                let (input, value) = put_in_box(val.0)?; 
                let value = match value {
                    expr::list(value) => value,
                    _ => panic!(),
                };
                let list = Cons(Box::new(list_var), val.1, Box::new(value));
                return Ok((input, expr::list(list)));
            },
            Err(q) => {
                // let (reststring, _) = tag(";")(v.0)?;
                // println!("after ; tag: {:?}", reststring);
                let nextchar = v.0.chars().next().unwrap();
                if nextchar == ';' || nextchar == ')' {
                    return Ok((v.0, expr::list(list_var)));
                }
                return Err(q);
            },
        };
    };
}

fn variable_cons_parser<'v>(value: &'v str, restvalue: &'v str) -> IResult<&'v str, expr> {
    let value = String::from(value);
    let list_var = if value == "true" {
        List::boolean(true) 
    } else if value == "false" {
        List::boolean(false)
    } else {
        var(variable::name(Box::new(value)))
    };
    let nextchar = restvalue.chars().next().unwrap();
    let if_val = if nextchar == ';'  {
        let (restvalue,_) = tag(";")(restvalue)?;
        return Ok((restvalue, expr::list(list_var)));
    }else if restvalue == "" {
        return Ok(("", expr::list(list_var)));
    } else {
        // operator used to fail
        match operator(restvalue){
            Ok(v)=>{
                let (input, value) = put_in_box(v.0)?; 
                let value = match value {
                    expr::list(value) => value,
                    _ => panic!(),
                };
    
                let list = Cons(Box::new(list_var), v.1, Box::new(value));
    
                Ok((input, expr::list(list)))
            },
            Err(q)=>{
                // this needs to be returned but only if tag ";" ???
                let nextchar = restvalue.chars().next().unwrap();
                if nextchar == ';' || nextchar == ')' {
                    return Ok((restvalue, expr::list(list_var)));
                }
                return Err(q);
            }
        }
    };
    return if_val;
                    
}

fn parser(input: &str) -> IResult<&str, &str> {
    preceded(
        multispace0,
        alt((
            digit1,
            take_while1(char::is_alphanumeric),
        )),
    )(input)
}

fn finalparser(input: &str) -> (&str, &str) {
    match parser(input) {
        Ok(v) => v,
        Err(q) => ("error", "error"),
    }
}


//Parses every mathematical operator, and returns it as an IResult type

use std;
fn operator(input: &str) -> IResult<&str, op> {
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
            map(tag("=="), |_| op::equal),
            map(tag("||"), |_| op::or),
            map(tag("&&"), |_| op::and),
            map(tag("<="), |_| op::lessEqual),
            map(tag(">="), |_| op::greatEqual),
        )),
    )(input);

    value
}

// gets everything in in between matched parathesis both between "{}" and "()"
fn get_parentheses_content(input: &str) -> IResult<&str, &str> {
    preceded(
        multispace0,
        alt((
            delimited(tag("("), take_until(")"), tag(")")),
            delimited(tag("{"), take_until("}"), tag("}")),
        )),
    )(input)
}

//Parses all function parameters, that can be supplied in a function call.

fn function_call_parentheses_parser(input: &str) -> IResult<&str, Vec<&str>> {
    println!("input to func_call_paran: {:?}", &input);
    let z: u8 = 0;
    let x: IResult<&str, Vec<&str>> = many0(delimited(take(z), take_until(","), tag(",")))(input);
    println!("x: {:?}",&x);
    x
}

// Calls function_call_parenthesis_parser and matches it so that error dont cause problems
// fn function_call_parentheses_parser_final(input: &str) -> Box<function_arguments_call> {
    // let (reststring, values) = match function_call_parentheses_parser(input) {
        // Ok(v) => v,
        // Err(q) => ("error", vec!["error"]),
    // };
    // func_var(values)
// }
// 
//Parses the arguments of a newly defined function, which uses the function_call_parentheses_parser to match two values.

fn function_def_parentheses_parser_final(input: &str) -> Box<function_arguments> {
    let (reststring, values) = match function_call_parentheses_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", vec!["error"]),
    };

    func_variable_defin(values, reststring)
}

//parses the arguments to a funtion call so in "function(var, func(3)+var2, 5)"" it parses the expresions "var", "func(3)+var2" and "5"  
fn func_var(mut input: Vec<expr>) -> Box<function_arguments_call> {
    println!("with input: {:?}",(&input));
    if input.len() == 1 {
        let x = input.pop().unwrap();         

        let x = match x {
            expr::list(x) => x,
            _ => panic!(),
        };
        return Box::new(function_arguments_call::bx(Box::new(x)));
    }
    let x = input.pop().unwrap();
 
    let x = match x {
        expr::list(x) => x,
        _ => panic!(),
    };
    let list = function_arguments_call::arg_call_list(
        Box::new(function_arguments_call::bx(Box::new(x))),
        func_var(input),
    );
    return Box::new(list);
}

//parses the function argumnents when defiining a function so in "fn(input:i32, input2:i32)=>i32{...}"" it parses "input:i32" and "input2:i32" 
fn func_variable_defin(mut input_vec: Vec<&str>, reststring: &str) -> Box<function_arguments> {
    if input_vec.len() == 0 {
        let nameparsed: IResult<&str, &str> =
            preceded(multispace0, take_while1(char::is_alphanumeric))(reststring);
        let (reststring, varname) = match nameparsed {
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
//needs fixing for nested ";" and "{}"(cant remember if true anymore)
//parses anythning in between curlybrackets "{}" in function definfitions and in if statements and in while statements  
pub fn get_curl_brack_body(input: &str) -> IResult<&str, Vec<expr>> {
    let z: u8 = 0;
    delimited(
        preceded(multispace0, tag("{")),
        many0(preceded(
            multispace0,
            alt((
                variable_parser,
                function_call_return_parser,
                if_parser,
                while_parser,
                put_in_box,
            )),
        )),
        preceded(multispace0, tag("}")),
    )(input)
}

pub fn get_reg_brack_cont(input:&str)->IResult<&str, Vec<expr>>{
    println!("input to get_reg_brack: {:?}", input);
    let output =  delimited(
        preceded(multispace0, tag("(")),
        many0(delimited(
            multispace0,
                put_in_box,
                // preceded(
                    multispace0,
                    // alt((
                        // tag(","),
                        // tag("")
                    // )),
                // ),
        )),
        preceded(multispace0, tag_semi_col),
    )(input);
    println!("output from get_reg_brack: {:?}",output);
    output
}

fn get_parentheses_body(input: &str) -> IResult<&str, expr> {
    let z: u8 = 0;
    let (input, exprval) = preceded(
        preceded(multispace0, tag("(")),
        preceded(
            multispace0,
            put_in_box,
        ),
    )(input)?;


    let exprval = match exprval {
        expr::list(v) => expr::list(List::paran(Box::new(v))),
        _ => panic!("parantesis priority bug"),
    };

    Ok((input, exprval))
}

// parses the return type of a newly defined function
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

//parses function body 
fn function_body_elements(mut input_Vec: Vec<expr>) -> (Box<function_elements>) {
    let input_expr: expr = input_Vec.remove(0);
    let input_fe = match input_expr {
        expr::list(x) => function_elements::List(x),
        expr::function(z) => function_elements::function(z),
        expr::variable(v) => function_elements::variable(v),
        expr::if_enum(a) => function_elements::if_enum(a),
        expr::while_enum(b) => function_elements::while_enum(b),
        expr::return_val(b) => function_elements::return_val(b),
    };
    if input_Vec.len() == 0 {
        return Box::new(input_fe);
    }

    return Box::new(function_elements::ele_list(
        Box::new(input_fe),
        function_body_elements(input_Vec),
    ));

}

//removes key words(I think?)
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

//parses functions when defined
fn function_parser(input: &str) -> IResult<&str, List> {
    let (input, varname) = match name_parser(input) {
        Ok(v) => v,
        Err(q) => return Err(q),
    };

    //Gets functions arguments
    let (input, paren_cont) = match get_parentheses_content(input) {
        Ok(v) => v,
        Err(q) => ("error", "error"),
    };

    //Gets function return type
    let (input2, return_type) = match return_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", Type::unknown(0)),
    };
    //checks if function has returntype
    if return_type == Type::unknown(0) {
        //gets everything in curly brackets
        let (input, curl_para_cont) = match get_curl_brack_body(input) {
            Ok(v) => v,
            Err(q) => panic!(),
        };
        //puts and returns function arguments in tree
        let function_arg = function_def_parentheses_parser_final(paren_cont);
        // puts and returns function elemenst in tree
        let function_elements = function_body_elements(curl_para_cont);
        let box_name = Box::new(String::from(varname));
        let function =
            function::parameters_def(box_name, function_arg, return_type, function_elements);

        return Ok((input, List::func(function)));
    }
    let (input, curl_para_cont) = match get_curl_brack_body(input2) {
        Ok(v) => v,
        Err(q) => {println!("error message {:?}", q); 
            panic!()},
    };

    //Puts and returns function arguments in tree
    let function_arg = function_def_parentheses_parser_final(paren_cont);
    //puts and returns function elements(function body) in tree
    let function_elements = function_body_elements(curl_para_cont);
    let box_name = Box::new(String::from(varname));
    let function = function::parameters_def(box_name, function_arg, return_type, function_elements);
    return Ok((input, List::func(function)));
}

//Parses the condition, and the body of if statements and while loops and adds them to the tree
fn if_parser(input: &str) -> IResult<&str, expr> {
    println!("entered if_parser");
    let (input, _) = preceded(multispace0, tag("if"))(input)?;

    let (input, paran_cont) = get_parentheses_content(input)?;
    let (_, pibresult) = put_in_box(paran_cont)?;
    let (input, curl_para_cont) = get_curl_brack_body(input)?;
    let function_elements = function_body_elements(curl_para_cont);

    let pibresult = match pibresult {
        expr::list(a) => a,
        _ => panic!("wrong in condition"),
    };

    let list = if_enum::condition(Box::new(pibresult), function_elements);
    Ok((input, expr::if_enum(list)))
}

//parses while loops
fn while_parser(input: &str) -> IResult<&str, expr> {

    let (input, _) = preceded(multispace0, tag("while"))(input)?;

    let (input, paran_cont) = get_parentheses_content(input)?;
    let (_, pibresult) = put_in_box(paran_cont)?;
    let (input, curl_para_cont) = get_curl_brack_body(input)?;
    let function_elements = function_body_elements(curl_para_cont);

    let pibresult = match pibresult {
        expr::list(a) => a,
        _ => panic!("wrong in condition"),
    };

    let list = while_enum::condition(Box::new(pibresult), function_elements);

    Ok((input, expr::while_enum(list)))
}
//Parses a return statement in a function, and returns it as an IResult.
fn function_call_return_parser(input: &str) -> IResult<&str,expr>{
    let (input,_) = preceded(multispace0,tag("return"))(input)?;

    let (input, pibresult) = put_in_box(input)?;

    let pibresult = match pibresult {
        expr::list(a) => a,
        _ => panic!("wrong in condition"),
    };

    let pibresult = variable_value::boxs(Box::new(pibresult));  



    Ok((input, expr::return_val(pibresult)))
}
