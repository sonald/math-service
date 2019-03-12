use super::math::*;
use cairo::*;
use std::fs::File;
use log::*;

pub struct ValidatorForMySon {
    pub has_mul_or_div: bool,
}

impl Validator for ValidatorForMySon {
    fn on_single(&mut self, v: i32) -> bool {
        v > 0 && v < 500
    }

    fn on_primitive(&mut self, op: Op, v1: i32, v2: i32) -> bool {
        match op {
            Op::Div => {
                self.has_mul_or_div = true;
                v1 < 100 && v2 < 10  && v2 > 1 && (v1 / v2 < 10) && (v1 % v2 == 0)
            },
            Op::Mul => {
                self.has_mul_or_div = true;
                v1 < 10 && 1 < v1 && 1 < v2 && v2 < 10
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

#[derive(Debug)]
pub struct Configuration<V> {
    pub validator: V,
    pub title: String,
    pub level: i32
}

impl<V> Configuration<V> where V: Validator {
    pub fn basic(v: V) -> Self {
        Configuration {
            validator: v,
            title: "".to_string(),
            level: 2
        }
    }
    /// generate random math expression
    /// 1 2 3 + * => 1 * (2+3)
    /// level: 1 => two oprands one op
    /// level: 2 => three oprands two op
    /// level: 3 => four oprands three op
    pub fn generate_rand_math(&mut self) -> Expr {
        let level = self.level;
        let (noprand, nop) = (level+1, level);
        let mut e: Expr;

        loop {
            e = gen_expr(noprand, nop);
            self.validator.init();
            if validate_expr(&e, &mut self.validator) && self.validator.pass() {
                break;
            }
        }
        //eprintln!("{:?} => {}", e, e);
        e
    }

    pub fn generate_math(&mut self, cr: &Context) {
        let msg = format!("{:10}={}", self.generate_rand_math().to_string(), " ".repeat(5));
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
                (0..4).for_each(|_| self.generate_math(&cr) );
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
