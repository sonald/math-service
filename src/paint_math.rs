use mathgen::paint::*;

fn main() {
    let mut cfg = Configuration {
        validator: ValidatorForMySon {has_mul_or_div: false},
        title: "四则混合练习题（曹宇轩）".to_string(),
        level: 2
    };

    (0..10).for_each(|i| { 
        let s = format!("math{}.pdf", i);
        cfg.render_pdf(&s); 
    });
    let s = format!("math.png");
    cfg.render_png(s); 
}
