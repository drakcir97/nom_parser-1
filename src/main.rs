#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unreachable_code)]
#![allow(non_camel_case)]

mod enums;
mod interpreter;
mod parser;
mod typechecker;
mod llvm;

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
    // let x = parser::put_in_box("1;");

    // let x = put_in_box("let x: i32 = 6+7;");

    // let x = parser::get_reg_brack_cont("(asd(test(1))+4);");
    //let x = parser::get_reg_brack_cont("(1-asd(5)-1;)");
    //let x = parser::get_curl_brack_body("{let testV:i32 = 1-asd(5)-1; }");

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
    //
    // let x = parserun(
    // "
    //     fn main(input: i32) -> i32{
    //         let testInp:i32 = 2;
    //         let testV:i32 = (1-asd(testInp+1))-1;
    //         return testV;
    //     }

    //     fn asd(input: i32)->i32{
    //         let test:i32 = 1;
    //         while (test < 5) {
    //             test := test+1;
    //         }
    //         return test;
    //     }
    // "
    // );
    //

    // fn main(input: i32) -> i32{
    //     let testInp:i32 = 2;
    //     let testV:i32 = (1-asd(testInp))-1;
    //     return testInp;
    // }
    // fn asd(input: i32)->i32{
    //     return input;
    // }

    //
    // let x = parser::if_parser("if (1<2){
    //     return 0+1;
    // }");
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

    // let x = get_curl_brack_body("{
    // let x = 5;
    // let b = 3+6+7;

    let x = parserun(
        "
        fn main(input: i32) -> i32 {
            let  tofunc:i32 = 5;
            if (funcbool(true)) {
                return funcloop (1) + funcstate(tofunc);
            }
            return  0;
        }
        
        fn  funcloop(input: i32) -> i32 {
            while (input <4) {
                input :=  input +1;
            }
            return  input;
        }
        
        fn  funcstate(input: i32) -> i32 {
            let  val:i32 = 1;
            if (val <input) {
                return  val;
            }
            return  input;
        }

        fn  funcbool(input: bool) -> bool {
            let  bval: bool = false;
            if (input  || bval) {
                return  true;
            }
            return  false;
        }"
    );

    println!("{:?}", x);
}

fn parserun(st: &str) {
    println!("---------- PARSER ----------\n");
    let parsed = parser::program_parser(st);
    println!("parsed: {:?} \n", parsed);

    println!("---------- TYPECHECKER ----------\n");

    typechecker::typechecker(parsed.clone());
    
    println!("---------- INTERPRETER ----------\n");

    let result = interpreter::execute(parsed.clone());

    let iter = result.iter();
    for line in iter {
        println!("Program state \n\n {:?}", line);
    }

    println!("---------- LLVM ----------\n");

    llvm::execute(parsed.clone());
}
