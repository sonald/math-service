use cairo::*;
use rand::prelude::*;
use std::fmt::*;

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Add,
    Minus,
    Mul,
    Div
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

#[allow(unused)]
#[derive(Debug, Clone)]
enum Expr {
    Single(i32),
    Primitive(Op, i32, i32),
    Compound(Op, Box<Expr>, Box<Expr>)
}

//(axb-3)x(cxd)
impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Single(v) => write!(f, "{}", v),
            Primitive(op, v1, v2) => write!(f, "{}{}{}", v1, op, v2),
            Compound(op, v1, v2) => {
                match op {
                    Op::Add => write!(f, "{}{}{}", v1, op, v2),
                    Op::Minus => {
                        write!(f, "{}{}", v1, op)?;
                        match v2.as_ref() {
                            Single(_) => write!(f, "{}", v2),
                            _ => write!(f, "({})", v2),
                        }
                    },
                    Op::Div | Op::Mul => {
                        match v1.as_ref() {
                            Single(_) => write!(f, "{}{}", v1, op),
                            Primitive(op2, _, _) | Compound(op2, _, _) => {
                                if *op2 == Op::Add || *op2 == Op::Minus {
                                    write!(f, "({}){}", v1, op)
                                } else {
                                    write!(f, "{}{}", v1, op)
                                }
                            },
                        }?;

                        match v2.as_ref() {
                            Single(_) => write!(f, "{}", v2),
                            Primitive(op2, _, _) | Compound(op2, _, _) => {
                                if *op2 == Op::Add || *op2 == Op::Minus {
                                    write!(f, "{}", v2)
                                } else {
                                    write!(f, "{}", v2)
                                }
                            },
                        }
                    },
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
        _ => unreachable!()
    }
}

fn gen_expr(mut noprand: i32, mut nop: i32) -> Expr {
    let mut rng = thread_rng();

    match (noprand, nop) {
        (1, 0) => Single(rng.gen_range(1, 100)),
        (2, 1) => Primitive(rand_op(), rng.gen_range(1, 100), rng.gen_range(1, 100)),
        _ => {

            let lnoprand = rng.gen_range(1, noprand);
            let rnoprand = noprand - lnoprand;

            let lhs = gen_expr(lnoprand, lnoprand-1);
            let rhs = gen_expr(rnoprand, rnoprand-1);

            //assert!()

            Compound(rand_op(), Box::new(lhs), Box::new(rhs))
        },
    }
}

fn eval_expr(e: &Expr) -> i32 {
    match e {
        Single(v) => *v,
        Primitive(op, v1, v2) => match op {
            Op::Div => *v1 / *v2,
            Op::Mul => *v1 * *v2,
            Op::Minus => *v1 - *v2,
            _ => *v1 + *v2,
        },
        Compound(op, v1, v2) => match op {
            Op::Div => eval_expr(v1) / eval_expr(v2),
            Op::Mul => eval_expr(v1) * eval_expr(v2),
            Op::Minus => eval_expr(v1) - eval_expr(v2),
            _ => eval_expr(v1) + eval_expr(v2),
        }
    }
}

trait Validator {
    fn on_single(&mut self, v: i32) -> bool;
    fn on_primitive(&mut self, op: Op, v1: i32, v2: i32) -> bool;
    fn pass(&self) -> bool;
    fn init(&mut self);
}

fn validate_expr<V: Validator>(e: &Expr, validator: &mut V) -> bool {
    match e {
        Single(v) => validator.on_single(*v),
        Primitive(op, v1, v2) => validator.on_primitive(*op, *v1, *v2),
        Compound(op, v1, v2) => {
            if validate_expr(v1, validator) && validate_expr(v2, validator) {
                validate_expr(&Expr::Primitive(*op, eval_expr(v1), eval_expr(v2)), validator)
            } else {
                false
            }
        }
    }
}

struct ValidatorForMySun {
    has_mul: bool,
}

impl Validator for ValidatorForMySun {
    fn on_single(&mut self, v: i32) -> bool {
        v > 0 && v < 500
    }

    fn on_primitive(&mut self, op: Op, v1: i32, v2: i32) -> bool {
        match op {
            Op::Div => v1 < 100 && v2 < 10  && v2 > 0 && (v1 / v2 < 10) && (v1 % v2 == 0),
            Op::Mul => {self.has_mul = true; v1 < 10 && v2 < 10},
            Op::Minus => v1 > v2,
            _ => true
        }
    }

    fn init(&mut self) {
        self.has_mul = false;
    }

    fn pass(&self) -> bool {
        self.has_mul
    }
}

//TODO: move policy out
#[allow(unused)]
fn validate_expr2(e: &Expr) -> bool {
    match e {
        Single(v) => *v > 0 && *v < 500,
        Primitive(op, v1, v2) => {
            match op {
                Op::Div => *v1 < 100 && *v2 < 10  && *v2 > 0 && (*v1 / *v2 < 10) && (*v1 % *v2 == 0),
                Op::Mul => *v1 < 10 && *v2 < 10,
                Op::Minus => *v1 > *v2,
                _ => true
            }
        },
        Compound(op, v1, v2) => {
            if validate_expr2(v1) && validate_expr2(v2) {
                validate_expr2(&Expr::Primitive(*op, eval_expr(v1), eval_expr(v2)))
            } else {
                false
            }
        }
    }
}

/// generate random math expression
/// 1 2 3 + * => 1 * (2+3)
/// level: 1 => two oprands one op
/// level: 2 => three oprands two op
/// level: 3 => four oprands three op
fn generate_rand_math<V: Validator>(validator: &mut V) -> Expr {
    let level = 2;
    let (noprand, nop) = (level+1, level);
    let mut e: Expr;

    loop {
        e = gen_expr(noprand, nop);
        validator.init();
        if validate_expr(&e, validator) && validator.pass() {
            break;
        }
    }
    eprintln!("{:?} => {}", e, e);
    e
}

#[allow(unused)]
fn generate_math_old(cr: &Context) {
    let mut rng = thread_rng();

    'out: loop {
        let (mut a, b, c, d) = (
            rng.gen_range(2, 10), rng.gen_range(2, 10), rng.gen_range(10, 50), rng.gen_range(10 ,50));

        let mut msg;
        let r = match rng.gen_range(1, 3) {
            1 => {
                msg = format!("{}+{}x{}-{}", c, a, b, d);
                c + a * b - d
            },
            _ => {
                a = a * b;
                msg = format!("{}+{}÷{}-{}", c, a, b, d);
                c + ( a / b) - d
            }
        };

        if r < 0 || r > 200 || a > 100 {
            continue 'out;
        }

        cr.show_text(&msg);

        msg = if msg.len() < 10 {
            format!("{}={}", " ".repeat(10-msg.len()), " ".repeat(5))
        } else {
            format!("={}", " ".repeat(5))
        };

        cr.show_text(&msg);
        break 'out;
    }
}

fn generate_math(cr: &Context) {
    let mut validator = ValidatorForMySun { has_mul: false };
    let msg = format!("{:10}={}", generate_rand_math(&mut validator).to_string(), " ".repeat(5));
    eprintln!("{}", &msg);
    cr.show_text(&msg);
}

#[allow(unused)]
fn render_page() {

    let target = pdf::File::new(8.3 * 72.0, 11.7 * 72.0, "math.pdf");

    let cr = Context::new(&target);
    cr.set_antialias(Antialias::Subpixel);
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_font_size(14.0);

    cr.move_to(20.0, 40.0);
    cr.select_font_face("Noto Sans CJK JP", FontSlant::Normal, FontWeight::Normal);
    let title = format!("{}{}", " ".repeat(60), "四则混合练习题（曹宇轩）");
    cr.show_text(title.as_str());

    let mut y = 30;

    for grp in 0..3 {
        y += 50;
        cr.move_to(20.0, y as f64);

        cr.select_font_face("Noto Sans CJK JP", FontSlant::Normal, FontWeight::Normal);
        cr.show_text("   日期:________   用时:________  错____个");

        cr.select_font_face("mono", FontSlant::Normal, FontWeight::Bold);
        for r in 0..6 {
            y += 35;
            cr.move_to(20.0, y as f64);
            for i in 0..4 {
                generate_math(&cr);
            }
        }
    }
}

fn main() {
    render_page();
    //(0..50).for_each(|_| { generate_rand_math(); });
}
