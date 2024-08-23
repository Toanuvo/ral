
use itertools::Itertools;

use crate::{ALError, Array, Func, Val, Verb};

pub trait Fold {
    //fn fold_dyd(v: Verb, x: Val, y: Val ) -> Val;
    fn fold_mon(v: Verb, x: Val) -> Result<Val, ALError>;
}

impl Fold for Val {
    //fn fold_dyd(v: Verb, x: Val, y: Val ) -> Val { panic!("nyi"); }

    fn fold_mon(v: Verb, y: Val) -> Result<Val, ALError> {
        match y {
            Val::IntArr(a) => fold(v, a),
            Val::FloatArr(a) => fold(v, a),
            Val::Sym(a) => Err(ALError::Type("cannot fold sym".to_string())),
            //Val::ValArr(a) => fold(u, a),
            y@(Val::Int(_) | Val::Float(_)) => Ok(y),
            _ => panic!("todo"),
        }
    }
}


pub fn fold<T: Into<Val>>(u: Verb, mut y: Array<T>) -> Result<Val, ALError> 
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
