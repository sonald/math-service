use mathgen::math::*;
use mathgen::math::Expr::*;

use cairo::*;
use log::*;
use std::fs::File;
use std::ops::{Range, Bound, RangeBounds};
use rand::prelude::*;
use std::fmt::Debug;

pub struct PrimitiveMathGen {
    pub level: i32,
    pub result_range: Range<i32>,
    pub single_range: Range<i32>,
    pub addition_range: Range<i32>,
    pub multiplication_range: Range<i32>,

    rng: ThreadRng,
    // has_mul: bool, // Removed
    // has_div: bool, // Removed
}

pub struct GenerativeMathGen {
    pub level: i32,
    pub result_range: Range<i32>,
    pub single_range: Range<i32>,
    pub add_range: Range<i32>,
    pub mul_range: Range<i32>,
    pub minus_range: Range<i32>,
    pub div_range: Range<i32>,

    rng: ThreadRng,
    // has_mul: bool, // Removed
    // has_div: bool, // Removed
}

pub struct MathPainter<G: MathGenerator> {
    g:  G,
    pub title: String,
}

fn range_union(r1: impl RangeBounds<i32> + Debug, r2: impl RangeBounds<i32> + Debug) -> Option<Range<i32>> {
    use Bound::*;
    let start = match (r1.start_bound(), r2.start_bound()) {
        (Included(&v1), Included(&v2)) => v1.max(v2),
        _ => unreachable!()
    };

    let end = match (r1.end_bound(), r2.end_bound()) {
        (Included(&v1), Included(&v2)) => v1.min(v2)+1,
        (Excluded(&v1), Included(&v2)) => v1.min(v2+1),
        (Included(&v1), Excluded(&v2)) => (v1+1).min(v2),
        (Excluded(&v1), Excluded(&v2)) => v1.min(v2),
        _ => unreachable!()
    };

    //eprintln!("range_union: {:?} U {:?} => {:?}", r1, r2, start..end);

    if start >= end {
        None
    } else {
        Some(start..end)
    }
}

impl MathGenerator for PrimitiveMathGen {
    /// generate random math expression
    /// 1 2 3 + * => 1 * (2+3)
    /// level: 1 => two oprands one op
    /// level: 2 => three oprands two op
    /// level: 3 => four oprands three op
    fn generate_rand_math(&mut self) -> Expr {
        let level = self.level;
        let (noprand, nop) = (level + 1, level);
        loop {
            let mut current_has_div = false;
            let mut current_has_mul = false;
            let e = self.gen_expr_with_state(noprand, nop, &mut current_has_div, &mut current_has_mul);
            //eprintln!("{:?} => {}", e, e);
            if self.result_range.contains(&e.eval()) && 
                (current_has_div || current_has_mul) { // Check the local state
                return e
            }
        }
    }

    // Renamed original gen to gen_expr_with_state to pass down div/mul state
    fn gen(&mut self, noprand: i32, nop: i32) -> Expr {
        let mut current_has_div = false; // Dummy state, not used by top-level call
        let mut current_has_mul = false; // Dummy state, not used by top-level call
        self.gen_expr_with_state(noprand, nop, &mut current_has_div, &mut current_has_mul)
    }
}

impl PrimitiveMathGen {
    // Helper function to pass down the div/mul state
    fn gen_expr_with_state(&mut self, noprand: i32, nop: i32, current_has_div: &mut bool, current_has_mul: &mut bool) -> Expr {
        match (noprand, nop) {
            (1, 0) => Single(self.rand(self.single_range.clone())),
            (2, 1) => {
                loop {
                    let op = self.rand_op();
                    let (l, r_val) = (self.rand(self.single_range.clone()),
                    self.rand(self.single_range.clone()));

                    let e = Primitive(op, l, r_val);
                    let mut op_has_div = false;
                    let mut op_has_mul = false;
                    if ! match op {
                        Op::Div => {
                            op_has_div = true;
                            (2..10).contains(&r_val) && (l / r_val < 10) && (l % r_val == 0)
                        },
                        Op::Mul => {
                            op_has_mul = true;
                            self.multiplication_range.contains(&l) && self.multiplication_range.contains(&r_val)
                        },
                        Op::Minus => {
                            self.addition_range.contains(&l) && self.addition_range.contains(&r_val) && l > r_val
                        },
                        _ =>  true
                    } {
                        continue;
                    }
                    *current_has_div |= op_has_div;
                    *current_has_mul |= op_has_mul;

                    return e
                }
            }
            _ => {
                let lnoprand = self.rand(1..noprand); // This uses Range<i32> which is fine for rand()
                let rnoprand = noprand - lnoprand;

                loop {
                    let lhs = self.gen_expr_with_state(lnoprand, lnoprand - 1, current_has_div, current_has_mul);
                    let rhs = self.gen_expr_with_state(rnoprand, rnoprand - 1, current_has_div, current_has_mul);

                    let op = self.rand_op();
                    let (l_eval, r_eval) = (lhs.eval(), rhs.eval());
                    let mut op_has_div = false;
                    let mut op_has_mul = false;
                    if ! match op {
                        Op::Div => {
                            op_has_div = true;
                            (2..10).contains(&r_eval) && (l_eval / r_eval < 10) && (l_eval % r_eval == 0)
                        },
                        Op::Mul => {
                            op_has_mul = true;
                            self.multiplication_range.contains(&l_eval) && self.multiplication_range.contains(&r_eval)
                        },
                        Op::Minus => {
                            self.addition_range.contains(&l_eval) && self.addition_range.contains(&r_eval) && l_eval > r_eval
                        },
                        _ =>  true
                    } {
                        continue;
                    }
                    *current_has_div |= op_has_div;
                    *current_has_mul |= op_has_mul;
                    return Compound(op, Box::new(lhs), Box::new(rhs));                
                }
            }
        }
    }

    pub fn new() -> Self {
        PrimitiveMathGen {
            level: 3,
            single_range: 10..150,
            result_range: 10..400,
            addition_range: 20..100,
            multiplication_range: 5..21,
            rng: thread_rng(),
            // has_mul: false, // Removed
            // has_div: false  // Removed
        }
    }

    pub fn rand(&mut self, r: Range<i32>) -> i32 {
        self.rng.gen_range(r.start, r.end) 
    }


    pub fn rand_op(&mut self) -> Op {
        match self.rng.gen_range(0, 4) { 
            3 => Op::Add,
            1 => Op::Minus,
            2 => Op::Mul,
            0 => Op::Div,
            _ => unreachable!(),
        }
    }
}


impl MathGenerator for GenerativeMathGen {
    fn generate_rand_math(&mut self) -> Expr {
        let level = self.level;
        self.gen(level+1, level)
    }

    fn gen(&mut self, noprand: i32, nop: i32) -> Expr {
        // The has_mul and has_div fields were removed, so no need to update them here.
        // The logic relies on gen_iter which doesn't use those fields.
        loop {
            if let Some(e) = self.gen_iter(noprand, nop, self.result_range.clone()) {
                return e
            }
        }
    }
}

macro_rules! try_option {
    ($e:expr) => (
        match $e {
            Some(v) => v,
            _ => return None
        }
    )
}
impl GenerativeMathGen {
    pub fn new() -> Self {
        GenerativeMathGen {
            level: 3,
            single_range: 10..150,
            result_range: 1..400,
            add_range: 20..400,
            minus_range: 20..100,
            mul_range: 11..200,
            div_range: 5..11,
            rng: thread_rng(),
            // has_mul: false, // Removed
            // has_div: false  // Removed
        }
    }

    pub fn gen_iter<T: RangeBounds<i32> + Clone + Debug>(&mut self, noprand: i32, nop: i32, bound: T) -> Option<Expr> {
        match (noprand, nop) {
            (1, 0) => {
                range_union(self.single_range.clone(), bound.clone())
                    .map(|range| Single(self.rand(range)))
            }
            (2, 1) => {
                let (mut l, mut r_val) = (0, 0); // Renamed r to r_val to avoid conflict
                let op = self.rand_op();
                match op {
                    Op::Div => {
                        let range = try_option!(range_union(bound.clone(), 2..10));
                        r_val = self.rand(range);
                        range_union(bound.clone(), self.div_range.clone())
                            .map(|range_inner| { // Renamed range to range_inner
                                let res = self.rand(range_inner);
                                assert!(res != 0);
                                l = r_val * res; 
                            });
                    },
                    Op::Mul => {
                        let range = try_option!(range_union(bound.clone(), 5..20));
                        r_val = self.rand(range);
                        range_union(bound.clone(), self.mul_range.clone())
                            .map(|range_inner| { // Renamed range to range_inner
                                let res = self.rand(range_inner);
                                if r_val != 0 { l = res / r_val; } else { l = 0; } // Avoid division by zero
                            });
                    },
                    Op::Minus => {
                        range_union(bound.clone(), self.single_range.clone())
                            .map(|range_inner| { // Renamed range to range_inner
                                r_val = self.rand(range_inner);
                                if let Some(res_range) = range_union(bound.clone(), self.minus_range.clone()) { // Renamed res to res_range
                                    l = self.rand((res_range.start+r_val)..(res_range.end+r_val)); // This uses Range<i32>
                                }
                            });
                    },
                    _ => { // Add and default case
                        range_union(bound.clone(), self.single_range.clone())
                            .and_then(|range_inner| { // Renamed range to range_inner
                                l = self.rand(range_inner);
                                range_union(bound.clone(), self.single_range.clone())
                                    .map(|range2| {
                                        r_val = self.rand(range2);
                                    })
                            });
                    }
                }
                Some(Primitive(op, l, r_val))
            }
            _ => {
                let lnoprand = self.rand(1..noprand); // This uses Range<i32>
                let rnoprand = noprand - lnoprand;

                let mut lhs: Expr; // Declare lhs as mutable
                let mut rhs: Expr; // Declare rhs as mutable
                let (mut l_eval, mut r_eval) = (0, 0); // Renamed l,r to l_eval, r_eval
                let op = self.rand_op();
                match op {
                    Op::Div => {
                        let mut retries = 10;
                        loop {
                            let range_rhs = try_option!(range_union(bound.clone(), 2..10)); // Renamed range to range_rhs
                            rhs = match self.gen_iter(rnoprand, rnoprand-1, range_rhs) {
                                Some(v) => v,
                                None => return None,
                            };

                            r_eval = rhs.eval();
                            if r_eval > 0 {
                                break
                            }

                            retries -= 1;
                            if retries <= 0 {
                                return None
                            }
                        }
                        let range_lhs = (self.div_range.start*r_eval)..(self.div_range.end*r_eval); // Renamed range to range_lhs
                        let range_lhs_final = try_option!(range_union(bound.clone(), range_lhs)); // Renamed range to range_lhs_final
                        lhs = match self.gen_iter(lnoprand, lnoprand-1, range_lhs_final) {
                            Some(v) => v,
                            None => return None,
                        };
                    },
                    Op::Mul => {
                        let mut retries = 10;
                        loop {
                            let range_lhs = try_option!(range_union(bound.clone(), 5..20)); // Renamed range to range_lhs
                            lhs = match self.gen_iter(lnoprand, lnoprand-1, range_lhs) {
                                Some(v) => v,
                                None => return None,
                            };

                            l_eval = lhs.eval();
                            if l_eval == 0 { // check if l_eval is zero before division
                                retries -=1;
                                if retries <=0 { return None;}
                                continue;
                            }
                            let range_rhs = (self.mul_range.start/l_eval)..(self.mul_range.end/l_eval); // Renamed range to range_rhs
                            rhs = match self.gen_iter(rnoprand, rnoprand-1, range_rhs) {
                                Some(v) => v,
                                None => return None,
                            };
                            r_eval = rhs.eval();

                            let range_check = try_option!(range_union(bound.clone(), self.mul_range.clone())); // Renamed range to range_check
                            if range_check.contains(&(l_eval * r_eval)) {
                                break
                            }

                            retries -= 1;
                            if retries <= 0 {
                                return None
                            }
                        }
                    },
                    Op::Minus => {
                        let range_rhs = try_option!(range_union(bound.clone(), self.single_range.clone())); // Renamed range to range_rhs
                        rhs = match self.gen_iter(rnoprand, rnoprand-1, range_rhs) {
                            Some(v) => v,
                            None => return None,
                        };

                        r_eval = rhs.eval();
                        let range_lhs = (self.minus_range.start+r_eval)..(self.minus_range.end+r_eval); // Renamed range to range_lhs
                        lhs = match self.gen_iter(lnoprand, lnoprand-1, range_lhs) {
                            Some(v) => v,
                            None => return None,
                        };
                    },
                    _ => { // Add and default case
                        let range_lhs = try_option!(range_union(bound.clone(), self.single_range.clone())); // Renamed range to range_lhs
                        lhs = match self.gen_iter(lnoprand, lnoprand-1, range_lhs) {
                            Some(v) => v,
                            None => return None,
                        };

                        l_eval = lhs.eval();
                        let range_rhs = (self.add_range.start-l_eval)..(self.add_range.end-l_eval); // Renamed range to range_rhs
                        rhs = match self.gen_iter(rnoprand, rnoprand-1, range_rhs) {
                            Some(v) => v,
                            None => return None,
                        };
                    }
                }

                Some(Compound(op, Box::new(lhs), Box::new(rhs)))

            }
        }
    }

    pub fn rand(&mut self, r: Range<i32>) -> i32 {
        self.rng.gen_range(r.start, r.end) 
    }


    pub fn rand_op(&mut self) -> Op {
        match self.rng.gen_range(0, 4) { 
            3 => Op::Add,
            1 => Op::Minus,
            2 => Op::Mul,
            0 => Op::Div,
            _ => unreachable!(),
        }
    }
}

impl<G> MathPainter<G> where G: MathGenerator {
    pub fn new(g: G) -> MathPainter<G> {
        MathPainter {
            g: g,
            title: "XXX".to_string()
        }
    }

    pub fn generate_math(&mut self, cr: &Context) {
        let msg = format!(
            "{:10}={}",
            self.g.generate_rand_math().to_string(),
            " ".repeat(5)
        );
        //eprintln!("{}", &msg);
        cr.show_text(&msg);
    }


    //render vertical form calculation
    pub fn render_vertical_form(&mut self, target: &Surface) {
        let cr = Context::new(&target);
        cr.set_antialias(Antialias::Subpixel);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_font_size(14.0);

        cr.move_to(20.0, 40.0);
        cr.select_font_face("Noto Sans CJK JP", FontSlant::Normal, FontWeight::Normal);
        let title = format!("{}{}", " ".repeat(60), self.title);
        cr.show_text(title.as_str());

        let mut y = 70;

        for _ in 0..3 {
            cr.move_to(20.0, y as f64);

            cr.select_font_face("Noto Sans CJK JP", FontSlant::Normal, FontWeight::Normal);
            cr.show_text("   日期:________   用时:________  错____个");

            cr.select_font_face("mono", FontSlant::Normal, FontWeight::Bold);
            for _ in 0..=1 {
                y += 30;
                cr.move_to(20.0, y as f64);
                (0..3).for_each(|_| self.generate_math(&cr));
                y += 90;
            }

            y += 10;
        }
    }

    pub fn render_mental_form(&mut self, target: &Surface) {
        let cr = Context::new(&target);
        cr.set_antialias(Antialias::Subpixel);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_font_size(14.0);

        cr.move_to(20.0, 40.0);
        cr.select_font_face("Noto Sans CJK JP", FontSlant::Normal, FontWeight::Normal);
        let title = format!("{}{}", " ".repeat(60), self.title);
        cr.show_text(title.as_str());

        let mut y = 30;

        for _ in 0..4 {
            y += 50;
            cr.move_to(20.0, y as f64);

            cr.select_font_face("Noto Sans CJK JP", FontSlant::Normal, FontWeight::Normal);
            cr.show_text("   日期:________   用时:________  错____个");

            cr.select_font_face("mono", FontSlant::Normal, FontWeight::Bold);
            for _ in 0..4 {
                y += 35;
                cr.move_to(20.0, y as f64);
                (0..4).for_each(|_| self.generate_math(&cr));
            }
        }
    }

    pub fn render_page(&mut self, target: &Surface) {
        //self.render_vertical_form(target)
        self.render_mental_form(target)
    }

    pub fn render_pdf<T: AsRef<str>>(&mut self, name: T) {
        let target = pdf::File::new(8.3 * 72.0, 11.7 * 72.0, name.as_ref());
        self.render_page(&target);
    }

    pub fn render_pdf_to_stream(&mut self) -> Vec<u8> {
        let mut buf = Vec::new();

        let target = pdf::Writer::new(8.3 * 72.0, 11.7 * 72.0, &mut buf);
        self.render_page(&target);
        target.finish();

        debug!("render_pdf_to_stream: size = {}", buf.len());

        buf
    }

    pub fn render_png<T: AsRef<str>>(&mut self, name: T) {
        // A4 @72dpi = 595x842
        let target = ImageSurface::create(Format::ARgb32, 595, 842).unwrap();
        self.render_page(&target);

        if let Ok(mut f) = File::create(String::from(name.as_ref())) {
            target.write_to_png(&mut f).ok();
        }
    }

    pub fn render_png_to_stream(&mut self) -> Vec<u8> {
        // A4 @72dpi = 595x842
        let target = ImageSurface::create(Format::ARgb32, 595, 842).unwrap();
        self.render_page(&target);

        let mut buf = Vec::new();
        target.write_to_png(&mut buf).ok();
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_test() {
        let r = 4..15;
        let r2 = 6..23;
        let res = range_union(r, r2).unwrap();
        assert_eq!(res.start_bound(), Bound::Included(&6));
        assert_eq!(res.end_bound(), Bound::Excluded(&15));

        let r = 4..=15;
        let r2 = 6..15;
        let res = range_union(r, r2).unwrap();
        assert_eq!(res.start_bound(), Bound::Included(&6));
        assert_eq!(res.end_bound(), Bound::Excluded(&15));

        let r = 4..15;
        let r2 = 2..9;
        let res = range_union(r, r2).unwrap();
        assert_eq!(res.start_bound(), Bound::Included(&4));
        assert_eq!(res.end_bound(), Bound::Excluded(&9));

        let r = 4..5;
        let r2 = 2..9;
        let res = range_union(r, r2).unwrap();
        assert_eq!(res.start_bound(), Bound::Included(&4));
        assert_eq!(res.end_bound(), Bound::Excluded(&5));

        let mut g = GenerativeMathGen::new();
        eprintln!("{}", g.gen(3, 2));
        eprintln!("{}", g.gen(3, 2));
        eprintln!("{}", g.gen(3, 2));

        let now = std::time::Instant::now();
        let mut g = GenerativeMathGen::new();
        (0..1000).for_each(|_| {
            eprintln!("{}", g.gen(3, 2)); // This will now print to stderr
            g.gen(3, 2); 
        });
        eprintln!("duration: {}", now.elapsed().as_millis());

        let now = std::time::Instant::now();
        let mut g = GenerativeMathGen::new();
        (0..1000).for_each(|_| {
            //eprintln!("{}", g.gen(4, 3));
            g.gen(4, 3); 
        });
        eprintln!("duration: {}", now.elapsed().as_millis());
    }

}
