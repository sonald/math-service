use super::math::*;
use cairo::*;
use log::*;
use std::fs::File;
use std::ops::Range;

trait Contains {
    type Item;
    fn has(&self, v: Self::Item) -> bool;
}

impl<T> Contains for Range<T>
where
    T: PartialOrd,
{
    type Item = T;
    fn has(&self, v: Self::Item) -> bool {
        self.start <= v && v < self.end
    }
}

#[derive(Debug)]
pub struct Configuration {
    pub title: String,
    pub level: i32,
    pub result_range: Range<i32>,
    pub single_range: Range<i32>,
    pub addition_range: Range<i32>,
    pub multiplication_range: Range<i32>,
}

pub struct ValidatorForMySon<'a> {
    cfg: &'a Configuration,
    pub has_mul_or_div: bool,
}

impl<'a> Validator for ValidatorForMySon<'a> {
    fn on_single(&mut self, v: i32) -> bool {
        v >= self.cfg.single_range.start && v < self.cfg.single_range.end
    }

    fn on_primitive(&mut self, op: Op, v1: i32, v2: i32) -> bool {
        match op {
            Op::Div => {
                self.has_mul_or_div = true;
                (2..10).has(v2) && (v1 / v2 < 10) && (v1 % v2 == 0)
            }
            Op::Mul => {
                self.has_mul_or_div = true;
                //(5..21).has(v1) && (5..21).has(v2)
                self.cfg.multiplication_range.has(v1) && self.cfg.multiplication_range.has(v2)
            }
            Op::Minus => {
                self.cfg.addition_range.has(v1) && self.cfg.addition_range.has(v2) && v1 > v2
            }

            _ => {
                self.cfg.addition_range.has(v1) && self.cfg.addition_range.has(v2)
            }
        }
    }

    fn init(&mut self) {
        self.has_mul_or_div = false;
    }

    fn pass(&self) -> bool {
        self.has_mul_or_div
    }
}

impl Configuration {
    pub fn basic() -> Configuration {
        Configuration {
            title: "四则混合练习题".to_string(),
            level: 2,
            single_range: 0..100,
            result_range: 0..300,
            addition_range: 10..100,
            multiplication_range: 5..21,
        }
    }

    /// generate random math expression
    /// 1 2 3 + * => 1 * (2+3)
    /// level: 1 => two oprands one op
    /// level: 2 => three oprands two op
    /// level: 3 => four oprands three op
    pub fn generate_rand_math(&mut self) -> Expr {
        let level = self.level;
        let (noprand, nop) = (level + 1, level);
        let mut e: Expr;

        let mut validator = ValidatorForMySon {
            cfg: &self,
            has_mul_or_div: false,
        };
        loop {
            e = Expr::gen(noprand, nop);
            validator.init();
            if e.validate(&mut validator) && validator.pass() {
                match e.eval() {
                    ev if self.result_range.start <= ev && ev <= self.result_range.end => {
                        break;
                    }
                    _ => {}
                }
            }
        }
        //eprintln!("{:?} => {}", e, e);
        e
    }

    pub fn generate_math(&mut self, cr: &Context) {
        let msg = format!(
            "{:10}={}",
            self.generate_rand_math().to_string(),
            " ".repeat(5)
        );
        //eprintln!("{}", &msg);
        cr.show_text(&msg);
    }

    pub fn render_page(&mut self, target: &Surface) {
        let cr = Context::new(&target);
        cr.set_antialias(Antialias::Subpixel);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_font_size(14.0);

        cr.move_to(20.0, 40.0);
        cr.select_font_face("Noto Sans CJK JP", FontSlant::Normal, FontWeight::Normal);
        let title = format!("{}{}", " ".repeat(60), self.title);
        cr.show_text(title.as_str());

        let mut y = 30;

        for _ in 0..3 {
            y += 50;
            cr.move_to(20.0, y as f64);

            cr.select_font_face("Noto Sans CJK JP", FontSlant::Normal, FontWeight::Normal);
            cr.show_text("   日期:________   用时:________  错____个");

            cr.select_font_face("mono", FontSlant::Normal, FontWeight::Bold);
            for _ in 0..6 {
                y += 35;
                cr.move_to(20.0, y as f64);
                (0..4).for_each(|_| self.generate_math(&cr));
            }
        }
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
