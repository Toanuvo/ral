use core::panic;
use std::{fmt::Debug, iter::{once, repeat, zip}, ops::{self, Index, Mul, Range, RangeBounds, ShlAssign}, process::id, usize, vec};

use crate::{Val, Array, Result, ALError};
use itertools::{repeat_n, Itertools};
use nix::libc::group;
use num::{abs, iter::{self}, Float};
use Val::*;


pub trait Shape {
    fn shape_mon(y: Val) -> Val;
    fn shape_ref(&self) -> &Vec<u32>;
    fn shape_dyd(x: Val, y: Val) -> Val;

    fn first(self) -> Self;
    fn last(self) -> Self;
    fn first_cell(self) -> Self;
    fn last_cell(self) -> Self;
    fn take(self, y: Val) -> Self;
    fn drop(self, y: Val) -> Self;
    //fn tail(self) -> Self;
    //fn curtail(self) -> Self;
    fn select(self, y: Val) -> Self;
    fn pick(self, y: Val) -> Self;
}

pub trait Select where Self: Sized {
    fn group(self, y:Self) -> Result<Self>;
}

fn  index<T: Into<Val> + Clone>(Array { data, shape }: &Array<T>, idx: usize, cells: bool) -> Val 
where Array<T>: Into<Val>
{
    if cells {
        if shape.len() == 1 {
            let v = data[idx].clone().into();
            Val::Unit(Box::new(v))
        } else {
            let nsh = &shape[1..];
            let step = shape[1..].iter().product::<u32>() as usize;
            Array {
                data: data[idx * step .. ][0..step].to_vec(),
                shape: shape[1..].to_vec() 
            }.into()
        }
    } else {
        data[idx].clone().into()
    }
}

fn  pick<T: Into<Val> + Clone>(Array { data, shape }: &Array<T>, idx: Vec<i64>) -> Val 
where Array<T>: Into<Val> 
{
    if shape.len() != idx.len() {
        panic!("bad index");
    }
    let i: i64 = idx.into_iter()
        .zip_eq(shape)
        .map(|(mut i, &s)| {
            let s = s as i64;
            let r = -s..s;
            if !r.contains(&i) {
                panic!("bad")
            }
            if i < 0  {
                i += s;
            }
            i * s
    }).sum();
    data[i as usize].clone().into()
}

fn  select<T: Into<Val> + Clone>(a: Array<T>, idx: Array<i64>) -> Val 
where Array<T>: Into<Val> 
{
    if a.shape.len() == 1 && idx.shape.len() == 1 && idx.shape[0] == 1 {
        return Box::new(a.cell(idx.data[0])[0].clone().into()).into();
    }

    let mut data = Vec::new();
    for i in idx.data {
        data.extend_from_slice(a.cell(i))
    }

    let mut shape = idx.shape.clone();
    if a.shape.len() > 1 {
        shape.extend_from_slice(&a.shape[1..]);
    }

    Array {
        data,
        shape
    }.into()
}

fn  take<T: Into<Val> + Clone + Default>(Array { data, shape }: Array<T>, v: Vec<i64>) -> Val 
where Array<T>: Into<Val> 
{
    if let [i] = v[..] {
        let cyc = once(T::default()).cycle();
        let step = if shape.len() == 1 { 1 } 
        else { shape[1..].iter().product::<u32>() as usize };

        let l = shape[0] as i64;
        let mut shape = shape.clone();

        let data = if abs(i) > l {
            let extra = (l - abs(i)) as usize;
            let v = data.clone();
            shape[0] += extra as u32;
            let extra = cyc.take(extra);
            if i < 0 {
                extra.chain(data.into_iter()).collect_vec()
            } else {
                data.into_iter().chain(extra).collect_vec()
            }
        } else if i < 0 {
            let i = (l + i) as usize;
            shape[0] -= i as u32;
            data[i*step..].to_vec()
        } else {
            shape[0] = i as u32;
            let i = i as usize;
            data[0..i*step].to_vec()
        };

        Array {
            data,
            shape
        }.into()
    } else {
        panic!("nyi")
    }
}

fn  drop<T: Into<Val> + Clone + Default>(Array { data, shape }: Array<T>, v: Vec<i64>) -> Val 
where Array<T>: Into<Val> 
{
    if let [i] = v[..] {
        let l = shape[0] as i64;
        let step = if shape.len() == 1 { 1 } 
        else { shape[1..].iter().product::<u32>() as usize };
        let mut shape = shape.clone();

        let data = if abs(i) > l {
            shape.fill(0);
            Vec::new()
        } else if i < 0 {
            let i = (l + i) as usize;
            shape[0] += i as u32;
            data[0..i*step].to_vec()
        } else {
            let i = i as usize;
            shape[0] -= i as u32;
            data[i*step..].to_vec()
        };
        Array {
            data,
            shape
        }.into()
    } else {
        panic!("nyi")
    }
}

impl Select for Val {
    fn group(self, y:Self) -> Result<Self> {
        match self {
            Int(x) => {
                if let Unit(u) = y {
                     group(
                        x.into(),
                        Array { data: vec![*u], shape: vec![1] }
                    )
                } else {
                    ALError::as_Type(format!("y must be an array, got x: {x}, y: {y}"))
                }
            },
            IntArr(x) => match y {
                IntArr(a) => group(x, a),
                FloatArr(a) => group(x, a),
                AsciiArr(a) => group(x, a),
                ValArr(a) => group(x, a),
                Unit(u) => group(x, Array { data: vec![*u], shape: vec![1] }),
                _ => ALError::as_Type(format!("y must be an array, got {y}")),
            }
            x => ALError::as_Value(format!("cannot group using: {}", x)),
        }
    }
}

fn group<T: Debug + Into<Val> + Clone>(x: Array<i64>, y: Array<T>) -> Result<Val> 
where Array<T>: Into<Val> 
{
    let xs = x.shape;
    if xs.len() > 1 {
        panic!("nyi");
    } else if xs[0] < y.shape[0] {
        return ALError::as_Shape("group: x len must be equal to y len");
    }
    if xs[0] > y.shape[0] { 
        panic!("nyi");
    }

    let mut res: Vec<Array<T>> = Vec::new();

    for i  in  0..xs[0] { 
        let v = y.cell(i as i64);
        let i = x.data[i as usize];
        if i != -1 {
            let i = i as usize;
            if res.len() <= i {
                res.resize(i + 1, Array::default());
            }
            let a = &mut res[i];
            a.data.extend_from_slice(v);
            if y.shape.len() > 1 && a.shape[0] == 0 {
                a.shape.extend_from_slice(&y.shape[1..]);
            }
            a.shape[0] += 1;
            dbg!(&res);
        }
    }

    let res = res.into_iter()
        .map_into::<Val>()
        //.map(|v| Val::Unit(Box::new(v)))
        .collect_vec();
    Ok(Array::from(res).into())
}

impl Shape for Val {
    fn pick(self, y: Val) -> Self {
        match self {
            Int(x) => match y {
                IntArr(a) => pick(&a, vec![x]),
                FloatArr(a) => pick(&a, vec![x]),
                _ => panic!("nyi"),
            },
            IntArr(x) if x.shape.len() == 1 => match y {
                IntArr(a) => pick(&a, x.data),
                FloatArr(a) => pick(&a, x.data),
                Unit(u) if x.shape[0] == 0 => *u,
                _ => panic!("nyi"),
            }
            _ => panic!("cannot pick using {self:?}"),
        }
    }

    fn select(self, y: Val) -> Self {
        match self {
            Int(x) => match y {
                IntArr(a) => select(a, x.into()),
                FloatArr(a) => select(a, x.into()),
                _ => panic!("nyi"),
            },
            IntArr(x) => match y {
                IntArr(a) => select(a, x),
                FloatArr(a) => select(a, x),
                _ => panic!("nyi"),
            },
            ValArr(x) => panic!("nyi"),
            _ => panic!("cannot select using {self:?}"),
        }
    }

    fn first(self) -> Self {
        match self {
            IntArr(a)  =>  index(&a, 0, false),
            FloatArr(a)  =>  index(&a, 0, false),
            Unit(b) => *b,
            y@(Int(_) | Float(_)) => y,
            _ => todo!("nyi"),
        }
    }

    fn first_cell(self) -> Self {
        match self {
            IntArr(a)  =>  index(&a, 0, true),
            FloatArr(a)  =>  index(&a, 0, true),
            _ => todo!("nyi"),
        }
    }

    fn last(self) -> Self {
        match self {
            IntArr(a)  =>  index(&a, a.data.len() - 1, false) ,
            FloatArr(a)  =>  index(&a, a.data.len()-1, false) ,
            Unit(b) => *b,
            y@(Int(_) | Float(_)) => y,
            _ => todo!("nyi"),
        }
    }

    fn last_cell(self) -> Self {
        match self {
            IntArr(a)  =>  index(&a, (a.shape[0] - 1) as usize, true),
            FloatArr(a)  =>  index(&a, (a.shape[0] - 1) as usize, true),
            _ => todo!("nyi"),
        }
    }

    fn take(self, y: Val) -> Self {
        match self {
             Int(i) =>  match y {
                IntArr(a) => take(a, vec![i]),
                FloatArr(a) => take(a, vec![i]),
                Int(a) => take(vec![a].into(), vec![i]),
                Float(a) => take(vec![a].into(), vec![i]),
                _ => panic!("cannot take from {y:?}"),
            },
            IntArr(i) => match y {
                IntArr(a) => take(a, i.data),
                FloatArr(a) => take(a, i.data),
                Int(a) => take(vec![a].into(), i.data),
                Float(a) => take(vec![a].into(), i.data),
                _ => panic!("cannot take from {y:?}"),
            }
            _ => panic!("cannot take using {self:?}"),
        }
    }

    fn drop(self, y: Val) -> Self {
        match self {
             Int(i) =>  match y {
                IntArr(a) => drop(a, vec![i]),
                FloatArr(a) => drop(a, vec![i]),
                Int(a) => drop(vec![a].into(), vec![i]),
                Float(a) => drop(vec![a].into(), vec![i]),
                _ => panic!("cannot drop from {y:?}"),
            },
            IntArr(i) => match y {
                IntArr(a) => drop(a, i.data),
                FloatArr(a) => drop(a, i.data),
                Int(a) => drop(vec![a].into(), i.data),
                Float(a) => drop(vec![a].into(), i.data),
                _ => panic!("cannot drop from {y:?}"),
            }
            _ => panic!("cannot drop using {self:?}"),
        }
    }

    fn shape_dyd(x: Val, y: Val) -> Val {
        panic!("nyii");
    }

    fn shape_ref(&self) -> &Vec<u32> {
        // is there a better way?
        static EMPTY_SHAPE: Vec<u32> = Vec::new();
        match self {
            IntArr(Array { data: _, shape }) | 
            AsciiArr(Array { data: _, shape }) | 
            FloatArr(Array { data: _, shape }) => {
                shape
            },
            Int(_) | Float(_) => {
                &EMPTY_SHAPE
            } ,
            _ => panic!("nyi"),
        }
    }

    fn shape_mon(y: Val) -> Val {
        match y {
            IntArr(Array { data: _, shape }) | 
            ValArr(Array { data: _, shape }) | 
            AsciiArr(Array { data: _, shape }) | 
            FloatArr(Array { data: _, shape }) => {
                let data = shape.into_iter()
                .map(|i| i as i64)
                .collect_vec();
                Array { shape: vec![data.len() as u32], data }.into()
            },
            Int(_) | Float(_) => Array { shape: vec![0], data: Vec::<i64>::new() }.into(),
            _ => panic!("nyi: {y:?}"),
        }
    }
}

pub trait Length {
    fn length_mon(y: Val) -> Val;
    fn length_dyd(x: Val, y: Val) -> Val;
}

impl Length for Val {
    fn length_dyd(x: Val, y: Val) -> Val {
        panic!("nyi");
    }

    fn length_mon(y: Val) -> Val {
        use Val::*;
        match y {
            IntArr(Array { data: _, shape }) | 
            AsciiArr(Array { data: _, shape }) | 
            FloatArr(Array { data: _, shape }) => Int(shape[0] as i64),
            Int(_) | Float(_) => Int(1),
            _ => panic!("nyi"),
        }
    }
}

pub trait Rank {
    fn rank(x: &Val) -> Val {
        use Val::*;
        match x {
            Int(_) | Float(_) | Unit(_) => Int(0),
            IntArr(Array { data: _, shape }) | 
            FloatArr(Array { data: _, shape }) => Int(shape.len() as i64),
            _ => panic!("nyi"),
        }
    }
}

impl Rank for Val {}
