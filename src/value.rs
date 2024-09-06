//#![feature(trace_macros)]
use itertools::{intersperse, Format, Itertools};
use nix::NixPath;
use num::{abs, cast::AsPrimitive, range_step, traits::{ops::overflowing::OverflowingMul, SaturatingSub}, PrimInt, Saturating};
use rustyline::{line_buffer::WordAction, Word};
use string_interner::{backend::{BucketBackend, StringBackend}, StringInterner};
use core::fmt;
use std::{any::TypeId, fmt::{Debug, Display, Write}, isize, marker::PhantomData, ops::{self, *}, os::unix::fs::OpenOptionsExt, process::{id, Output}, u32, usize, vec::IntoIter};
use std::mem::{Discriminant, discriminant};
use colored::Colorize;

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
    Utf16Arr(Array<u16>),
    Utf32Arr(Array<u32>),
    IntArr(Array<i64>),
    FloatArr(Array<f64>),
    ValArr(Array<Val>),
    ValFunc(Func),
    Unit(Box<Val>),
    SymArr(Array<Symbol>),
}

/*

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GenericVal<T: From<Val> + Clone>(mem::Discriminant<Val>, T);

impl <T: Into<Val> + Clone> GenericVal<T> {
    fn of<G:Into<Val> + Clone>() -> Option<Self> {

        
    }
    
}

impl <T: Into<Val> + Clone, G: Into<Val> + Clone> From<G> for GenericVal<T> {
    fn from(v: G) -> Self {
        if type_id::<G>() == type_id::<T>() {

        }

        GenericVal::<T>(mem::discriminant(&v), PhantomData)
    }
}

fn type_cast<T: Into<Val> + Clone, G: Into<Val> +Clone>(y: G, t: Discriminant<Val>) -> Option<GenericVal<T>>{
    if type_id::<T>() == type_id::<G>() {
        Some(GenericVal(t, y))
    } else {
        None
    }
    
}

impl  <T: From<Val> + Clone> TryFrom<Val> for GenericVal<T> {
    type Error = ();
    fn try_from(v: Val) -> Result<Self, Self::Error> {
        use Val::*;
        match v {
            Int(y) => Ok(GenericVal(mem::discriminant(&v), y as T),)
            
        }
        
    }
    
}
*/

#[macro_export()]
macro_rules! is_arr {
    ($bind:ident) => {
        AsciiArr($bind) |
        IntArr($bind) |
        FloatArr($bind) |
        ValArr($bind)
    };
}

impl TryFrom<Array<u8>> for String {
    type Error = ALError;
    fn try_from(Array { data, shape }: Array<u8>) -> Result<Self, Self::Error>{
        if shape.len() > 1 {
            Err(ALError::Shape(format!("cannot make string from rank {:?} Array", shape.len())))
        } else {
            Ok(String::from_utf8(data).unwrap())
        }
    }
}


impl From<Vec<char>> for Val {
    fn from(y: Vec<char>) -> Self {
        match y.iter().max() {
            Some(c) => {
                let c = c.clone() as u32;
                if c <= u8::MAX as u32 {
                    Array {
                        shape: vec![y.len() as u32],
                        data: y.into_iter().map(|c| c as u8).collect_vec(),
                    }.into()
                } else if c <= u16::MAX as u32 {
                    Array {
                        shape: vec![y.len() as u32],
                        data: y.into_iter().map(|c| c as u16).collect_vec(),
                    }.into()
                } else {
                    Array {
                        shape: vec![y.len() as u32],
                        data: y.into_iter().map(|c| c as u32).collect_vec(),
                    }.into()
                }
            },
            None => Array::<u8>::default().into(),
        }
    }
}

impl From<Box<Val>> for Val {
    fn from(y: Box<Val>) -> Self { Val::Unit(y) }
}

macro_rules! impl_conv {
    ($($tag:ident-$tp:ty);+) => {
$(
        impl From<$tp> for Val {
        fn from(y: $tp) -> Self { Val::$tag(y) }
        }
        impl TryFrom<Val> for $tp {
        type Error = ();
        fn try_from(v: Val) -> Result<Self, Self::Error> {
        if let Val::$tag(y) = v {
        Ok(y)
        } else {
        Err(())
        }
        }
        }
)+
    };
}

impl_conv!(
    Int-i64;
    Float-f64;
    AsciiArr-Array<u8>;
    IntArr-Array<i64>;
    FloatArr-Array<f64>
);

impl From<char> for Val {
    // todo char val?
    fn from(y: char) -> Self { Val::Int(y as i64) }
}

impl From<u8> for Val {
    // todo char val?
    fn from(y: u8) -> Self { Val::Int(y as i64) }
}

impl TryFrom<Val> for u8 {
    // todo char val?
    type Error = ();
    fn try_from(v: Val) -> Result<Self, Self::Error> {
        if let Val::Int(y) = v {
            u8::try_from(y).map_err(|_|())
        } else {
            Err(())
        }
    }
}

impl From<Array<u16>> for Val {
    fn from(y: Array<u16>) -> Self { Val::Utf16Arr(y) }
}

impl From<Array<u32>> for Val {
    fn from(y: Array<u32>) -> Self { Val::Utf32Arr(y) }
}

/*
impl From<Array<char>> for Val {
    fn from(y: Array<char>) -> Self { Val::AsciiArr(y) }
}
*/

impl From<Array<Val>> for Val {
    fn from(y: Array<Val>) -> Self { Val::ValArr(y) }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Array<T> {
    pub data: Vec<T>,
    pub shape: Vec<u32>,
}

impl <T> Default for Array<T> {
    fn default() -> Self {
        Array { data: vec![], shape: vec![0] }
    }
}

impl <T> Array<T> {
    pub fn rank(&self) -> usize {
        self.shape.len()
    }

    pub fn is_single(&self) -> bool {
        self.shape.len() == 1 && self.shape[0] == 1
    }

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

    pub fn cast<G: Into<Val> + From<T>>(self) -> Array<G> {
        Array { data: self.data.into_iter().map(G::from).collect_vec(), shape: self.shape }
    }
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Val::*;
        match self {
            Int(y) => f.write_fmt(format_args!("{}", y)),
            Unit(y) => f.write_fmt(format_args!("<{}>", y)),
            IntArr(y) => f.write_fmt(format_args!("{}", y)),
            ValArr(y) => f.write_fmt(format_args!("v{}v", y)),
            FloatArr(y) => f.write_fmt(format_args!("{}", y)),
            AsciiArr(y) => {
                if y.shape.len() == 1 {
                    let s = std::str::from_utf8(&y.data[..]).unwrap().escape_default();
                    let s = s.to_string().blue();
                    f.write_fmt(format_args!("\"{s}\""))
                } else {
                    f.write_fmt(format_args!("{}", y))
                }
            },
            y => f.write_fmt(format_args!("{:?}", y)),
        }
    }
}

type Grid<T = char> = Vec<Vec<T>>;
type Metagrid = Grid<Grid>;
//let mut sz: winsize = winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
//let ok = unsafe { termsize(stdout().as_raw_fd(), &mut sz) };

impl <T: Display> fmt::Display for Array<T> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const summary_insert: &str = "...";
        const edge_items: usize = 3;
        const separator: &str = " ";
        const line_width: usize = 75;

        let mut idx: Vec<_> = Vec::new();
        
        fn recur<T: Display>(arr: &Array<T>, f: & mut fmt::Formatter<'_>, index: &mut Vec<usize>, hanging_indent: String, curr_width: usize) -> Result<String, fmt::Error> {
            let axis = index.len();
            let axes_left = arr.shape.len() - axis;

            if axes_left == 0 {
                let i: usize = index
                    .iter()
                    .zip(arr.shape.iter())
                    .fold(0, |i, (y,s)| {
                        i * *s as usize + y
                    });
                //println!("{index:?}:{i}:{}", arr.data[i]);
                return Ok(format!("{}", arr.data[i]));
            }

            let next_hanging_indent = {
                let mut h = hanging_indent.clone();
                h.push(' ');
                h
            };

            let next_width = curr_width - ']'.len_utf8();
            let a_len = arr.shape[axis] as usize;
            //let show_summary = 2*edge_items < a_len;
            let show_summary = false;

            let (leading_items, trailing_items) = if show_summary {
                (edge_items, edge_items) 
            } else {
                (0, a_len)
            };

            let mut s = String::new();
            if axes_left == 1 {
                let elem_width = curr_width - separator.len().max(']'.len_utf8());  
                let mut line = hanging_indent.clone();
                for i in 0..leading_items {
                    index.push(i);
                    let word = recur(arr, f, index, next_hanging_indent.clone(), next_width)?;
                    _ = index.pop();
                    // _extendLine_pretty?
                    line.push_str(&word);
                    line.push_str(separator);
                }
                if show_summary {
                    // _extendLine?
                    line.push_str(separator);
                }
                
                /*
                for i in range_step(trailing_items as isize, 1, -1){
                    let i = i as usize;
                    index.push(a_len-i);
                    let word = recur(arr, f, index, next_hanging_indent.clone(), next_width)?;
                    _ = index.pop();
                    // _extendLine_pretty?
                    line.push_str(&word);
                    line.push_str(separator);
                }
                index.push(a_len-1);
                let word = recur(arr, f, index, next_hanging_indent.clone(), next_width)?;
                _ = index.pop();
                // _extendLine_pretty?
                line.push_str(&word);
                line.push_str(separator);
                */

                index.push(0);
                let last = a_len.saturating_sub(1);
                for i in a_len.saturating_sub(trailing_items)..last {
                    *index.last_mut().unwrap() = i;
                    // _extendLine_pretty?
                    let word = recur(arr, f, index, next_hanging_indent.clone(), next_width).unwrap();
                    extend_line(&mut s, &mut line, word, elem_width, hanging_indent.clone());
                    line.push_str(separator);
                }
                if a_len != 0 {
                    *index.last_mut().unwrap() = last;
                    let word = recur(arr, f, index, next_hanging_indent.clone(), next_width).unwrap();
                    extend_line(&mut s, &mut line, word, elem_width, hanging_indent.clone());
                }
                s.push_str(&line);
                index.pop();
            } else {
                let line_sep = format!("{}{}", separator, "\n".repeat(axes_left - 1));
                for i in 0..leading_items {
                    index.push(i);
                    let nested = recur(arr, f, index, next_hanging_indent.clone(), next_width)?;
                    index.pop();
                    let nested = format!("{hanging_indent}{nested}{line_sep}");
                    s.push_str(&nested);
                }
                if show_summary {
                    s.push_str(&hanging_indent);
                    s.push_str(summary_insert);
                    s.push_str(", \n");
                }

                /*
                if index.is_empty() {
                    let base = trailing_items;
                    println!("b: {base}, l: {a_len}");

                    println!("simiple");
                    for i in  (a_len - base)..a_len {
                        println!("{i}");
                    }
                    println!("range");
                    for i in range_step(base as isize, 1, -1){
                        let i = i as usize;
                        let i = a_len - i;
                        println!("{i}");
                    }
                    println!("{}", a_len - 1);
                }
                */

                index.push(0);
                let nested = ((a_len.saturating_sub(trailing_items))..a_len).map(|i| {
                    *index.last_mut().unwrap() = i;
                    let mut nested = recur(arr, f, index, next_hanging_indent.clone(), next_width).unwrap();
                    nested.insert_str(0, &hanging_indent);
                    nested
                }).join(&line_sep);
                index.pop();
                s.push_str(&nested);
            }

            Ok(format!("[{}]", s.split_off(hanging_indent.len())))
        }

        let grid = recur(self, f, &mut idx, String::from(" "), line_width)?;
        f.write_str(&grid)
    }
}

fn extend_line(mut s: &mut String, mut line: &mut String, word: String, line_width: usize, next_line_prefix: String) {
    let words = word.lines().collect_vec();
    if let [w] = words[..]  {
        if line.len() + w.len() > line_width {
            s.push_str(&line);
            s.push('\n');
            line.clone_from(&next_line_prefix)
        }
        line.push_str(&word);
    } else {
        let max_word_length = words.iter().map(|w| w.len()).max().unwrap_or(0);
        println!("l: {}, mx: {}, w: {}, pre: {}", line.len(), max_word_length, line_width, next_line_prefix.len());
        //println!("w: {word}");
        let indent = if line.len() + max_word_length > line_width 
        && line.len() > next_line_prefix.len() {
            s.push_str(&line);
            s.push('\n');

            *line = next_line_prefix.clone();
            line.push_str(words[0]);
             next_line_prefix
        } else {
            let ind = " ".repeat(line.len());
            line.push_str(words[0]);
            ind
        };

        for word in &words[1..] {
            s.push_str(&line);
            s.push('\n');
            *line = indent.clone();
            line.push_str(word);
        }
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
impl_from_arr!(f64, i64, u8, u16, u32, Val);
