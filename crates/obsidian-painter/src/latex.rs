//! The slice of Desmos LaTeX that drawings use, parsed and evaluated.
//!
//! Numbers, `+ - \cdot \times /`, powers (`x^{2}`, `x^2`), implicit
//! multiplication (`2x`, `3\sin t`), `\frac{a}{b}`, `\sqrt{a}`, `\sqrt[n]{a}`,
//! `\left| a \right|`, the trigonometric, hyperbolic, `\ln`, `\log` and `\exp`
//! functions (`\sin^{2}x` too), `\pi`, `e`, `\theta`, and named values
//! (`a`, `a_{1}`) that an assignment defined.
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Var(String),
    Neg(Box<Self>),
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),
    Pow(Box<Self>, Box<Self>),
    Call(String, Box<Self>),
    /// A tuple: a point or a parametric curve.
    Pair(Box<Self>, Box<Self>),
}

pub type Env = HashMap<String, f64>;

impl Expr {
    /// The value, with `env` for the free names. Unknown names are NaN.
    #[must_use]
    pub fn eval(&self, env: &Env) -> f64 {
        match self {
            Self::Num(n) => *n,
            Self::Var(v) => env.get(v).copied().unwrap_or(f64::NAN),
            Self::Neg(a) => -a.eval(env),
            Self::Add(a, b) => a.eval(env) + b.eval(env),
            Self::Sub(a, b) => a.eval(env) - b.eval(env),
            Self::Mul(a, b) => a.eval(env) * b.eval(env),
            Self::Div(a, b) => a.eval(env) / b.eval(env),
            Self::Pow(a, b) => a.eval(env).powf(b.eval(env)),
            Self::Call(f, a) => call(f, a.eval(env)),
            Self::Pair(..) => f64::NAN,
        }
    }

    /// Names the expression uses (other than the constants).
    pub fn names(&self, out: &mut Vec<String>) {
        match self {
            Self::Var(v) => {
                if !out.contains(v) {
                    out.push(v.clone());
                }
            }
            Self::Neg(a) | Self::Call(_, a) => a.names(out),
            Self::Add(a, b)
            | Self::Sub(a, b)
            | Self::Mul(a, b)
            | Self::Div(a, b)
            | Self::Pow(a, b)
            | Self::Pair(a, b) => {
                a.names(out);
                b.names(out);
            }
            Self::Num(_) => {}
        }
    }
}

fn call(f: &str, x: f64) -> f64 {
    match f {
        "sin" => x.sin(),
        "cos" => x.cos(),
        "tan" => x.tan(),
        "sec" => 1.0 / x.cos(),
        "csc" => 1.0 / x.sin(),
        "cot" => 1.0 / x.tan(),
        "arcsin" => x.asin(),
        "arccos" => x.acos(),
        "arctan" => x.atan(),
        "sinh" => x.sinh(),
        "cosh" => x.cosh(),
        "tanh" => x.tanh(),
        "ln" => x.ln(),
        "log" => x.log10(),
        "exp" => x.exp(),
        "sqrt" => x.sqrt(),
        "abs" => x.abs(),
        "floor" => x.floor(),
        "ceil" => x.ceil(),
        _ => f64::NAN,
    }
}

const FUNCTIONS: [&str; 17] = [
    "sin", "cos", "tan", "sec", "csc", "cot", "arcsin", "arccos", "arctan", "sinh", "cosh", "tanh",
    "ln", "log", "exp", "floor", "ceil",
];

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Name(String),
    Func(String),
    Op(char),
    Open,
    Close,
    OpenBrace,
    CloseBrace,
    Bar,
    Frac,
    Sqrt,
    Comma,
    Eq,
}

#[allow(clippy::too_many_lines)] // one flat match over the characters
fn tokenize(s: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\n' => i += 1,
            '0'..='9' | '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                out.push(Tok::Num(s.parse().map_err(|_| format!("bad number {s}"))?));
            }
            '+' | '-' | '*' | '/' | '^' => {
                out.push(Tok::Op(c));
                i += 1;
            }
            '(' | '[' => {
                out.push(Tok::Open);
                i += 1;
            }
            ')' | ']' => {
                out.push(Tok::Close);
                i += 1;
            }
            '{' => {
                out.push(Tok::OpenBrace);
                i += 1;
            }
            '}' => {
                out.push(Tok::CloseBrace);
                i += 1;
            }
            '|' => {
                out.push(Tok::Bar);
                i += 1;
            }
            ',' => {
                out.push(Tok::Comma);
                i += 1;
            }
            '=' => {
                out.push(Tok::Eq);
                i += 1;
            }
            '_' => {
                // a subscript belongs to the name before it: a_{1} → a_1
                i += 1;
                let mut sub = String::new();
                if chars.get(i) == Some(&'{') {
                    i += 1;
                    while i < chars.len() && chars[i] != '}' {
                        sub.push(chars[i]);
                        i += 1;
                    }
                    i += 1;
                } else if let Some(&ch) = chars.get(i) {
                    sub.push(ch);
                    i += 1;
                }
                match out.last_mut() {
                    Some(Tok::Name(n)) => {
                        n.push('_');
                        n.push_str(&sub);
                    }
                    _ => return Err("a subscript needs a name before it".into()),
                }
            }
            '\\' => {
                i += 1;
                let start = i;
                while i < chars.len() && chars[i].is_ascii_alphabetic() {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                match word.as_str() {
                    "left" | "right" => {
                        // \left( \left| \left[ — the bracket that follows is the token
                    }
                    "cdot" | "times" => out.push(Tok::Op('*')),
                    "div" => out.push(Tok::Op('/')),
                    "frac" => out.push(Tok::Frac),
                    "sqrt" => out.push(Tok::Sqrt),
                    "pi" => out.push(Tok::Num(std::f64::consts::PI)),
                    "theta" | "phi" | "alpha" | "beta" | "tau" => {
                        if word == "tau" {
                            out.push(Tok::Num(std::f64::consts::TAU));
                        } else {
                            out.push(Tok::Name(word));
                        }
                    }
                    "operatorname" => {
                        // \operatorname{name}
                        if chars.get(i) == Some(&'{') {
                            let end = chars[i..]
                                .iter()
                                .position(|&ch| ch == '}')
                                .ok_or("unclosed \\operatorname")?;
                            let name: String = chars[i + 1..i + end].iter().collect();
                            i += end + 1;
                            out.push(Tok::Func(name));
                        }
                    }
                    "" => {
                        // "\ " (a LaTeX space) or "\{"
                        if chars.get(i) == Some(&'{') {
                            out.push(Tok::OpenBrace);
                            i += 1;
                        } else if chars.get(i) == Some(&'}') {
                            out.push(Tok::CloseBrace);
                            i += 1;
                        } else {
                            i += 1;
                        }
                    }
                    w if FUNCTIONS.contains(&w) => out.push(Tok::Func(w.to_string())),
                    w => return Err(format!("\\{w} is not supported")),
                }
            }
            c if c.is_ascii_alphabetic() => {
                if c == 'e' {
                    out.push(Tok::Num(std::f64::consts::E));
                } else {
                    out.push(Tok::Name(c.to_string()));
                }
                i += 1;
            }
            c => return Err(format!("unexpected '{c}'")),
        }
    }
    Ok(out)
}

struct Parser {
    toks: Vec<Tok>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.at).cloned();
        self.at += 1;
        t
    }

    fn expect(&mut self, t: &Tok) -> Result<(), String> {
        match self.next() {
            Some(ref got) if got == t => Ok(()),
            got => Err(format!("expected {t:?}, found {got:?}")),
        }
    }

    /// sum := term (('+'|'-') term)*
    fn sum(&mut self) -> Result<Expr, String> {
        let mut left = self.term()?;
        while let Some(Tok::Op(op @ ('+' | '-'))) = self.peek().cloned() {
            self.at += 1;
            let right = self.term()?;
            left = if op == '+' {
                Expr::Add(left.into(), right.into())
            } else {
                Expr::Sub(left.into(), right.into())
            };
        }
        Ok(left)
    }

    /// term := unary (('*'|'/'|implicit) unary)*
    fn term(&mut self) -> Result<Expr, String> {
        let mut left = self.unary()?;
        loop {
            match self.peek() {
                Some(Tok::Op('*')) => {
                    self.at += 1;
                    left = Expr::Mul(left.into(), self.unary()?.into());
                }
                Some(Tok::Op('/')) => {
                    self.at += 1;
                    left = Expr::Div(left.into(), self.unary()?.into());
                }
                Some(
                    Tok::Num(_)
                    | Tok::Name(_)
                    | Tok::Func(_)
                    | Tok::Open
                    | Tok::OpenBrace
                    | Tok::Frac
                    | Tok::Sqrt,
                ) => left = Expr::Mul(left.into(), self.power()?.into()),
                _ => return Ok(left),
            }
        }
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if let Some(Tok::Op('-')) = self.peek() {
            self.at += 1;
            return Ok(Expr::Neg(self.unary()?.into()));
        }
        if let Some(Tok::Op('+')) = self.peek() {
            self.at += 1;
        }
        self.power()
    }

    /// power := atom ('^' exponent)?
    fn power(&mut self) -> Result<Expr, String> {
        let base = self.atom()?;
        if let Some(Tok::Op('^')) = self.peek() {
            self.at += 1;
            let exp = self.exponent()?;
            return Ok(Expr::Pow(base.into(), exp.into()));
        }
        Ok(base)
    }

    /// `^{...}`, or a single token: `x^2`, `x^-1`.
    fn exponent(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Tok::OpenBrace) => {
                self.at += 1;
                let e = self.sum()?;
                self.expect(&Tok::CloseBrace)?;
                Ok(e)
            }
            Some(Tok::Op('-')) => {
                self.at += 1;
                Ok(Expr::Neg(self.atom()?.into()))
            }
            _ => self.atom(),
        }
    }

    fn atom(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Tok::Num(n)) => Ok(Expr::Num(n)),
            Some(Tok::Name(n)) => Ok(Expr::Var(n)),
            Some(Tok::OpenBrace) => {
                let e = self.sum()?;
                self.expect(&Tok::CloseBrace)?;
                Ok(e)
            }
            Some(Tok::Open) => {
                let first = self.sum()?;
                if let Some(Tok::Comma) = self.peek() {
                    self.at += 1;
                    let second = self.sum()?;
                    self.expect(&Tok::Close)?;
                    return Ok(Expr::Pair(first.into(), second.into()));
                }
                self.expect(&Tok::Close)?;
                Ok(first)
            }
            Some(Tok::Bar) => {
                let e = self.sum()?;
                self.expect(&Tok::Bar)?;
                Ok(Expr::Call("abs".into(), e.into()))
            }
            Some(Tok::Frac) => {
                self.expect(&Tok::OpenBrace)?;
                let a = self.sum()?;
                self.expect(&Tok::CloseBrace)?;
                self.expect(&Tok::OpenBrace)?;
                let b = self.sum()?;
                self.expect(&Tok::CloseBrace)?;
                Ok(Expr::Div(a.into(), b.into()))
            }
            Some(Tok::Sqrt) => {
                // \sqrt[n]{a} or \sqrt{a}
                let root = if let Some(Tok::Open) = self.peek() {
                    self.at += 1;
                    let n = self.sum()?;
                    self.expect(&Tok::Close)?;
                    Some(n)
                } else {
                    None
                };
                self.expect(&Tok::OpenBrace)?;
                let a = self.sum()?;
                self.expect(&Tok::CloseBrace)?;
                Ok(match root {
                    Some(n) => {
                        Expr::Pow(a.into(), Expr::Div(Expr::Num(1.0).into(), n.into()).into())
                    }
                    None => Expr::Call("sqrt".into(), a.into()),
                })
            }
            Some(Tok::Func(f)) => {
                // \sin^{2} x is (\sin x)^2
                let power = if let Some(Tok::Op('^')) = self.peek() {
                    self.at += 1;
                    Some(self.exponent()?)
                } else {
                    None
                };
                let arg = match self.peek() {
                    Some(Tok::Open | Tok::OpenBrace) => self.atom()?,
                    _ => self.power()?,
                };
                let applied = Expr::Call(f, arg.into());
                Ok(match power {
                    Some(p) => Expr::Pow(applied.into(), p.into()),
                    None => applied,
                })
            }
            t => Err(format!("unexpected {t:?}")),
        }
    }
}

/// One line of a graph: an assignment or equation (`left = right`) or a bare expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Equation(Expr, Expr),
    Expr(Expr),
}

/// Parse one Desmos expression line.
pub fn parse(latex: &str) -> Result<Line, String> {
    let toks = tokenize(latex)?;
    let mut parts = toks.split(|t| *t == Tok::Eq);
    let left = parts.next().unwrap_or(&[]).to_vec();
    let right = parts.next().map(<[Tok]>::to_vec);
    if parts.next().is_some() {
        return Err("more than one '='".into());
    }
    let whole = |toks: Vec<Tok>| -> Result<Expr, String> {
        let mut p = Parser { toks, at: 0 };
        let e = p.sum()?;
        if p.at < p.toks.len() {
            return Err(format!("unexpected {:?}", p.toks[p.at]));
        }
        Ok(e)
    };
    match right {
        Some(r) => Ok(Line::Equation(whole(left)?, whole(r)?)),
        None => Ok(Line::Expr(whole(left)?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(s: &str, x: f64) -> f64 {
        let Line::Expr(e) = parse(s).unwrap() else {
            panic!("not an expression: {s}")
        };
        e.eval(&[("x".to_string(), x), ("t".to_string(), x)].into())
    }

    #[test]
    fn evaluates_the_usual_forms() {
        let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
        assert!(close(value("96+2t+8t^{2}-6t^{3}", 1.0), 100.0));
        assert!(close(value("2x", 3.0), 6.0));
        assert!(close(value("\\frac{1}{2}x^2", 2.0), 2.0));
        assert!(close(value("\\sin^{2}x+\\cos^{2}x", 0.7), 1.0));
        assert!(close(value("3\\sin(\\pi x)", 0.5), 3.0));
        assert!(close(value("\\sqrt{x}\\cdot\\sqrt[3]{8}", 4.0), 4.0));
        assert!(close(value("\\left|x-5\\right|", 2.0), 3.0));
        assert!(close(value("-x^2", 3.0), -9.0));
        assert!(close(value("2^{-1}", 0.0), 0.5));
        assert!(close(value("e^{0}", 0.0), 1.0));
    }

    #[test]
    fn pairs_equations_and_subscripts() {
        assert!(matches!(
            parse("\\left(1,2\\right)").unwrap(),
            Line::Expr(Expr::Pair(..))
        ));
        let Line::Equation(Expr::Var(name), _) = parse("a_{1}=3").unwrap() else {
            panic!()
        };
        assert_eq!(name, "a_1");
        assert!(parse("\\foo{x}").is_err());
        assert!(parse("x=1=2").is_err());
    }
}
