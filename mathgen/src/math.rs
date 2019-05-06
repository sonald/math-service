use rand::prelude::*;
use std::fmt::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Add,
    Minus,
    Mul,
    Div,
}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Minus => write!(f, "-"),
            Op::Mul => write!(f, "x"),
            Op::Div => write!(f, "÷"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Single(i32),
    Primitive(Op, i32, i32),
    Compound(Op, Box<Expr>, Box<Expr>),
}

/// render Expr with least brackets required
impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Single(v) => write!(f, "{}", v),
            Primitive(op, v1, v2) => write!(f, "{}{}{}", v1, op, v2),
            Compound(op, v1, v2) => match op {
                Op::Add => write!(f, "{}{}{}", v1, op, v2),
                Op::Minus => {
                    write!(f, "{}{}", v1, op)?;

                    match v2.as_ref() {
                        Single(_) => write!(f, "{}", v2),
                        Primitive(op2, _, _) | Compound(op2, _, _) => {
                            if *op2 == Op::Add || *op2 == Op::Minus {
                                write!(f, "({})", v2)
                            } else {
                                write!(f, "{}", v2)
                            }
                        }
                    }
                }
                Op::Div | Op::Mul => {
                    match v1.as_ref() {
                        Single(_) => write!(f, "{}{}", v1, op),
                        Primitive(op2, _, _) | Compound(op2, _, _) => {
                            if *op2 == Op::Add || *op2 == Op::Minus {
                                write!(f, "({}){}", v1, op)
                            } else {
                                write!(f, "{}{}", v1, op)
                            }
                        }
                    }?;

                    match v2.as_ref() {
                        Single(_) => write!(f, "{}", v2),
                        Primitive(op2, _, _) | Compound(op2, _, _) => {
                            if *op2 == Op::Add || *op2 == Op::Minus || *op == Op::Div {
                                write!(f, "({})", v2)
                            } else {
                                write!(f, "{}", v2)
                            }
                        }
                    }
                }
            },
        }
    }
}

use self::Expr::*;

fn rand_op() -> Op {
    let mut rng = thread_rng();
    match rng.gen_range(0, 4) {
        0 => Op::Add,
        1 => Op::Minus,
        2 => Op::Mul,
        3 => Op::Div,
        _ => unreachable!(),
    }
}

impl Expr {
    pub fn gen(noprand: i32, nop: i32) -> Expr {
        let mut rng = thread_rng();

        match (noprand, nop) {
            (1, 0) => Single(rng.gen_range(1, 200)),
            (2, 1) => Primitive(rand_op(), rng.gen_range(1, 200), rng.gen_range(1, 200)),
            _ => {
                let lnoprand = rng.gen_range(1, noprand);
                let rnoprand = noprand - lnoprand;

                let lhs = Expr::gen(lnoprand, lnoprand - 1);
                let rhs = Expr::gen(rnoprand, rnoprand - 1);

                Compound(rand_op(), Box::new(lhs), Box::new(rhs))
            }
        }
    }

    pub fn eval(&self) -> i32 {
        match self {
            Single(v) => *v,
            Primitive(op, v1, v2) => match op {
                Op::Div => *v1 / *v2,
                Op::Mul => *v1 * *v2,
                Op::Minus => *v1 - *v2,
                _ => *v1 + *v2,
            },
            Compound(op, v1, v2) => match op {
                Op::Div => v1.eval() / v2.eval(),
                Op::Mul => v1.eval() * v2.eval(),
                Op::Minus => v1.eval() - v2.eval(),
                _ => v1.eval() + v2.eval(),
            },
        }
    }

    pub fn validate<V: Validator>(&self, validator: &mut V) -> bool {
        match self {
            Single(v) => validator.on_single(*v),
            Primitive(op, v1, v2) => {
                validator.on_single(*v1)
                    && validator.on_single(*v2)
                    && validator.on_primitive(*op, *v1, *v2)
            }
            Compound(op, v1, v2) => {
                v1.validate(validator)
                    && v2.validate(validator)
                    && Expr::Primitive(*op, v1.eval(), v2.eval()).validate(validator)
            }
        }
    }
}

pub trait Validator {
    fn on_single(&mut self, v: i32) -> bool;
    fn on_primitive(&mut self, op: Op, v1: i32, v2: i32) -> bool;
    fn pass(&self) -> bool;
    fn init(&mut self);
}
