

use std::{any::TypeId, borrow::BorrowMut, collections::HashMap, f64, fmt::Debug, iter::{self, Once}, mem, ops::DerefMut, os::linux::fs::MetadataExt, str::FromStr};
use std::ops::*;
use num::Float;

use crate::value::*;
use crate::ops::*;
use crate::verb::*;

use crate::{Adverb, Array, Func, Part, Conj, PrimConj, PseudoChar, SpellIn, Val, Verb, PrimVerb};
use super::ALError;


#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Verb(Verb),
    Ident(String),
    Noun(Val),
    Adv(Adverb),
    Conj(Conj),
    Mark,
    Lpar,
    Rpar,
    Asgn,
}

impl Token {
    fn edge(&self) -> bool {
        use Token::*;
        matches!(self, Lpar | Mark | Asgn)
    }

    fn avn(&self) -> bool {
        use Token::*;
        matches!(self, Adv(_) | Verb(_)  | Noun(_))
    }

    fn cavn(&self) -> bool {
        use Token::*;
        matches!(self, Conj(_) | Adv(_) | Verb(_)  | Noun(_))
    }
}


pub fn eval(mut words: Vec<&str>, mut env: &mut HashMap<String, Val>) -> Result<(), super::ALError>{
    let mut toks: Vec<Token> = Vec::with_capacity(words.len());
    if words.is_empty() {return Ok(());}

    while !words.is_empty() || toks.len() > 1 {
        println!("toks {toks:?}");
        use Token::*;

        toks = match toks.as_mut_slice() {
            [ e, Verb(v), Noun(y), rest@.. ] if e.edge() => {
                println!("mon");
                let mut rest = rest.to_vec();
                let v = mem::replace(v, crate::Verb::Prim(PrimVerb::plus));
                let y = mem::replace(y, Val::Int(-1));
                let r = eval_mon(v, y);
                rest.insert(0, Noun(r));
                rest.insert(0, mem::replace(e, Token::Mark));

                rest
            },
            [ e, u@Verb(_), Verb(v), Noun(y), rest@.. ] if e.edge() || e.avn() => {
                println!("mon");
                let mut rest = rest.to_vec();
                let v = mem::replace(v, crate::Verb::Prim(PrimVerb::plus));
                let y = mem::replace(y, Val::Int(-1));
                let r = eval_mon(v, y);
                rest.insert(0, Noun(r));
                rest.insert(0, mem::replace(u, Token::Mark));
                rest.insert(0, mem::replace(e, Token::Mark));

                rest
            },
            [e, Noun(x), Verb(v), Noun(y), rest@..]if e.edge() || e.avn()  => {
                println!("dyd");
                let mut rest = rest.to_vec();
                let x = mem::replace(x, Val::Int(-1));
                let v = mem::replace(v, crate::Verb::Prim(PrimVerb::plus));
                let y = mem::replace(y, Val::Int(-1));
                let r = eval_dyd(v, x, y);
                rest.insert(0, Noun(r));
                rest.insert(0, mem::replace(e, Token::Mark));


                rest
            },
            //[ e, u@(Verb(_) | Noun(_)), Adv(a), rest@.. ] if e.edge() || e.avn() => {
            [ e, Verb(u) , Adv(a), rest@.. ] if e.edge() || e.avn() => {
                println!("adv");
                let mut rest = rest.to_vec();
                let a = mem::replace(a, crate::Adverb::Prim(PrimAdv::slsh));
                
                //rest.insert(0, Verb(crate::Verb::Adv { u: Box::new(u.clone()), src: a }));
                rest.insert(0, Verb(a.cons(u.clone())));
                rest.insert(0, mem::replace(e, Token::Mark));
                rest
            },
            [ e, u@(Verb(_) | Noun(_)), Conj(c), v@(Verb(_) | Noun(_)), rest@.. ] if e.edge() || e.avn() => {
                println!("conj");
                let mut rest = rest.to_vec();

                let u = match u {
                    Noun(n) => crate::Verb::Id(Box::new(n.clone())),
                    Verb(x) => x.clone(),
                    _ => unreachable!(),
                };

                let v = match v {
                    Noun(n) => crate::Verb::Id(Box::new(n.clone())),
                    Verb(x) => x.clone(),
                    _ => unreachable!(),
                };

                rest.insert(0, Verb(c.clone().cons2(u, v)));
                rest.insert(0, mem::replace(e, Token::Mark));
                rest
            },
            [e, Verb(f), Verb(g), Verb(h), rest@..] if e.edge() || e.avn() => {
                println!("fork");
                let mut rest = rest.to_vec();
                
                rest.insert(0, Verb(crate::Verb::Fork { 
                    f: Box::new(f.clone()), 
                    g: Box::new(g.clone()), 
                    h: Box::new(h.clone())
                }));

                rest.insert(0, mem::replace(e, Token::Mark));
                println!("rest: {rest:?}");
                println!("w: {words:?}");
                rest
            },
            [e, f, g, h, rest@..] if e.edge() && f.cavn() && g.cavn() && h.cavn() => {
                println!("train");
                let mut rest = rest.to_vec();

                rest.insert(0, eval_train(f.clone(), g.clone(), h.clone()));
                rest.insert(0, mem::replace(e, Token::Mark));
                rest

            },

            [Ident(s), Asgn, y, rest@..] if y.cavn()  => {
                println!("asgn");
                let rest = rest.to_vec();
                let y = mem::replace(y, Token::Mark);
                let y = match y {
                     Conj(y) => Val::ValFunc(Func::C(y)),
                     Verb(y) => Val::ValFunc(Func::V(y)),
                     Adv(y) => Val::ValFunc(Func::A(y)),
                     Noun(y) => y,
                    y => panic!("not cavn: {y:?}"),
                };
                let s = mem::replace(s, String::new());
                env.insert( s, y);
                rest
            },
            [Lpar, v, Rpar, any@..] => {
                println!("punc");
                let mut restv = any.to_vec();
                //restv.rotate_left(mid)
                restv.insert(0, mem::replace(v, Token::Mark));
                println!("restv: {restv:?}");
                restv
            },
            //m => unreachable!("bad match: {m:?}"),
            //}
            //toks.splice(lo..hi, iter::once(y));
            //restv
            //},
            [y] | [Mark, y] if words.is_empty() => {
                println!("result: {y:?}");
                break;
            },
            rest if !words.is_empty() => {
                println!("move");
                let mut t = rest.to_vec();
                if let Some(Asgn) = rest.first() {
                    t.insert(0, move_words(&mut words, env, true)?);
                } else {
                    t.insert(0, move_words(&mut words, env, false)?);
                }
                t
            },
            [m, rest@..] if words.is_empty() && *m != Token::Mark => {
                println!("mark");
                let mut rest = rest.to_vec();
                rest.insert(0, mem::replace(m, Token::Mark));
                rest.insert(0, Token::Mark);
                rest

            },
            _ => {
                println!("unexptected state: {toks:?} {words:?}") ;
                return  Err(ALError::Syntax);
            },
        };
    }
    Ok(())
}

/*
EDGE,      VERB,      NOUN, ANY,       monad,   ..., 1,2, ...,
EDGE+AVN,  VERB,      VERB, NOUN,      monad,   ..., 2,3, ...,
EDGE+AVN,  NOUN,      VERB, NOUN,      dyad,    ..., 1,3, ...,
EDGE+AVN,  VERB+NOUN, ADV,  ANY,       adv,     ..., 1,2, ...,
EDGE+AVN,  VERB+NOUN, CONJ, VERB+NOUN, conj,    ..., 1,3, ...,
EDGE+AVN,  VERB,      VERB, VERB,      trident, ..., 1,3, ...,
EDGE,      CAVN,      CAVN, CAVN,      trident, ..., 1,3, ...,
EDGE,      CAVN,      CAVN, ANY,       bident,  ..., 1,2, ...,
NAME+NOUN, ASGN,      CAVN, ANY,       is,      ..., 0,2, ...,
LPAR,      CAVN,      RPAR, ANY,       punc,    ..., 0,2, ...,
*/

//fn eval_match(a: &mut Token, b: &mut Token, c: &mut Token, d: &mut Token) -> (usize, usize) { }

fn move_words(words: &mut Vec<&str>, env: &mut HashMap<String, Val>, asgn: bool) -> Result<Token, ALError> {
    let w = words.pop().unwrap();
    let wb = w.as_bytes();

    if let Ok(pc) = PseudoChar::try_from(w) {
         Ok(match pc.part {
            Part::Verb(v) => Token::Verb(Verb::Prim(v)),
            Part::Adv(a) => Token::Adv(Adverb::Prim(a)),
            Part::Conj(c) => Token::Conj(Conj::Prim(c)),
            Part::Lpar => Token::Lpar,
            Part::Rpar => Token::Rpar,
            Part::Asgn => Token::Asgn,
            _ => unreachable!("unhanded char: {pc:?}"),
        })
    } else if asgn {
        Ok(Token::Ident(match wb[0] {
            b'a'..=b'z' | b'A'..=b'Z' =>  String::from_utf8(wb.to_vec()).expect("not utf8"),
            b'\'' => String::from_utf8(wb[1..wb.len()-1].to_vec()).expect("not utf8"),
            b'_' | b'0'..=b'9' => panic!("not a name"),
            _ => panic!("unhandled: {w:?}")
        }))
    } else {
        match wb[0] {
            b'a'..=b'z' | b'A'..=b'Z' =>  {
                let s = String::from_utf8(wb.to_vec()).expect("not utf8");
                if let Some(y) = env.get(&s) {
                    Ok(match y.clone() {
                        Val::ValFunc(y) => match y {
                            Func::A(y) => Token::Adv(y),
                            Func::C(y) => Token::Conj(y),
                            Func::V(y) => Token::Verb(y),
                        }
                        y  => Token::Noun(y), 
                    } )
                } else {
                    Err(ALError::Value(s))
                }

            },
            b'\'' => Ok(Token::Noun(Val::AsciiArr(Array {
                data: wb[1..wb.len()-1].to_vec(),
                shape: vec![(wb.len() - 2) as u32],
            }))),
            b'_' | b'0'..=b'9' => Ok(Token::Noun(parse_nums(w, words))),
            _ => panic!("unhandled: {w:?}")
        }
    }
}

fn parse_nums(w: &str, words: &mut Vec<&str>) -> Val {
    let mut count = 0;
    let mut floats = w.contains('.');
    for i in (0..words.len()).rev() {
        let s = words[i];
        if matches!(s.as_bytes()[0], b'_' | b'0'..=b'9') {
            count += 1;
            floats |= s.contains('.');
        } else {
            break;
        }
    }

    if count > 0 {
        let mut nums = words.split_off(count);
        nums.push(w);
        let nums: Vec<String> = nums.into_iter()
            .map(str::to_string)
            .map(|mut s| unsafe {
                let n = s.as_bytes_mut() ;
                if n[0] == b'_' {
                    n[0] = b'-';
                }
                s
            })
            .collect();

        if floats {
            nums.into_iter()
                .map(|n| n.parse::<f64>())
                .collect::<Result<Array<_>, _>>()
                .unwrap()
                .into()
        } else {
            nums.into_iter()
                .map(|n| n.parse::<i64>())
                .collect::<Result<Array<_>, _>>()
                .unwrap()
                .into()
        }
    } else if floats {
        w.parse::<f64>().unwrap().into() 
    } else {
        w.parse::<i64>().unwrap().into() 
    }

}


fn eval_train(f: Token, g: Token, h: Token) -> Token { 
    use Token::*;
    let adv = |t| Adv(Adverb::Train(Box::new(t)));
    let conj = |t| Conj(crate::Conj::Train(Box::new(t)));

    let func = |(f, g, h): (Token, Token, Token)| -> (Func, Func, Func) {
        use Token::*;
        use Func::*;
        let make_func = |t| match t {
            Verb(v) => V(v),
            Adv(v) => A(v),
            Conj(v) => C(v),
            Noun(n) => V(crate::Verb::Id(Box::new(n))),
            _ => panic!("not a func {t:?}"),
        };

        (make_func(f) , make_func(g), make_func(h))
    };

    let tr = (f, g, h);
    match &tr {
        //(Verb(_), Noun(g), Conj(_)) => adv(func(tr)),
        (Adv(_), Verb(_), Verb(_)) => adv(func(tr)),
        (Adv(_), Adv(_), Adv(_)) => adv(func(tr)),
        (Noun(_), Conj(_), Adv(_)) => adv(func(tr)), 
        (Verb(_), Conj(_), Adv(_)) => adv(func(tr)),
        (Adv(_), Conj(_), Noun(_)) => adv(func(tr)),
        (Adv(_), Conj(_), Verb(_)) => adv(func(tr)),

        (Verb(_), Verb(_), Conj(_)) => conj(func(tr)),
        (Noun(_), Verb(_), Conj(_)) => conj(func(tr)),
        (Conj(_), Verb(_), Verb(_)) => conj(func(tr)), 
        (Conj(_), Verb(_), Conj(_)) => conj(func(tr)),
        (Adv(_), Adv(_), Verb(_)) => conj(func(tr)),
        (Conj(_), Adv(_), Adv(_)) => conj(func(tr)),
        (Noun(_), Conj(_), Conj(_)) => conj(func(tr)),
        (Verb(_), Conj(_), Conj(_)) => conj(func(tr)),
        (Adv(_), Conj(_), Adv(_)) => conj(func(tr)),
        (Adv(_), Conj(_), Conj(_)) => conj(func(tr)),
        (Conj(_), Conj(_), Noun(_)) => conj(func(tr)),
        (Conj(_), Conj(_), Verb(_)) => conj(func(tr)),
        (Conj(_), Conj(_), Adv(_)) => conj(func(tr)),
        (Conj(_), Conj(_), Conj(_)) => conj(func(tr)),
        (f, g, h) => panic!("unhandled train: ({f:?} {g:?} {h:?})")
    }
}
