
use crate::{Func, Val, Verb, Array};

pub trait Fold {
    fn fold_dyd(v: Verb, x: Val, y: Val ) -> Val;
    fn fold_mon(v: Verb, x: Val) -> Val;
}
impl Fold for Val {
    fn fold_dyd(v: Verb, x: Val, y: Val ) -> Val {
        panic!("nyi");
    }

    fn fold_mon(v: Verb, y: Val) -> Val {
        match y {
            Val::IntArr(a) => fold(v, a),
            Val::FloatArr(a) => fold(v, a),
            //Val::ValArr(a) => fold(u, a),
            y@(Val::Int(_) | Val::Float(_)) => y,
            _ => panic!("todo"),
        }
    }
}

pub fn fold<T: Into<Val>>(u: Verb, mut y: Array<T>) -> Val 
where Array<T>: Into<Val>
{
    if y.data.len() == 1 {
        return y.into();
    }
    if y.shape.len() > 1 {
        panic!("fold non vector");
    }
    let init: Val = y.data.pop().unwrap().into();
    y.data.into_iter()
        .fold(init, |acc, y| 
            super::eval_dyd(u.clone(), acc, y.into()))
}
