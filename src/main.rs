#![allow(unused_variables,unused_imports)]

use string_interner::{backend::{BucketBackend, StringBackend}, StringInterner};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor};
use colored::Colorize;

use std::{collections::HashMap, fmt::Debug, io::stdout, os::fd::AsRawFd, result};
use std::io::{stderr, Write};
use std::{iter::Zip, ops::Add, slice::Iter, vec};


use std::{io, ops::*};
use eval::eval;
use value::*;
use verb::*;

use crate::lexer::lex;
use crate::verb::PrimVerb;
use itertools::Itertools;

use nix::{ioctl_read, ioctl_read_bad, libc::{termios, termios2, winsize, TIOCGWINSZ}, sys};

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

fn repl() -> rustyline::Result<()> {
    let inp = io::stdin();
    let oerr = io::stderr();
    let mut buf = String::new();
    let mut env = Env{
        names: HashMap::new(),
        syms: StringInterner::<BucketBackend>::new(),
    };
    let mut rl = DefaultEditor::new()?;

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline("\x1b[48;5;46m \x1b[0m");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let words = lex(&line);
                println!("lex: {words:?}");
                match  eval(words, &mut env){
                    Err(e) => eprintln!("err: {e:?}"),
                    Ok(Some(v)) => println!("res: {v}"),
                    _ => {},
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
    rl.save_history("history.txt");

    Ok(())
}

pub struct Env {
    pub names: HashMap<String, Val>,
    pub syms: StringInterner<BucketBackend>,
}

/*
[│┃┏[]⎡1

⎡1
[1 2 3 = ⎢2
⎣3
*/

ioctl_read_bad!(termsize, TIOCGWINSZ, winsize);

#[derive(Debug)]
pub enum ALError {
    Syntax,
    Value(String),
    Type(String),
    Shape(String),
    IO(io::Error),
}

impl ALError {
    fn as_Type<T>(msg: &str) -> Result<T> {
        Err(ALError::Type(String::from(msg)))
    }
    fn as_Value<T, S: ToString>(msg: S) -> Result<T> {
        Err(ALError::Type(msg.to_string()))
    }
    fn as_Shape<T, S: ToString>(msg: S) -> Result<T> {
        Err(ALError::Shape(msg.to_string()))
    }
}


type Result<T> = result::Result<T, ALError>;

impl From<io::Error> for ALError {
    fn from(v: io::Error) -> Self {
        Self::IO(v)
    }
}
