

use std::vec;

use itertools::Itertools;

use crate::{ALError, Array, Func, GenericVal, Result, Val, Verb};

use super::{eval_dyd, Shape};

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

pub fn scan<T: GenericVal + TryFrom<Val>>(u: Verb, mut y: Array<T>) -> Result<Val> 
where Array<T>: Into<Val> 
{
    if y.shape[0] == 1{
        Ok(y.into())
        /*
    } else if y.rank() == 1 {
        let mut accum: Val = y.data[0].clone().into();
        let mut same = true;
        let mut outVal: Vec<Val> = Vec::new();
        let mut out: Val = Array { data: vec![y.data[0].clone()] , shape: vec![1]}.into();

        for i in 1..y.shape[0] {
            let i = i as usize;
            accum = eval_dyd(
                    u.clone(), 
                    accum,
                    y.data[i].clone().into()
                )?;

            out = out.join(accum.clone())?;
        }
        Ok(out)
*/
    } else {
        let s: Array<T> = Array {
            data: y.cell(0).to_vec(),
            shape: y.cell_shape(),
        };

        let mut out: Val = Array { 
            data: vec![s.clone().into()],
            shape: {let mut ys = y.shape.clone(); ys[0] = 1; ys}
        }.into();
        let mut accum: Val = s.into();

        for i in 1..y.shape[0] {
            let i = i as i64;
            //let a: Val = Array {
                //data: y.cell(i-1).to_vec().into(),
                //shape: y.cell_shape(),
            //}.into();
            let a = Val::Int(i).select(y.clone().into());
            accum = eval_dyd( u.clone(), accum, a)?;
            out = out.join(accum.clone())?;
        }
        Ok(Array::from(out).into())
    }

}


pub fn fold<T: GenericVal>(u: Verb, mut y: Array<T>) -> Result<Val> 
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
