use itertools::Itertools;



#[derive(Debug, Clone, Copy)]
enum State {
    SS, // Space
    SX, // Other
    SA, // Alphanumeric
    ST, // `
    SN, // N
    SNB, // NB
    SNZ, //
    S9,  // 0-9 _
    SQ,  // Quote
    SQQ, // even Quote
    SZ   // comment
}

#[derive(Debug, Clone, Copy)]
enum CharClass {
    ///Space
   CS,
    /// N
   CN,
    /// B
   CB,
    /// .
   CD,
    /// :
   CC,
    /// '
   CQ,
    /// `
   CT,
    /// 0-9 _
   C9,
    /// Other
   CX,
    /// Letters not N or B 
   CA, 
}

impl From<char> for CharClass {
    fn from(c: char) ->  Self {
        use CharClass::*;
        match c {
            ' ' => CS,
            '.' => CD,
            '`' => CT,
            ':' => CC,
            '\'' => CQ,
            'N' => CN,
            'B' => CB,
            '0'..='9' | '_' => C9,
            'a'..='z' | 'A'..='Z' => CA,
            _ => CX,
        }
    }
}

pub fn lex(mut s: &str) -> Vec<&str> {
    use CharClass::*;
    use State::*;

    let mut curs = SS;
    let mut j = 0;
    let mut out = Vec::new();

    s = s.trim_end();

    fn id(cc: CharClass) -> State{
        match cc {
            CX => SX,
            CS => SS,
            CA => SA,
            CN => SN,
            // CB => SB,
            C9 => S9,
            CQ => SQ,
            CT => SA,
            _ => panic!()
        }
    }

    for (i, c) in s.chars().enumerate(){
        let cc: CharClass = c.into();
        // X   S   A   N   B   9   D   C   Q
        curs = match curs {
            // XN  S   AN  NN  AN  9N  XN  XN  QN     S  Space
            SS => match cc {
                CS => SS,
                CB => {j = i; SA},
                CD => {j = i; SX},
                CC => {j = i; SX},
                CX | CA | CN | C9 | CQ | CT => {j = i; id(cc)}
            },
            // XI  SI  AI  NI  AI  9I  X   X   QI     X  Other
            SX => match cc {
                CB => {out.push(&s[j..i]); j = i; SA},
                CD => SX,
                CC => SX,
                CX | CS | CA | CN | C9 | CQ | CT => {out.push(&s[j..i]); j = i; id(cc)}
            },
            // XI  SI  A   A   A   A   X   X   QI     A  Alphanumeric
            SA => match cc {
                CA | CN | CB | C9 => SA,
                CD | CC => SX,
                CX | CS | CQ | CT => {out.push(&s[j..i]); j = i; id(cc)}
            },
            // XI  SI  A   A   NB  A   X   X   QI     N  N
            SN => match cc {
                CA | CN | C9 => SA,
                CD | CC => SX,
                CB => SNB,
                CX | CS | CQ | CT => {out.push(&s[j..i]); j = i; id(cc)}
            },
            // XI  SI  A   A   A   A   NZ  X   QI     NB NB
            SNB => match cc {
                CA | CN | CB | C9 => SA,
                CC => SX,
                CD => SNZ,
                CX | CS | CQ | CT => {out.push(&s[j..i]); j = i; id(cc)}
            },
            // Z   Z   Z   Z   Z   Z   X   X   Z      NZ NB.
            SNZ => match cc {
                CD | CC => SX,
                CX | CS | CA | CN | CB | C9 | CQ | CT => SZ
            },
            // XI  SI  9   9   9   9   9   X   QI     9  Numeric
            S9 => match cc {
                CC => SX,
                CA | CN | CB | C9 | CD => S9,
                CX | CS | CQ | CT => {out.push(&s[j..i]); j = i; id(cc)}
            },
            // Q   Q   Q   Q   Q   Q   Q   Q   QQ     Q  Quote
            SQ => match cc {
                CQ => SQQ,
                _ => SQ
            },
            // XI  SI  AI  NI  AI  9I  XI  XI  Q      QQ Even Quotes
            SQQ => match cc {
                CQ => SQ,
                CC | CD => {out.push(&s[j..i]); j = i; SX}
                CB => {out.push(&s[j..i]); j = i; SA}
                CX | CS | CA | CN | C9 | CT => {out.push(&s[j..i]); j = i; id(cc)}
            },
            // Z   Z   Z   Z   Z   Z   Z   Z   Z      Z  Trailing Comment
            SZ => SZ,
            ST => match cc {
                CS => SS,
                CB | CA => {j = i; SA},
                CD => {j = i; SX},
                CC => {j = i; SX},
                CX | CN | C9 | CQ | CT => {j = i; id(cc)}
            },
        }
    }
    out.push(&s[j..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex() {
        let r = lex("(+/1 2)  NB. hello   ");
        assert_eq!(r, vec!["(", "+", "/", "1", "2", ")", "NB. hello"]);
        let r = lex("' abc 12 +-5::.()'  +:. /. 1.333 _1.2");
        assert_eq!(r, vec!["' abc 12 +-5::.()'", "+:.", "/.", "1.333", "_1.2"]);

    }
}

pub trait SubsliceOffset {
    /**
    Returns the byte offset of an inner slice relative to an enclosing outer slice.
    Examples
    ```ignore
    let string = "a\nb\nc";
    let lines: Vec<&str> = string.lines().collect();
    assert!(string.subslice_offset_stable(lines[0]) == Some(0)); // &"a"
    assert!(string.subslice_offset_stable(lines[1]) == Some(2)); // &"b"
    assert!(string.subslice_offset_stable(lines[2]) == Some(4)); // &"c"
    assert!(string.subslice_offset_stable("other!") == None);
    ```
    */
    fn subslice_offset_stable(&self, inner: &Self) -> Option<usize>;
}

impl SubsliceOffset for str {
    fn subslice_offset_stable(&self, inner: &str) -> Option<usize> {
        let self_beg = self.as_ptr() as usize;
        let inner = inner.as_ptr() as usize;
		 //(self_beg..self_beg.wrapping_add(self.len())).contains(&inner);
        if inner < self_beg || self_beg.wrapping_add(self.len()) < inner  {
            None
        } else {
            Some(inner.wrapping_sub(self_beg))
        }
    }

}
