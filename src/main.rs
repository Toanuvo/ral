#![allow(unused_variables,unused_imports)]

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

use std::collections::HashMap;
use std::io::{stderr, Write};
use std::{iter::Zip, ops::Add, slice::Iter, vec};

use std::{io, ops::*};
use eval::eval;
use value::*;
use verb::*;

use crate::lexer::lex;
use crate::verb::PrimVerb;
use itertools::Itertools;

mod value;
mod verb;
mod lexer;
mod eval;
mod ops;

fn main() {
    let x = Val::Int(5);
    let xaf = Val::FloatArr(Array {
        data: vec![1.1, 2.2, 3.3],
        shape: vec![3],
    });

    let yaf = Val::FloatArr(Array {
        data: vec![4.4, 0.5, 3.5, 4.4, 2.5, 6.6],
        shape: vec![3, 2],
    });
    let xai = Val::IntArr(Array {
        data: vec![4, 5, 6, 4, 5, 6],
        shape: vec![3, 2, 1],
    });
    let ya = Val::IntArr(Array {
        data: vec![1, 2, 3],
        shape: vec![3],
    });
    //let rf: Array<f64> = xaf * yaf;
    //let r: Array<i64> = xai * ya;
    //print!("{:?}", x.gt(yaf));

    //let code = "1+5&:";
    //let words = lexer::lex(code);
    //eval::eval(words);
    repl().expect("");
}

fn repl() -> Result<()> {
    let inp = io::stdin();
    let oerr = io::stderr();
    let mut buf = String::new();
    let mut env: HashMap<String, Val> = HashMap::new();
    let mut rl = DefaultEditor::new()?;

    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline("\x1b[48;5;46m \x1b[0m");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let words = lex(&line);
                if let Err(e) = eval(words, &mut env) {
                    eprintln!("err: {e:?}");
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }

    #[cfg(feature = "with-file-history")]
    rl.save_history("history.txt");
    Ok(())
}



#[derive(Debug, Clone, PartialEq)]
pub enum ALError {
    Syntax,
    Value(String),
}
