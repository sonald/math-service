use paint_math::paint::*;
use std::process::Command;

//TODO: use serde-yaml to load configurations
fn main() {

    dotenv::dotenv().ok();

    {
        let mut gen = PrimitiveMathGen::new();
        gen.single_range = 2..100;
        gen.result_range = 10..999;
        gen.addition_range = 10..100;
        gen.multiplication_range = 2..20;
        gen.level = 2;

        let mut painter = MathPainter::new(gen);
        painter.title = format!("1000以内4则混合练习题（{}）", std::env::var("PM_NAME").unwrap());

        (0..10).for_each(|i| { 
            let s = format!("math{}.pdf", i);
            painter.render_pdf(&s); 
        });

        Command::new("pdfunite")
            .args((0..10).map(|v| format!("math{}.pdf", v)).collect::<Vec<_>>())
            .arg("math_hybrid.pdf")
            .spawn()
            .expect("generate math.pdf");
    }
}
