#![allow(unused_variables,unused_imports)]
mod shape;
mod arith;
mod adverb;

use crate::{Func, Val, Array, Verb, Adverb, PrimAdv, PrimConj, PrimVerb};
use crate::PrimConj::*;
use crate::PrimVerb::*;
use crate::PrimAdv::*;

use shape::*;
use arith::*;
use adverb::*;

pub fn eval_mon(v: Verb,  y: Val) -> Val {
    use Val::*;
    match v {
        Verb::Id(x) => *x,
        Verb::Adv { u, p } => eval_mon_adv(*u, p, y),
        Verb::Conj { u, p, v } => match p {
            ampr_dot => {
                let x = eval_mon(*u, y.clone());
                eval_dyd( *v, x, y)
            }
            ampr_col => {
                let x = eval_mon(*v, y.clone());
                eval_dyd( *u, y, x)
            }
            _ => panic!("todo"),
        },
        Verb::Fork { f, g, h } => {
            eval_dyd(*g ,
                eval_mon(*f, y.clone()),
                eval_mon(*h, y.clone())
            )
        },
        Verb::Prim(p) => match p {
            excl => Val::til_mon(y),
            dllr => Val::shape_mon(y),
            hash => Val::length_mon(y),
            hash_col => Val::rank(&y),
            lbrak | rbrak => y,
            _ => panic!("todo"),
        },
        _ => panic!("todo"),
    }
}

pub fn eval_dyd(v: Verb, x: Val, y: Val) -> Val {
    use Val::*;
    match v {
        Verb::Id(x) => *x,
        Verb::Prim(p) => match p {
            plus => x + y,
            pcnt => x / y,
            star => x * y,
            dash => x - y,
            larr => x.lt(y),
            rarr => x.gt(y),
            lbrak => x,
            rbrak => y,
            _ => panic!("nyi"),
        },
        Verb::Conj { u, p, v } => match p {
            ampr_dot => {
                let x = eval_mon(*u, x);
                eval_dyd(*v, x, y)
            }
            ampr_col => {
                let y = eval_mon(*v, y);
                eval_dyd(*u, x, y)
            }
            _ => panic!("todo"),
        },
        _ => panic!("todo"),
    }
}


fn eval_mon_adv(u: Verb, a: PrimAdv, y:Val) -> Val {
    match a  {
        slsh => Val::fold_mon(u, y),
        _ => panic!("todo"),
    }
}

