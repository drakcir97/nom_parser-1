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
    //println!("Entered variable parser");

    //println!("with input: {:?}",&input);
    // ////println!("whole input: {:?}", &input);
    let (input, varname) = name_parser(input)?; //{
                                                // Ok(v) => v,
                                                // Err(q) => ("error", "error"),
                                                // };
    // ////println!("input and varname: {:?}", (&input, &varname));
    let (pibval, vartype) = variable_type_parser(input)?;

    //pibval till parentesparsern?
    //  let (pibval, rest) = get_parentheses_body(pibval)
    // {

    // Ok(v) => v,
    // Err(q) => ("error", Type::unknown(0)),
    // };
    // //println!("input and vartype: {:?}", (&input, &vartype));
    // let (input, pibval) = match variable_expression_parser(input) {
    // Ok(v) => v,
    // Err(q) => ("error", "error"),
    // };
    // //println!("input and pibval {:?}", (&input, &pibval));

    if vartype == Type::boolean {
        let box_4_varname = Box::new(String::from(varname));
        let (input, box_4_value) = put_in_box(pibval)?; //{
                                                        // Ok(v)=> v,
                                                        // _=>panic!(),
                                                        // };
        let z = match box_4_value {
            expr::list(z) => z,
            _ => panic!(),
        };
        let param = parameters(box_4_varname, vartype, Box::new(boxs(Box::new(z))));

        return Ok((input, expr::variable(param)));
    }
    //////println!("put in box thing {:?}", pibval);
    let (input, x) = put_in_box(pibval)?; //{
                                          // Ok(v)=>v,
                                          // _=>panic!(),
                                          // };
    let x = match x {
        expr::list(v) => v,
        _ => panic!(),
    };
    let box_4_varname = Box::new(String::from(varname));
    let param = parameters(box_4_varname, vartype, Box::new(boxs(Box::new(x))));

    Ok((input, expr::variable(param)))
}

fn name_parser(input: &str) -> IResult<&str, &str> {
    //println!("Entered name parser");
    //println!("with input: {:?}",&input);
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
    //println!("Entered vairable type parser");
    //println!("with input: {:?}",&input);
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
    //println!("Entered variable ecxpression parser");
    //println!("with input: {:?}",&input);
    let (input, pibval) =
        preceded(multispace0, delimited(tag("="), take_until(";"), tag(";")))(input)?;

    //let x = put_in_box(pibval);
    //let box_4_varname = Box::new(varname)
    //let param = parameters(box_4_varname,vartype,x)expr::list(x)
    //Box::new(param)

    Ok((input, pibval))
}

//parses one or more digits 
fn parser2(input: &str) -> IResult<&str, &str> {
    //println!("Entered parser2");
    //println!("with input: {:?}",&input);
    digit1(input)
}

fn tag_semi_col(input: &str) -> IResult<&str, &str> {
    //println!("Entered Tag Semi Col");
    //println!("with input: {:?}",&input);
    preceded(
        multispace0,
        alt((
            tag(";"),
            preceded(tag(")"), preceded(multispace0, tag(";"))),
            tag(")"),
        )),
    )(input)
}

// fn gpb_facade(input: &str)->IResult<&str,expr>{
// let (input, exprlist) get_parentheses_body(input)?;
//
// }
//
// fn gbp_exprvec_to_expr(input: Vec<expr>){
// let input_expr = input.pop().unwrap();
// let input_list = match input_expr{
// expr::list(q)=>List::paran(q),
// _=>panic!(),
// }
// if input.len()==0{
//
// }
// }

// Parses everything right of a "=" so in "x:i32 = 3+x+(y(1)+8)" it parses the "3+x+(y(1)+8)" part
pub fn put_in_box(input: &str) -> IResult<&str, expr> {
    // ////println!("inputTTTTT: {:?}", input);
    //println!("Enterd Put In BOX");
    //println!("with input: {:?}",&input);
    let (input, value1) = match get_parentheses_body(input) {
        Ok(v) => {
            let if_var = if v.0 == ";" || v.0 == "" {
                return Ok(("", v.1));
            } else {
                let (checkvar1, checkvar2) = match tag_semi_col(v.0) {
                    Ok(v) => v,
                    Err(q) => ("error", "Error"),
                };
                let if_var_inner = if checkvar2 == ";" || checkvar2 == ")" {
                    // ////println!("restvalue and checkvar1: {:?}", (&v.0, &checkvar1));
                    Ok((checkvar1, v.1))
                } else {
                    let (restvalue, operator) = operator(v.0)?;
                    // ////println!("CHECK IF OPERATOR FUCKS: {:?}", (&restvalue, &operator));
                    let (input, value2) = put_in_box(restvalue)?; //{
                                                                  //     Ok(v) => v,
                                                                  //      _ => panic!()
                                                                  // };
                    // ////println!("put in box returns input, value2: {:?}", (&input, &value2));
                    let value2 = match value2 {
                        expr::list(value2) => value2,
                        _ => panic!(),
                    };
                    let value = match v.1 {
                        expr::list(value) => value,
                        _ => panic!(),
                    };
                    // ////println!("value2 after change: {:?}", &value2);
                    let list = Cons(Box::new(value), operator, Box::new((value2)));
                    Ok((input, expr::list(list)))
                };
                return if_var_inner;
            };
            return if_var;

            // //println!("value2 after change: {:?}", &value2);
            // let list = Cons(Box::new(value), operator,Box::new((value2)));
            // (input,expr::list(list))
        }
        Err(q) => {
            // ////println!("DID NOT FINNISH!!!!");
            let (restvalue, value) = finalparser(input);
            //let value: IResult<&str, &str> = finalpar2(input);
            //let value = value.as_bytes();
            let test: (&str, expr) = match parser2(value) {
                Ok(v) => {
                    let value: i32 = value.parse().unwrap();
                    let list_var = Num(value);
                    let if_var = if restvalue == ";" || restvalue == "" {
                        return Ok(("", expr::list(list_var)));
                    } else {
                        let (checkvar1, checkvar2) = match tag_semi_col(restvalue) {
                            Ok(v) => v,
                            Err(q) => ("error", "Error"),
                        };
                        let if_var_inner = if checkvar2 == ";" || checkvar2 == ")" {
                            // ////println!("restvalue and checkvar1: {:?}", (&restvalue, &checkvar1));
                            (checkvar1, expr::list(list_var))
                        } else {
                            let (restvalue, operator) = operator(restvalue)?;
                            // ////println!("CHECK IF OPERATOR FUCKS: {:?}", (&restvalue, &operator));

                            let (input, value2) = put_in_box(restvalue)?; //{
                                                                          //     Ok(v) => v,
                                                                          //      _ => panic!()
                                                                          // };
                            // ////println!("put in box returns input, value2: {:?}", (&input, &value2));
                            let value2 = match value2 {
                                expr::list(value2) => value2,
                                _ => panic!(),
                            };
                            // ////println!("value2 after change: {:?}", &value2);
                            let list = Cons(Box::new(Num(value)), operator, Box::new((value2)));
                            (input, expr::list(list))
                        };
                        if_var_inner
                    };
                    if_var
                }
                Err(q) => match get_parentheses_content(restvalue) {
                    Ok(v) => {
                        let value = String::from(value);
                        // ////println!("value: {:?}", value);
                        let func_box_var = function_call_parentheses_parser_final(v.1);
                        // ////println!("func_box_var: {:?}", func_box_var);
                        let funcpar = function::parameters_call(Box::new(value), func_box_var);
                        // ////println!("func_par: {:?}", funcpar);
                        let list_var = func(funcpar);

                        let if_val = if v.0 == ";" || v.0 == "" {
                            ("", expr::list(list_var))
                        } else {
                            //println!("{:?}",&v.0);
        
                            let (checkvar1, checkvar2) = match tag_semi_col(v.0) {
                                                                              Ok(v)=>v,
                                                                             Err(q)=> ("error","error")
                                                                              };
                           // println!("checkvar {:?}",&checkvar2);
                            let if_val_inner = if checkvar2 == ";" || checkvar2 == ")" {
                                // ////println!("v.0 and checkvar1: {:?}", (&v.0, &checkvar1));
                                (checkvar1, expr::list(list_var))
                            } else {
                                // ////println!("listvar: {:?}", list_var);
                                let (restvalue, operator) = operator(v.0)?;
                                //                            list
                                let (input, value) = put_in_box(restvalue)?; //{
                                                                             // Ok(v) => v,
                                                                             // _ => panic!()
                                                                             // };
                                let value = match value {
                                    expr::list(value) => value,
                                    _ => panic!(),
                                };
                                let list = Cons(Box::new(list_var), operator, Box::new(value));
                                (input, expr::list(list))
                            };
                            if_val_inner
                        };
                        if_val
                    }
                    Err(q) => {
                        let value = String::from(value);
                        let list_var = var(variable::name(Box::new(value)));

                        let if_val = if restvalue == ";" || restvalue == "" {
                            ("", expr::list(list_var))
                        } else {
                            let (checkvar1, checkvar2) = match tag_semi_col(restvalue){
                                Ok(v)=>v,
                                Err(q)=> ("error","Error"),
                            };
                            let if_val_inner = if checkvar2 == ";" || checkvar2 == ")" {
                                // ////println!(
                                //    // "restvalue2 and checkvar1: {:?}",
                                //    // (&restvalue, &checkvar1)
                                // //);
                                (checkvar1, expr::list(list_var))
                            } else {
                                let (restvalue, operator) = operator(restvalue)?;
                                let (input, value) = put_in_box(restvalue)?; //{
                                                                             // Ok(v)=>v,
                                                                             // _=> panic!()
                                                                             // };
                                let value = match value {
                                    expr::list(value) => value,
                                    _ => panic!(),
                                };

                                let list = Cons(Box::new(list_var), operator, Box::new(value));

                                (input, expr::list(list))
                            };
                            if_val_inner
                        };
                        if_val
                    }
                },
            };
            test
        }
    };
    // ////println!("input and value1 ; {:?}", (&input, &value1));
    return Ok((input, value1));
}

fn parser(input: &str) -> IResult<&str, &str> {
    //println!("Entered parser");
    //println!("with input: {:?}",&input);
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
    //println!("Enterd finalparser");
    //println!("with input: {:?}",&input);
    match parser(input) {
        Ok(v) => v,
        Err(q) => ("error", "error"),
    }
}

// fn finalpar2(input: &str) -> IResult(&str, &str) {
// parser(input)
// }

//Parses every mathematical operator, and returns it as an IResult type

use std;
fn operator(input: &str) -> IResult<&str, op> {
    //println!("Entered operator");
    //println!("with input: {:?}",&input);
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

    value

    // match value { recent commnent
    // Ok(v) => v,
    // Err(q) => ("error", op::unknown(0)),
    // }

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

// gets everything in in between matched parathesis both between "{}" and "()"
fn get_parentheses_content(input: &str) -> IResult<&str, &str> {
    //println!("Entered get_parathesis content");
    //println!("with input: {:?}",&input);
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
    //println!("Entered function_call_paranthesis_parser");
    //println!("with input: {:?}",&input);

    let z: u8 = 0;
    let x: IResult<&str, Vec<&str>> = many0(delimited(take(z), take_until(","), tag(",")))(input);

    x
}

// Calls function_call_parenthesis_parser and matches it so that error dont cause problems
fn function_call_parentheses_parser_final(input: &str) -> Box<function_arguments_call> {
    //println!("Entered function_call_paranthesis_parser_final");
    //println!("with input: {:?}",&input);
    let (reststring, values) = match function_call_parentheses_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", vec!["error"]),
    };
    // ////println!("reststring, values: {:?}", (&reststring, &values));
    func_var(values, reststring)
}

//Parses the arguments of a newly defined function, which uses the function_call_parentheses_parser to match two values.

fn function_def_parentheses_parser_final(input: &str) -> Box<function_arguments> {
    //println!("Entered function_def_paranthesis_parser_final");
    //println!("with input: {:?}",&input);
    let (reststring, values) = match function_call_parentheses_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", vec!["error"]),
    };

    func_variable_defin(values, reststring)
}

//parses the arguments to a funtion call so in "function(var, func(3)+var2, 5)"" it parses the expresions "var", "func(3)+var2" and "5"  
fn func_var(mut input: Vec<&str>, reststring: &str) -> Box<function_arguments_call> {
    //println!("Entered funcvar");
    println!("with input: {:?}",(&input, &reststring));
    if input.len() == 0 {
        // ////println!("enter check");
        //return Box::new(function_arguments_call::bx(put_in_box(input.pop().unwrap())))
        let (_, x) = match put_in_box(reststring) {
            Ok(v) => v,
            _ => panic!(),
        };
        // ////println!("x: {:?}", &x);
        let x = match x {
            expr::list(x) => x,
            _ => panic!(),
        };
        // ////println!("x: {:?}", &x);
        return Box::new(function_arguments_call::bx(Box::new(x)));
    }
    let (input_str, x) = match put_in_box(input.pop().unwrap()) {
        Ok(v) => v,
        _ => panic!(),
    };
    let x = match x {
        expr::list(x) => x,
        _ => panic!(),
    };
    let list = function_arguments_call::arg_call_list(
        Box::new(function_arguments_call::bx(Box::new(x))),
        func_var(input, reststring),
    );
    return Box::new(list);
}

//parses the function argumnents when defiining a function so in "fn(input:i32, input2:i32)=>i32{...}"" it parses "input:i32" and "input2:i32" 
fn func_variable_defin(mut input_vec: Vec<&str>, reststring: &str) -> Box<function_arguments> {
    //println!("Entered func_variable_defin");
    //println!("with input: {:?}",(&input_vec, &reststring));
    if input_vec.len() == 0 {
        // ////println!("full reststring fvd {:?}", reststring);
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

        // ////println!("varnaem and var_type: {:?}", (&varname, &var_type));
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
fn get_curl_brack_body(input: &str) -> IResult<&str, Vec<expr>> {
    println!("Entered get_curl_brack_body");
    println!("with input: {:?}",&input);
    let z: u8 = 0;
    delimited(
        preceded(multispace0, tag("{")),
        many0(preceded(
            multispace0,
            // alt((test_loop, terminated(take_until(";"), tag(";")))),
            alt((
                variable_parser,
                put_in_box,
                function_call_return_parser,
                if_parser,
                //function_parser,
            )),
        )),
        preceded(multispace0, tag("}")),
    )(input)
}

//parses everything in betweeen parenthesis ()
fn get_parentheses_body(input: &str) -> IResult<&str, expr> {
    //println!("Entered get_paranthesis_Body");
    //println!("with input: {:?}",&input);
    ////println!("ENTER CHECK !!!!!");
    let z: u8 = 0;
    let (input, exprval) = preceded(
        preceded(multispace0, tag("(")),
        preceded(
            multispace0,
            // alt((test_loop, terminated(take_until(";"), tag(";")))),
            // alt((
            put_in_box,
            //variable_parser
            //function_parser,
            // )),
        ),
    )(input)?;
    // let (input, exprval) = delimited(
    // preceded(multispace0, tag("(")),
    // preceded(
    // multispace0,
    /////alt((test_loop, terminated(take_until(";"), tag(";")))),
    ////alt((
    // put_in_box,
    ////variable_parser
    ////function_parser,
    //// )),
    // ),
    // preceded(multispace0, tag(")")),
    // )(input)?;

    let exprval = match exprval {
        expr::list(v) => expr::list(List::paran(Box::new(v))),
        _ => panic!("parantesis priority bug"),
    };

    // //println!(
        // "check what parantbody GAAA returns: {:?}",
        // (&input, &exprval)
    // );
    Ok((input, exprval))
}

// parses the return type of a newly defined function
fn return_parser(input: &str) -> IResult<&str, Type> {
    //println!("Entered return_parser");
    //println!("with input: {:?}",&input);
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
    println!("Entered function_body_elements");
    println!("with input: {:?}",&input_Vec);
    // //println!("input_vec: {:?}", input_Vec);
    let input_expr: expr = input_Vec.remove(0);
    let input_fe = match input_expr {
        expr::list(x) => function_elements::List(x),
        expr::function(z) => function_elements::function(z),
        expr::variable(v) => function_elements::variable(v),
        expr::if_enum(a) => function_elements::if_enum(a),
        expr::while_enum(b) => function_elements::while_enum(b),
        expr::return_val(b) => function_elements::return_val(b),
    };
    println!("inputVec {:?}", input_Vec)
    if input_Vec.len() == 0 {
        return Box::new(input_fe);
    }

    return Box::new(function_elements::ele_list(
        Box::new(input_fe),
        function_body_elements(input_Vec),
    ));

    //panic!("just stops return fault while programming")
}

//removes key words(I think?)
fn thing(input: &str) -> IResult<&str, &str> {
    //println!("Entered thing");
    //println!("with input: {:?}",&input);
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
    //println!("Entered function_parser");
    //println!("with input: {:?}",&input);
    //Gets function name
    let (input, varname) = match name_parser(input) {
        Ok(v) => v,
        Err(q) => return Err(q),
    };
    // if input == "error" && varname == "error"{
        // return Err((    ))
    // }
    //println!("After  nameparser: {:?}", input);
    // ////println!("varname: {:?}", varname);
    //Gets functions arguments
    let (input, paren_cont) = match get_parentheses_content(input) {
        Ok(v) => v,
        Err(q) => ("error", "error"),
    };
    //println!("After  get paranthesis content: {:?}", input);
    // ////println!("paran_cont: {:?}", &paren_cont);
    //Gets function return type
    let (input2, return_type) = match return_parser(input) {
        Ok(v) => v,
        Err(q) => ("error", Type::unknown(0)),
    };
    // ////println!("input2, returntype: {:?}", (input2, &return_type));
    //println!("After return parser: {:?}", input2);
    //checks if function has returntype
    if return_type == Type::unknown(0) {
        //gets everything in curly brackets
        let (input, curl_para_cont) = match get_curl_brack_body(input) {
            Ok(v) => v,
            Err(q) => panic!(),
        };
        // ////println!("curl_para_cont in if: {:?}", curl_para_cont);
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
        Err(q) => panic!(),
    };

    // ////println!("curl_para_cont out if: {:?}", curl_para_cont);
    //Puts and returns function arguments in tree
    let function_arg = function_def_parentheses_parser_final(paren_cont);
    //puts and returns function elements(function body) in tree
    let function_elements = function_body_elements(curl_para_cont);
    let box_name = Box::new(String::from(varname));
    let function = function::parameters_def(box_name, function_arg, return_type, function_elements);
    //println!("rest of input: {:?} ", input);
    return Ok((input, List::func(function)));
}

//Parses the condition, and the body of if statements and while loops and adds them to the tree
fn if_parser(input: &str) -> IResult<&str, expr> {
    println!("Entered if_parser");
    println!("with input: {:?}",&input);
    let (input, _) = preceded(multispace0, tag("if"))(input)?;

    let (input, paran_cont) = get_parentheses_content(input)?;
    let (_, pibresult) = put_in_box(paran_cont)?;
    let (input, curl_para_cont) = get_curl_brack_body(input)?;
    println!("if get curl brack body result {:?}", (&input, &curl_para_cont));
    let function_elements = function_body_elements(curl_para_cont);
    println!("if function_body_elements result {:?}",function_elements);

    let pibresult = match pibresult {
        expr::list(a) => a,
        _ => panic!("wrong in condition"),
    };

    let list = if_enum::condition(Box::new(pibresult), function_elements);
    let (input, _) = tag(";")(input)?;

    Ok((input, expr::if_enum(list)))
}

//parses while loops
fn while_parser(input: &str) -> IResult<&str, expr> {
    //println!("Entered while_parser");
    //println!("with input: {:?}",&input);
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
    let (input, _) = tag(";")(input)?;

    Ok((input, expr::while_enum(list)))
}
//Parses a return statement in a function, and returns it as an IResult.
fn function_call_return_parser(input: &str) -> IResult<&str,expr>{
    let (input,_) = preceded(multispace0,tag("return"))(input)?;
    //println!( "After tag {:?}", &input);
    let (input, pibresult) = put_in_box(input)?;
    //println!( "After Put_in_box {:?}", (&input,&pibresult));
    let pibresult = match pibresult {
        expr::list(a) => a,
        _ => panic!("wrong in condition"),
    };

    let pibresult = variable_value::boxs(Box::new(pibresult));  



    Ok((input, expr::return_val(pibresult)))
}
