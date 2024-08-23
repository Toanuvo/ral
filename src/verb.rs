#![allow(non_camel_case_types)]

use crate::{eval::{Token}, Func, Val};


#[derive(Debug, Clone, PartialEq)]
pub enum Verb {
    Prim(PrimVerb),
    Adv { u: Box<Verb>, p: PrimAdv },
    Conj { u: Box<Verb>, p: PrimConj, v: Box<Verb>},
    Comp { u: Box<Verb>, v: Box<Verb>},
    Fork { f: Box<Verb>, g: Box<Verb>, h: Box<Verb>},
    Id(Box<Val>),
}

impl Verb {
    pub fn fork(f: Verb, g: Verb, h: Verb) -> Self{
        Self::Fork {
            f: Box::new(f),
            g: Box::new(g),
            h: Box::new(h)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Adverb {
    Prim(PrimAdv),
    Train(Box<(Func, Func, Func)>),
    Conj {
        left: bool,
        u: Box<Verb>,
        src: Box<Conj>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Conj {
    Prim(PrimConj),
    Train(Box<(Func, Func, Func)>),
}

impl Adverb {
    pub fn cons(self, v: Verb) -> Verb {
        match self {
            Self::Prim(p) => Verb::Adv { u: Box::new(v), p },
            Self::Train(t) => {
                use Func::*;
                match *t {
                    (A(f), V(g), V(h)) => crate::Verb::fork(f.cons(v), g, h),
                    (A(f), A(g), A(h)) => h.cons(g.cons(f.cons(v))),
                    //(N(f), C(g), A(h)) => g.cons2(crate::Verb::Id(Box::new(f)), h.cons(v)),
                    (V(f), C(g), A(h)) => g.cons2(f, h.cons(v)),
                    //(A(f), C(g), N(h)) => g.cons2(f.cons(v), crate::Verb::Id(Box::new(f))),
                    (A(f), C(g), V(h)) => g.cons2(f.cons(v), h),
                    (f, g, h) => panic!("unexpected train: {f:?} {g:?} {h:?}"),
                }
            }
            _ => panic!("nyi: {self:?}"),
        }
    }
}

impl Conj {
    pub fn cons1(self, u: Verb, left: bool) -> Adverb {
        Adverb::Conj { left, u: Box::new(u), src: Box::new(self)}
    }

    pub fn cons2(self, u: Verb, v: Verb) -> Verb {
        match self {
            Self::Prim(p) => Verb::Conj { u: Box::new(u), p, v: Box::new(v)},
            Self::Train(t) => {
                use Func::*;
                match *t {
                    (V(f), V(g), C(h)) => crate::Verb::fork(f, g, h.cons2(u, v)),
                    //(N(f), V(g), C(h)) => 
                    (C(f), V(g), V(h)) => crate::Verb::fork(f.cons2(u, v), g, h),
                    (C(f), V(g), C(h)) => crate::Verb::fork(f.cons2(u.clone(), v.clone()), g, h.cons2(u, v)),
                    (A(f), A(g), V(h)) => crate::Verb::fork(f.cons(u), g.cons(v), h),
                    (C(f), A(g), A(h)) => h.cons(g.cons(f.cons2(u, v))),
                    //(N(f), C(g), C(h)) => 
                    (V(v1), C(c1), C(c2)) => c1.cons2(v1, c2.cons2(u, v)),
                    (A(f), C(g), A(h)) => g.cons2(f.cons(u), h.cons(v)),
                    (A(f), C(g), C(h)) => g.cons2(f.cons(u.clone()), h.cons2(u, v)),
                    //(C(f), C(g), N(h)) =>
                    (C(f), C(g), V(h)) => g.cons2(f.cons2(u,v), h),
                    (C(f), C(g), A(h)) => g.cons2(f.cons2(u, v.clone()), h.cons(v)),
                    (C(f), C(g), C(h)) => g.cons2(f.cons2(u.clone(), v.clone()), h.cons2(u, v)),
                    _ => panic!("nyi"),
                }
            }
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimVerb {
    i_dot,
    i_col,
    hash,
    hash_col,
    lcrl,
    lcrl_dot,
    lcrl_col,
    rcrl,
    rcrl_dot,
    rcrl_col,
    lbrak,
    rbrak,
    dllr,
    excl,
    plus,
    star,
    dash,
    pcnt,
    larr,
    rarr,
    equal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimAdv {
    slsh,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimConj {
    at,
    ampr,
    ampr_dot,
    ampr_col,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Part {
    Null,
    Verb(PrimVerb),
    Adv(PrimAdv),
    Conj(PrimConj),
    Lpar,
    Rpar,
    Asgn,
    //Mark,
    //Noun,
    //Name,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PseudoChar {
    pub ch: u8,
    pub part: Part,
    pub infl: Inflection,
    pub infl2: Inflection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Inflection {
    None,
    Dot,
    Col,
}

impl Inflection {
    fn of(c: u8) -> Option<Self> {
         match c {
            b'.' => Some(Self::Dot),
            b':' => Some(Self::Col),
            _ => None,
        }
    }
}

impl TryFrom<&str> for PseudoChar {
    type Error = ();
    fn try_from(w: &str) -> Result<Self, Self::Error> {
        let w = w.as_bytes();
        if w.len() > 3 {
            return Err(());
        }
        let c = if 0x20 <= w[0] && w[0] < 0x80 {
            w[0] - 0x20
        } else {
            return Err(());
        };
        let infl = if w.len() > 1 { Inflection::of(w[1]) } else { Some(Inflection::None) };
        let infl2 = if w.len() > 2 { Inflection::of(w[2]) } else { Some(Inflection::None) };
        if infl.is_none() || infl2.is_none() { return Err(()); }
        let (infl, infl2) = (infl.unwrap(), infl2.unwrap());

        if w.len() < 3 {
            SpellIn[c as usize][infl as usize].ok_or(())
        } else {
            unreachable!("{:?}", w);
        }
        
    }
}

type SpellInArr = [[Option<PseudoChar>;3];128-0x20]; 

pub static SpellIn: SpellInArr = {
    let mut a = [[None;3];(128-0x20)];

    const fn setup(mut arr: SpellInArr, ch: u8, p: [u8;3], (p1, p2, p3): (Part, Part, Part)) -> SpellInArr {
        if p[0] > 0 {
            arr[(ch - 0x20) as usize][0] = Some(PseudoChar {ch, part: p1, infl: Inflection::None, infl2: Inflection::None})
        }
        if p[1] > 0 {
            arr[(ch - 0x20) as usize][1] = Some(PseudoChar {ch, part: p2, infl: Inflection::Dot, infl2: Inflection::None})
        }
        if p[2] > 0 {
            arr[(ch - 0x20) as usize][2] = Some(PseudoChar {ch, part: p3, infl: Inflection::Col, infl2: Inflection::None})
        }
        arr
    }

    use Part::*;
    use PrimVerb::*;
    use PrimAdv::*;
    use PrimConj::*;
    a = setup(a, b'+', [1, 0, 0], (Verb(plus), Null, Null));
    a = setup(a, b'-', [1, 0, 0], (Verb(dash), Null, Null));
    a = setup(a, b'*', [1, 0, 0], (Verb(star), Null, Null));
    a = setup(a, b'%', [1, 0, 0], (Verb(pcnt), Null, Null));
    a = setup(a, b'!', [1, 0, 0], (Verb(excl), Null, Null));
    a = setup(a, b'$', [1, 0, 0], (Verb(dllr), Null, Null));
    a = setup(a, b'#', [1, 0, 1], (Verb(hash), Null, Verb(hash_col)));
    a = setup(a, b'<', [1, 0, 0], (Verb(larr), Null, Null));
    a = setup(a, b'>', [1, 0, 0], (Verb(rarr), Null, Null));
    a = setup(a, b'{', [1, 1, 0], (Verb(lcrl), Verb(lcrl_dot), Verb(lcrl_col)));
    a = setup(a, b'}', [1, 1, 0], (Verb(rcrl), Verb(rcrl_dot), Verb(rcrl_col)));
    a = setup(a, b'[', [1, 0, 0], (Verb(lbrak), Null, Null));
    a = setup(a, b']', [1, 0, 0], (Verb(rbrak), Null, Null));
    a = setup(a, b'i', [1, 1, 1], (Null, Verb(i_dot), Verb(i_col)));

    a = setup(a, b'&', [0, 1, 1], (Null, Conj(ampr_dot), Conj(ampr_col)));

    a = setup(a, b'/', [1, 0, 0], (Adv(slsh), Null, Null));

    a = setup(a, b'=', [1, 0, 1], (Verb(equal), Null, Asgn));
    a = setup(a, b'(', [1, 0, 0], (Lpar, Null, Null));
    a = setup(a, b')', [1, 0, 0], (Rpar, Null, Null));

    a
};
