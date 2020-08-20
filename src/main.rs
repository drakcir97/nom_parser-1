#![allow(non_snake_case)]
#![allow(unused_imports)]

mod parser;
mod interpreter;
mod typechecker;
mod enums;

fn main() {
    //let k = put_in_box("-3");
    //let list = Box::leak(k);
    //  let  strinf = "abcd";

    //    println!("{:?}", varib);
    // let z: u8 = 0;
    //2+3
    // let varib = "ab,ab,ab,ef";
    // let x: IResult<&str,Vec<&str>> = many0(
    // delimited(
    // take(z),
    // take_until(","),
    // tag(","),
    // )val.pop()
    // )(varib);
// 
    // let x = parser::put_in_box("1+(2-(3/9)+2);");
// 
    //let x = parser::put_in_box("1+2-trst+2;");

    // let x = put_in_box("let x: i32 = 6+7;");
    
    //let x = variable_parser("let x: i32 = 6+7;");
    //let input = "(a+b)";
    //let x: IResult<&str, &str> = delimited(tag("("), take_until(")"), tag(")"))(input);

    // let x = function_parser(
    //  "fn getfunkbody(input: i32) -> i32{
    //   let z:i32 = 9;
    //   if (1>2) {
    //   let x: i32 = 4;
    //   let q: i32 = x+2;
    //   };
    //  }"
    // );

    let x = parserun(
        "fn getfunkbody(input: i32) -> i32{
            let z:i32 = 9;
            return input+1;
        }

        fn main(input: i32) -> i32{
            let testInt:i32 = 1;
            let testVar:bool = 1 > 4;
            getfunkbody(testInt);
            let y:i32 = 5;
            while (y>2){
                let y: i32 = y-1;
            }
        }
        "
        );
// 
    
// 
    // let x = if_parser("if (1+2){
    // let x:i32 = 5;
    // let y: i32 =18*7;
    // if (2>7){
    // let test: i32 = 9;
    // };
    // };
    // ");
    //let x:i32 = 1*(2+3)/5

    // let x = get_curl_brack_body(
    // "{
    // let x = 5;
    // let k = 9;
    // if true {
    // if jdad {
    // let banna = false;
    // let apple = true;
    // };
    // let i = 89;
    // let ifthing = 7;
    // };
    // }",
    // );
    println!("{:?}", x);

    // let x = get_curl_brack_body("{
    // let x = 5;
    // let b = 3+6+7;
}

fn parserun(st : &str) {
    let parsed = parser::program_parser(st);
    println!("parsed: {:?} \n\n",parsed);
    typechecker::typechecker(parsed.clone());
    let result = interpreter::execute(parsed.clone());

    let iter = result.iter();
    for line in iter {
        println!("Program state \n\n {:?}",line);  
    }
}