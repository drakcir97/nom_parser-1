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
}
use crate::variable_value::{boxs, Boolean, Number};

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
    List,
    function,
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
    // )
    // )(varib);

    // let x = put_in_box("1+2+func(1+2)+3+a");
    let input = "(a+b)";
    let x: IResult<&str, &str> = delimited(tag("("), take_until(")"), tag(")"))(input);

    //let x = get_funk_body(
    //       "(
    //  let x = 5;
    // while jada;
    //a=7+6(2*7);
    //return a;)....slut)",
    // );

    // let x: IResult<&str, &str> = take(z)(varib);
    println!("{:?}", x);
}

fn variable_parser(input: &str) -> Box<variable> {
    let (input, varname) = match name_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", "error"),
    };

    let (input, vartype) = match variable_type_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", Type::unknown(0)),
    };

    let (input, pibval) = match variable_expression_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", "error"),
    };

    if vartype == Type::boolean {
        let box_4_varname = Box::new(String::from(varname));
        let box_4_value = Box::new(String::from(pibval));
        let param = parameters(box_4_varname, vartype, Box::new(Boolean(box_4_value)));

        return Box::new(param);
    }

    let x = put_in_box(pibval);
    let box_4_varname = Box::new(String::from(varname));
    let param = parameters(box_4_varname, vartype, Box::new(boxs(x)));

    return Box::new(param);
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
                map(tag("i32"), |_| Type::Integer),
                map(tag("bool"), |_| Type::boolean),
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
            map(tag("*"), |_| op::mult),
            map(tag("/"), |_| op::div),
            map(tag("-"), |_| op::sub),
            map(tag("%"), |_| op::res),
            map(tag("<"), |_| op::less),
            map(tag(">"), |_| op::greater),
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

fn get_funk_body(input: &str) -> IResult<&str, Vec<&str>> {
    delimited(
        preceded(multispace0, tag("(")),
        many0(preceded(multispace0, terminated(take_until(";"), tag(";")))),
        tag(")"),
    )(input)
}

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

fn function_body_elements(mut input_Vec: Vec<&str>) -> Box<function_elements> {
    let input: &str = input_Vec.pop().unwrap();
    let (rest, value) = match thing(input) {
        Ok(v) => v,
        Err(_q) => ("error", "error"),
    };
    let whtrem = input.trim_start_matches(" ");
    if whtrem.starts_with("let") {
        let x = variable_parser(input);
        let list = function_elements::ele_list(
            Box::new(function_elements::boxs(x)),
            function_body_elements(input_Vec),
        );
        return Box::new(list);
    } else if whtrem.starts_with("return") {
        panic!("Fix later");
    //Do stuff
    } else {
        panic!("Fix later");
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

fn function_parser(input: &str) {
    // -> IResult<&str,&str>{
    let (input, varname) = match name_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", "error"),
    };

    let (_, paren_cont) = match get_parentheses_content(input) {
        Ok(v) => v,
        Err(q) => ("error", "error"),
    };

    let (input2, return_type) = match return_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", Type::unknown(0)),
    };

    if return_type == Type::unknown(0) {
        let (input, curl_para_cont) = match get_funk_body(input) {
            Ok(v) => v,
            Err(q) => ("error", vec!["error"]),
        };
    }
}

fn if_parser(input: &str) -> Box<List> {
    let condition: IResult<&str, &str> =
        preceded(multispace0, delimited(tag("if"), take_until("{"), tag("{")))(input);
    let (restString, value) = match condition {
        Ok(v) => v,
        Err(e) => ("j4d4", "£rr0r"),
    };
    put_in_box(value)
}
