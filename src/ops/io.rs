use std::{env, fs, io::Read, os};

use itertools::Itertools;

use crate::{ALError, Array, Func, Val, Verb};


pub trait IoOps {
    fn read(y: Val) -> Result<Val, ALError>; 
}

impl IoOps for Val {
    fn read(y: Val) -> Result<Val, ALError> {
        println!("cwd: {:?}", env::current_dir());
        if let Val::AsciiArr(s) = y {
            let mut buf = String::new();
            let str: String = s.try_into()?;
            fs::File::open(str)?.read_to_string(&mut buf)?;
            Ok(Array::from(buf.chars().collect_vec()).into())
        } else {
            Err(ALError::Type(format!("val not stringable: {y:?}")))
        }
    }
    
}
