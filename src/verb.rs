#![allow(non_camel_case_types)]

use std::cell::OnceCell;
use std::fmt::{Display, Pointer, Write};
use std::sync::OnceLock;
use std::{collections::HashMap, hash::Hash};
use std::collections::hash_map;
use colored::Colorize;

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

impl Display for Verb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Verb::*;
        match self {
            Prim(v) => f.write_str(&format!("{v}").green().to_string()),
            Adv{u, p} => f.write_str(&format!("{u}{}", format!("{p}").cyan())),
            Conj{u, p , v} => f.write_str(&format!("{u}{}{v}", format!("{p}").magenta())),
            Comp { u, v } => f.write_fmt(format_args!("({u} {v})")),
            Fork { f:ff, g, h } => f.write_fmt(format_args!("({ff} {g} {h})")),
            Id(v) => f.write_str(&format!("{v}").blue()),
        }
    }
}

impl Verb {
    pub fn identity(&self, y: Val) -> Option<Val> {
        use PrimVerb::*;
        if let Verb::Prim(v) = self {
            match v {
                plus => Some(Val::Int(0)),
                dash => Some(Val::Int(0)),
                star => Some(Val::Int(1)),
                pcnt  => Some(Val::Float(1.0)),
                _ => None,
            }
        } else {
            None
        }
    }
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


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimVerb {
    i_dot,
    i_col,
    h_dot,
    h_col,
    H_dot,
    H_col,
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
    semi,
    semi_dot,
    semi_col,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimAdv {
    slsh,
    bslsh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Display for PrimVerb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = SPELL_IN_OUT.get().unwrap().spell_out(Part::Verb(*self)).unwrap();
        f.write_fmt(format_args!("{}{}{}", c.ch as char, c.infl, c.infl2))
    }
}

impl Display for PrimAdv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = SPELL_IN_OUT.get().unwrap().spell_out(Part::Adv(*self)).unwrap();
        f.write_fmt(format_args!("{}{}{}", c.ch as char, c.infl, c.infl2))
    }
}

impl Display for PrimConj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = SPELL_IN_OUT.get().unwrap().spell_out(Part::Conj(*self)).unwrap();
        f.write_fmt(format_args!("{}{}{}", c.ch as char, c.infl, c.infl2))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PseudoChar {
    pub ch: u8,
    pub part: Part,
    pub infl: Inflection,
    pub infl2: Inflection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Inflection {
    None,
    Dot,
    Col,
}

impl Display for Inflection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dot => f.write_char('.'),
            Self::Col => f.write_char(':'),
            Self::None => Ok(()),
        }
    }
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
            SPELL_IN_OUT.get().unwrap().spell_in[c as usize][infl as usize].ok_or(())
        } else {
            unreachable!("{:?}", w);
        }
        
    }
}


#[derive(Debug)]
pub struct SpellInOut {
    spell_in: [[Option<PseudoChar>;3];128-0x20],
    verb: HashMap<PrimVerb, PseudoChar>,
    adv: HashMap<PrimAdv, PseudoChar>,
    conj: HashMap<PrimConj, PseudoChar>,
}

impl SpellInOut {
    fn link(&mut self, p: Part, c: PseudoChar) {
        use Part::*;
        match p {
             Verb(v) => _ = self.verb.insert(v, c),
             Adv(a) => _ = self.adv.insert(a, c),
            Conj(cj) => _ = self.conj.insert(cj, c),
            Null | Rpar | Lpar | Asgn => (),
        };
    }

    fn add(&mut self, ch: u8, infl: Inflection, part: Part) -> PseudoChar {
        let c = PseudoChar {ch, part, infl, infl2: Inflection::None};
        self.link(part, c);
        c
    }

    pub fn spell_out(&self, part: Part) -> Option<PseudoChar> {
        use Part::*;
        match part {
            Adv(v) => self.adv.get(&v).copied(),
            Verb(v) => self.verb.get(&v).copied(),
            Conj(v) => self.conj.get(&v).copied(),
            _ => None,
        }
    }

    fn setup(&mut self,  ch: u8, p: [u8;3], (p1, p2, p3): (Part, Part, Part)) {
        if p[0] > 0 {
            self.spell_in[(ch - 0x20) as usize][0] = Some(self.add(ch, Inflection::None, p1));
        }
        if p[1] > 0 {
            self.spell_in[(ch - 0x20) as usize][1] = Some(self.add(ch, Inflection::Dot, p2));
        }
        if p[2] > 0 {
            self.spell_in[(ch - 0x20) as usize][2] = Some(self.add(ch, Inflection::Col, p3));
        }
    }

    pub fn init() -> Self {
        let mut a = SpellInOut {
            spell_in: [[None;3];(128-0x20)],
            verb: HashMap::new(),
            adv: HashMap::new(),
            conj: HashMap::new(),
        };

        use Part::*;
        use PrimVerb::*;
        use PrimAdv::*;
        use PrimConj::*;
        a.setup(b'+', [1, 0, 0], (Verb(plus), Null, Null));
        a.setup( b'-', [1, 0, 0], (Verb(dash), Null, Null));
        a.setup( b'*', [1, 0, 0], (Verb(star), Null, Null));
        a.setup( b'%', [1, 0, 0], (Verb(pcnt), Null, Null));
        a.setup( b'!', [1, 0, 0], (Verb(excl), Null, Null));
        a.setup( b'$', [1, 0, 0], (Verb(dllr), Null, Null));
        a.setup( b'@', [1, 0, 0], (Conj(at), Null, Null));
        a.setup( b'#', [1, 0, 1], (Verb(hash), Null, Verb(hash_col)));
        a.setup( b'<', [1, 0, 0], (Verb(larr), Null, Null));
        a.setup( b'>', [1, 0, 0], (Verb(rarr), Null, Null));
        a.setup( b'{', [1, 1, 0], (Verb(lcrl), Verb(lcrl_dot), Verb(lcrl_col)));
        a.setup( b'}', [1, 1, 0], (Verb(rcrl), Verb(rcrl_dot), Verb(rcrl_col)));
        a.setup( b'[', [1, 0, 0], (Verb(lbrak), Null, Null));
        a.setup( b']', [1, 0, 0], (Verb(rbrak), Null, Null));
        a.setup( b'i', [0, 1, 1], (Null, Verb(i_dot), Verb(i_col)));
        a.setup( b'h', [0, 1, 1], (Null, Verb(h_dot), Verb(h_col)));
        a.setup( b'H', [0, 1, 1], (Null, Verb(H_dot), Verb(H_col)));


        a.setup( b'&', [0, 1, 1], (Null, Conj(ampr_dot), Conj(ampr_col)));

        a.setup( b'/', [1, 0, 0], (Adv(slsh), Null, Null));
        a.setup( b'\\', [1, 0, 0], (Adv(bslsh), Null, Null));

        a.setup( b';', [1, 1, 1], (Verb(semi), Verb(semi_dot), Verb(semi_col)));
        a.setup( b'=', [1, 0, 1], (Verb(equal), Null, Asgn));
        a.setup( b'(', [1, 0, 0], (Lpar, Null, Null));
        a.setup( b')', [1, 0, 0], (Rpar, Null, Null));
        a
    }

}


pub static SPELL_IN_OUT: OnceLock<SpellInOut> = OnceLock::new();
