//#![feature(trace_macros)]
use itertools::{Format, Itertools};
use num::{abs, cast::AsPrimitive, traits::ops::overflowing::OverflowingMul, PrimInt};
use string_interner::{backend::{BucketBackend, StringBackend}, StringInterner};
use core::fmt;
use std::{any::TypeId, fmt::{Display, Write}, ops::*, process::{id, Output}, vec::IntoIter};

type Symbol = string_interner::DefaultSymbol;

use crate::{eval::Token, verb::Verb, ALError, Adverb, Conj, PrimConj};

#[derive(Debug, Clone, PartialEq)]
pub enum Func {
    V(Verb),
    A(Adverb),
    C(Conj),
}


#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Int(i64),
    Float(f64),
    Sym(Symbol),
    AsciiArr(Array<u8>),
    IntArr(Array<i64>),
    FloatArr(Array<f64>),
    ValArr(Array<Val>),
    ValFunc(Func),
    Unit(Box<Val>),
    SymArr(Array<Symbol>),
}

impl TryFrom<Array<u8>> for String {
    type Error = ALError;
    fn try_from(Array { data, shape }: Array<u8>) -> Result<Self, Self::Error>{
        if shape.len() > 1 {
            Err(ALError::Shape(format!("cannot make string from rank {:?} Array", shape.len())))
        } else {
            String::from_utf8(data).map_err(|e| ALError::Type(format!("invalid utf8: {e}")))
        }
    }
}


impl From<Box<Val>> for Val {
    fn from(y: Box<Val>) -> Self { Val::Unit(y) }
}

impl From<i64> for Val {
    fn from(y: i64) -> Self { Val::Int(y) }
}

impl From<f64> for Val {
    fn from(y: f64) -> Self { Val::Float(y) }
}

impl From<Array<f64>> for Val {
    fn from(y: Array<f64>) -> Self { Val::FloatArr(y) }
}

impl From<Array<i64>> for Val {
    fn from(y: Array<i64>) -> Self { Val::IntArr(y) }
}

impl From<Array<u8>> for Val {
    fn from(y: Array<u8>) -> Self { Val::AsciiArr(y) }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Array<T> {
    pub data: Vec<T>,
    pub shape: Vec<u32>,
}

impl <T> Array<T> {
    pub fn cell(&self, idx: i64) -> &[T] {
        let &Array { data, shape } = &self;
        let l = shape[0] as i64;
        if abs(idx) >= l {
            panic!("bad index");
        }

        let idx = if idx < 0 {
            l + idx 
        } else {
            idx 
        } as usize;

        let step = if shape.len() == 1 { 1 } 
        else { shape[1..].iter().product::<u32>() as usize };
        &data[idx * step.. (idx + 1) * step]
    }
}




#[derive(Debug)]
pub enum ValueError {
    Conversion,
}

impl <T: Into<Val>> FromIterator<T> for Array<T>
    where Array<T>: From<Vec<T>>
{
    fn from_iter<IT: IntoIterator<Item = T>>(iter: IT) -> Self {
        iter.into_iter().collect::<Vec<T>>().into()
    }
}

impl From<i64> for Array<f64> {
    fn from(i: i64) -> Self {
        Array {
            data: vec![i as f64],
            shape: vec![1],
        }
    }
}

impl From<Array<i64>> for Array<f64> {
    fn from(Array { data, shape }: Array<i64>) -> Self {
        Array {
            data: data.into_iter().map(i64::as_).collect_vec(),
            shape,
        }
    }
}

impl<T> IntoIterator for Array<T> {
    type Item = T;
    type IntoIter = IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

macro_rules! impl_from_arr {
    ($($tp:ty),+) => {
        $(
        impl From<$tp> for Array<$tp> {
            fn from(y: $tp) -> Self {
                Array {
                    data: vec![y],
                    shape: vec![1],
                }
            }
        }
        impl From<Vec<$tp>> for Array<$tp> {
            fn from(y: Vec<$tp>) -> Self {
                Array {
                    shape: vec![y.len() as u32],
                    data: y,
                }
            }
        }
        impl From<&[$tp]> for Array<$tp> {
            fn from(y: &[$tp]) -> Self {
                Array {
                    data: y.to_vec(),
                    shape: vec![y.len() as u32],
                }
            }
        })+
    };
}
impl_from_arr!(f64, i64, u8);
