

use itertools::Itertools;
use std::ops::*;

use crate::{Array, Val};
pub trait Til {
    fn til_mon(y: Val) -> Val;
    fn til_dyd(x: Val, y: Val) -> Val;
}



impl Til for Val {
    fn til_dyd(x: Val, y: Val) -> Val {
        panic!("nyi");
    }

    fn til_mon(y: Val) -> Val {
        use Val::*;
        match y {
            Int(y) => til(vec![y as u32], 0..y),
            IntArr(Array { data, shape }) =>  til(
                data.iter().map(|i| *i as u32).collect_vec(),
                0..data.into_iter().product()
            ),
            _ => panic!("cannot til {y:?}")
            
        }
    }
}

fn til(shape: Vec<u32>, r: Range<i64>) -> Val {
    Array {         
        data: r.collect_vec(),
        shape, 
    }.into()
}

macro_rules! impl_op {
    ( $($name:ident-$fn:ident);+) => {
        $(impl $name for Val  {
    type Output = Val;
    fn $fn(self, rhs: Self) -> Self::Output {
        use Val::*;
        return match (self, rhs) {
             (Int(x), Int(y)) => Int($name::$fn(x, y)),
             (Int(x), Float(y)) => Float((x as f64).$fn(y)),
             (Float(x), Int(y)) => Float(x.$fn(y as f64)),
             (Float(x), Float(y)) => Float(x.$fn(y)),
             (IntArr(x), IntArr(y)) => IntArr(x.$fn(y)),
            (FloatArr(x), FloatArr(y)) => FloatArr(x.$fn(y)),

            (Int(x), IntArr(y)) => IntArr($name::$fn(x, y)),
            (IntArr(x), Int(y)) => IntArr(x.$fn(y)),

            (Int(x), FloatArr(y)) => FloatArr(Array::<f64>::from(x).$fn(y)),
            (FloatArr(x), Int(y)) => FloatArr(x.$fn(Array::<f64>::from(y))),

        (Float(x), IntArr(y)) => FloatArr(Array::<f64>::from(x).$fn(Array::<f64>::from(y))),
        (IntArr(x), Float(y)) => FloatArr(Array::<f64>::from(x).$fn(y)),

        (IntArr(x), FloatArr(y)) => FloatArr(Array::<f64>::from(x).$fn(y)),
        (FloatArr(x), IntArr(y)) => FloatArr(x.$fn(Array::<f64>::from(y))),
        (x, y) => unreachable!("nyi: {:?} op {:?}", x, y),
        };
        }
        })+
    };
}

impl_op!(
    Add-add;
    Sub-sub;
    Mul-mul;
    //Div-div;
    Max-max;
    Min-min;
    GtrEq-ge;
    Gtr-gt;
    Less-lt;
    LessEq-le
);

impl Div for Val  {
    type Output = Val;
    fn div(self, rhs: Self) -> Self::Output {
        use Val::*;
        println!("{:?} div {:?}", self, rhs);
        let floatify = |x| match x {
            Int(x) => Float(x as f64),
            IntArr(x) => FloatArr(Array::<f64>::from(x)),
            FloatArr(_) | Float(_) => x,
            x => unreachable!("nyi: {:?} div y", x),
        };

        match (floatify(self), floatify(rhs)) {
            (Float(x), Float(y)) => Float(x / y),
            (FloatArr(x), Float(y)) => FloatArr(x / y),
            (Float(x), FloatArr(y)) => FloatArr(x / y),
            (FloatArr(x), FloatArr(y)) => FloatArr(x / y),
            _ => unreachable!("no"),
        }
    }
}

pub trait Max<Rhs = Self> {
    type Output;
    fn max(self, rhs: Rhs) -> Self::Output;
}

pub trait Min<Rhs = Self> {
    type Output;
    fn min(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_base_ops {
    ($($name:ident-$fn:ident);+) => {
$(

pub trait $name<Rhs = Self> {
    type Output;
    fn $fn(self, rhs: Rhs) -> Self::Output;
}

 impl <T: PartialOrd + From<u8>> $name<T> for T {
    type Output = Self;
    fn $fn(self, rhs: T) -> Self::Output {
        if PartialOrd::$fn(&self, &rhs) {
            T::from(1)
        } else {
            T::from(0)
        }
    }
})+

    };
}
impl_base_ops!(
    GtrEq-ge;
    Gtr-gt;
    Less-lt;
    LessEq-le
);


macro_rules! make_arr_ops  {
    ($($name:ident-$fn:ident);+) => {
        pub trait ArrayOps<T>:
        $($name<Array<T>, Output = Array<T>> +)+
        where T:
        $($name<Array<T>, Output = Array<T>> +)+
        {
            $(fn $fn(x: T, y: Array<T>) -> Array<T> { T::$fn(x, y) })+
        }

    };
}
make_arr_ops! {
    Add-add;
    Sub-sub;
    Mul-mul;
    Div-div;
    Min-min;
    Max-max;
    GtrEq-ge;
    Gtr-gt;
    Less-lt;
    LessEq-le
}

macro_rules! impl_arr_ops  {
    ($($tp:ty),+) => {
        $(impl ArrayOps<$tp> for $tp {})+
    };
}
impl_arr_ops!(i64, f64);

macro_rules! impl_cust_ops {
    (($($name:ident-$fn:ident);+) $tps:tt) => {
        $(
        impl_cust_ops!(@call $name $fn $tps);
        impl $name for f64 {
        type Output = Self;
        fn $fn(self, rhs: Self) -> Self::Output { self.$fn(rhs) }
        }
        )+
    };
    (@call $name:ident $fn:ident ($($tp:ty),+)) => {
        $(impl $name for $tp {
        type Output = Self;
        fn $fn(self, y: Self) -> Self { std::cmp::Ord::$fn(self, y) }
        })+
    };
}

impl_cust_ops!(
    (Max-max; Min-min)
    (i64)
);

macro_rules! impl_arr_prim_op {
    ( $name:ident, $fn:ident, $($tp:ty),+) => {
        $(
        impl $name<Array<$tp>> for $tp {
            type Output = Array<$tp>;
            fn $fn(self, Array { data, shape }: Array<$tp>) -> Self::Output {
                Array {
                    data: data
                        .into_iter()
                        .map(|x| <$tp as $name>::$fn(self, x))
                        .collect_vec(),
                    shape,
                }
            }
        })+
    };
}

macro_rules! impl_arr_op {
    ( $($name:ident-$fn:ident);+) => {
        $( impl<T: $name<T, Output = T> + std::fmt::Debug + ArrayOps<T>> $name<Array<T>> for Array<T>
        where
            for<'a> Array<T>: From<T> + From<&'a [T]>,
        {
            type Output = Self;
            fn $fn(self, rhs: Self) -> Self::Output {
                let mut y = rhs;
                let x = self;
                let opt = x
                    .shape
                    .iter()
                    .zip(&y.shape)
                    .find_position(|(x, y)| *x != *y);
                if let Some((rank, _)) = opt {
                    //let len = x.shape[0..rank].iter().fold(1, |a, x| a * x);
                    let len_a = x.shape[rank..].iter().fold(1, |a, x| a * x);
                    let len_b = y.shape[rank..].iter().fold(1, |a, x| a * x);
                    if len_a == 1 {
                        y.data = x
                            .data
                            .into_iter()
                            .zip(y.data.chunks_exact(len_b as usize))
                            .flat_map(|(x, yc)| <T as ArrayOps<T>>::$fn(x, Array::<T>::from(yc)))
                            .collect_vec();
                    } else if len_b == 1 {
                        y.data = x
                            .data
                            .chunks_exact(len_a as usize)
                            .into_iter()
                            .zip(y.data.into_iter())
                            .flat_map(|(xc, yv)| Array::<T>::$fn(xc.into(), yv.into()))
                            .collect();
                        y.shape = x.shape
                    } else {
                        unreachable!("eql len");
                    }
                } else {
                    y.data = x
                        .data
                        .into_iter()
                        .zip(y.data.into_iter())
                        .map(|(x, y)| <T as $name>::$fn(x, y))
                        .collect_vec();
                }
                return y;
            }
        }

        impl_arr_prim_op!($name, $fn, i64, f64);

        impl<T: $name<Output = T> + Copy> $name<T> for Array<T> {
            type Output = Self;
            fn $fn(self, y: T) -> Self::Output {
                Array {
                    data: self.data.into_iter().map(|x| T::$fn(x, y)).collect_vec(),
                    shape: self.shape,
                }
            }
        })+
    };
}

impl_arr_op!(
    Add-add;
    Sub-sub;
    Mul-mul;
    Div-div;
    Max-max;
    Min-min;
    GtrEq-ge;
    Gtr-gt;
    Less-lt;
    LessEq-le
);
