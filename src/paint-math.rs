use mathgen::math::*;
use cairo::*;

struct ValidatorForMySun {
    has_mul_or_div: bool,
}

impl Validator for ValidatorForMySun {
    fn on_single(&mut self, v: i32) -> bool {
        v > 0 && v < 500
    }

    fn on_primitive(&mut self, op: Op, v1: i32, v2: i32) -> bool {
        match op {
            Op::Div => {
                self.has_mul_or_div = true;
                v1 < 100 && v2 < 10  && v2 > 0 && (v1 / v2 < 10) && (v1 % v2 == 0)
            },
            Op::Mul => {
                self.has_mul_or_div = true;
                v1 < 10 && v2 < 10
            },
            Op::Minus => v1 > v2,
            _ => true
        }
    }

    fn init(&mut self) {
        self.has_mul_or_div = false;
    }

    fn pass(&self) -> bool {
        self.has_mul_or_div
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
    //eprintln!("{:?} => {}", e, e);
    e
}

fn generate_math(cr: &Context) {
    let mut validator = ValidatorForMySun { has_mul_or_div: false };
    let msg = format!("{:10}={}", generate_rand_math(&mut validator).to_string(), " ".repeat(5));
    //eprintln!("{}", &msg);
    cr.show_text(&msg);
}

#[allow(unused)]
fn render_page<T: AsRef<str>>(title: T) {

    let target = pdf::File::new(8.3 * 72.0, 11.7 * 72.0, title.as_ref());

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
    (0..10).for_each(|i| { 
        let s = format!("math{}.pdf", i);
        render_page(s); 
    });
    //(0..50).for_each(|_| { generate_rand_math(); });
}
