#![allow(unused_variables,unused_imports)]
mod shape;
mod arith;
mod adverb;
mod io;

use std::any::TypeId;

use crate::ops::io::IoOps;
use crate::{ALError, Adverb, Array, Func, PrimAdv, PrimConj, PrimVerb, Val, Verb};
use crate::PrimConj::*;
use crate::PrimVerb::*;
use crate::PrimAdv::*;

use shape::*;
use arith::*;
use adverb::*;

pub fn eval_mon(v: Verb,  y: Val) -> Result<Val, ALError> {
    use Val::*;
    Ok(match v {
        Verb::Id(x) => *x,
        Verb::Adv { u, p } => eval_mon_adv(*u, p, y)?,
        Verb::Conj { u, p, v } => match p {
            ampr_dot => {
                let x = eval_mon(*u, y.clone());
                eval_dyd( *v, x?, y)?
            }
            ampr_col => {
                let x = eval_mon(*v, y.clone());
                eval_dyd( *u, y, x?)?
            }
            _ => panic!("todo"),
        },
        Verb::Fork { f, g, h } => {
            eval_dyd(*g ,
                eval_mon(*f, y.clone())?,
                eval_mon(*h, y.clone())?
            )?
        },
        Verb::Prim(p) => match p {
            i_dot => Val::read(y)?,
            excl => Val::til_mon(y),
            dllr => Val::shape_mon(y),
            hash => Val::length_mon(y),
            hash_col => Val::rank(&y),
            lbrak | rbrak => y,
            _ => panic!("todo"),
        },
        _ => panic!("todo"),
    })
}



pub fn eval_dyd(v: Verb, x: Val, y: Val) -> Result<Val, ALError> {
   use Val::*;
    Ok(match v {
        Verb::Id(x) => *x,
        Verb::Prim(p) => match p {
            p@(plus | pcnt | star | dash | larr | rarr) => eval_arith(p, x, y)?,
            lbrak => x,
            rbrak => y,
            _ => panic!("nyi"),
        },
        Verb::Conj { u, p, v } => match p {
            ampr_dot => {
                let x = eval_mon(*u, x)?;
                eval_dyd(*v, x, y)?
            }
            ampr_col => {
                let y = eval_mon(*v, y)?;
                eval_dyd(*u, x, y)?
            }
            _ => panic!("todo"),
        },
        _ => panic!("todo"),
    })
}

fn eval_arith(p: PrimVerb, x: Val, y: Val) -> Result<Val, ALError> {
    use Val::*;
    if matches!(x, Sym(_) | ValFunc(_)) || matches!(y, Sym(_) | ValFunc(_)) {
        let msg = format!("cannot eval: {x:?} {p:?} {y:?}");
        Err(ALError::Type(msg))
    } else {
        Ok( match p {
            plus => x + y,
            pcnt => x / y,
            star => x * y,
            dash => x - y,
            larr => x.lt(y),
            rarr => x.gt(y),
            p => unreachable!("not arith dyd verb: {p:?}"),
        })
    }
}

fn eval_mon_adv(u: Verb, a: PrimAdv, y:Val) -> Result<Val, ALError> {
    Ok(match a  {
        slsh => Val::fold_mon(u, y)?,
        _ => panic!("todo"),
    })
}

