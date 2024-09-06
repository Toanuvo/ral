

use std::vec;

use itertools::Itertools;

use crate::{Result, ALError, Array, Func, Val, Verb};

use super::eval_dyd;

pub trait Fold where Self: Sized {
    //fn fold_dyd(v: Verb, x: Val, y: Val ) -> Val;
    fn fold_mon(v: Verb, x: Val) -> Result<Val>;
    fn scan(v: Verb, x: Self) -> Result<Self>;
}

impl Fold for Val {
    //fn fold_dyd(v: Verb, x: Val, y: Val ) -> Val { panic!("nyi"); }

    fn fold_mon(v: Verb, y: Val) -> Result<Val> {
        match y {
            Val::IntArr(a) => fold(v, a),
            Val::FloatArr(a) => fold(v, a),
            Val::Sym(a) => Err(ALError::Type("cannot fold sym".to_string())),
            //Val::ValArr(a) => fold(u, a),
            y@(Val::Int(_) | Val::Float(_)) => Ok(y),
            _ => panic!("todo"),
        }
    }

    fn scan(v: Verb, y: Self) -> Result<Self> {
        match y {
            Val::IntArr(a) => scan(v, a),
            Val::FloatArr(a) => scan(v, a),
            Val::AsciiArr(a) => scan(v, a),
            Val::ValArr(a) => scan(v, a),
            _ => panic!("todo scan: {y:?}"),
        }
    }
}

pub fn scan<T: Into<Val> + Clone + TryFrom<Val>>(u: Verb, mut y: Array<T>) -> Result<Val> 
where Array<T>: Into<Val> 
{
    if y.shape[0] == 1{
        Ok(y.into())
    } else if y.rank() == 1 {
        let mut accum: Val = y.data[0].clone().into();
        let mut same = true;
        let mut outVal: Vec<Val> = Vec::new();
        let mut out: Vec<T> = vec![y.data[0].clone()];

        for i in 1..y.shape[0] {
            let i = i as usize;
            accum = eval_dyd(
                    u.clone(), 
                    accum,
                    y.data[i].clone().into()
                )?;
            if same {
                //let t: std::Result<T, Val> = &accum .try_into();
            }
            
            //out.push(accum.clone());
        }
        //Ok(Array::from(out).into())
        Ok(Array::from(outVal).into())
    } else {
        let s: Array<T> = Array {
            data: y.cell(0).to_vec(),
            shape: y.shape[1..].to_vec(),
        };
        let mut out: Vec<Val> = vec![s.clone().into()];
        let mut accum: Val = s.into();

        for i in 1..y.shape[0] {
            let i = i as i64;
            let a: Val = Array {
                data: y.cell(i-1).to_vec().into(),
                shape: y.shape[1..].to_vec(),
            }.into();
            accum = eval_dyd( u.clone(), accum, a)?;
            out.push(accum.clone()); 
        }
        Ok(Array::from(out).into())
    }

}


pub fn fold<T: Into<Val>>(u: Verb, mut y: Array<T>) -> Result<Val> 
where Array<T>: Into<Val>
{
    if y.data.len() == 1 {
        Ok(y.into())
    } else if y.shape.len() > 1 {
        Err(ALError::Shape("fold non vector".to_string()))
    } else {
        let init: Val = y.data.pop().unwrap().into();
        y.data.into_iter()
            .try_fold(init, |acc: Val, y| 
                super::eval_dyd(u.clone(), acc, y.into()))
    }
}
