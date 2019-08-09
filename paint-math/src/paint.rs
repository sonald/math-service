use mathgen::math::*;
use mathgen::math::Expr::*;

use cairo::*;
use log::*;
use std::fs::File;
use std::ops::Range;
use rand::prelude::*;

pub struct PrimitiveMathGen {
    pub level: i32,
    pub result_range: Range<i32>,
    pub single_range: Range<i32>,
    pub addition_range: Range<i32>,
    pub multiplication_range: Range<i32>,

    rng: ThreadRng,
    has_mul: bool,
    has_div: bool,

}

pub struct MathPainter<G: MathGenerator> {
    g:  G,
    pub title: String,
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
            let e = self.gen(noprand, nop);
            //eprintln!("{:?} => {}", e, e);
            if self.result_range.contains(&e.eval()) && 
                (self.has_div || self.has_mul) {
                return e
            }
        }
    }

    fn gen(&mut self, noprand: i32, nop: i32) -> Expr {
        match (noprand, nop) {
            (1, 0) => Single(self.rand(self.single_range.clone())),
            (2, 1) => {
                loop {
                    let op = self.rand_op();
                    let (l, r) = (self.rand(self.single_range.clone()),
                    self.rand(self.single_range.clone()));

                    let e = Primitive(op, l, r);
                    if ! match op {
                        Op::Div => {
                            self.has_div = true;
                            (2..10).contains(&r) && (l / r < 10) && (l % r == 0)
                        },
                        Op::Mul => {
                            self.has_mul = true;
                            self.multiplication_range.contains(&l) && self.multiplication_range.contains(&r)
                        },
                        Op::Minus => {
                            self.addition_range.contains(&l) && self.addition_range.contains(&r) && l > r
                        },
                        _ =>  true
                    } {
                        self.has_div = false;
                        self.has_mul = false;
                        continue;
                    }

                    return e
                }
            }
            _ => {
                let lnoprand = self.rand(1..noprand);
                let rnoprand = noprand - lnoprand;

                loop {
                    let lhs = self.gen(lnoprand, lnoprand - 1);
                    let rhs = self.gen(rnoprand, rnoprand - 1);

                    let op = self.rand_op();
                    let (l, r) = (lhs.eval(), rhs.eval());
                    if ! match op {
                        Op::Div => {
                            (2..10).contains(&r) && (l / r < 10) && (l % r == 0)
                        },
                        Op::Mul => {
                            self.multiplication_range.contains(&l) && self.multiplication_range.contains(&r)
                        },
                        Op::Minus => {
                            self.addition_range.contains(&l) && self.addition_range.contains(&r) && l > r
                        },
                        _ =>  true
                    } {
                        continue;
                    }

                    return Compound(op, Box::new(lhs), Box::new(rhs));                
                }
            }
        }
    }
}

impl PrimitiveMathGen {
    pub fn new() -> Self {
        PrimitiveMathGen {
            level: 3,
            single_range: 10..150,
            result_range: 10..400,
            addition_range: 20..100,
            multiplication_range: 5..21,
            rng: thread_rng(),
            has_mul: false,
            has_div: false
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
            "{:15}={}",
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
                (0..3).for_each(|_| self.generate_math(&cr));
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
