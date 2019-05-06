use mathgen::paint::*;
use std::process::Command;

fn main() {
    let mut cfg = Configuration::basic();
    cfg.title = "四则混合练习题（曹宇轩）".to_string();

    (0..10).for_each(|i| { 
        let s = format!("math{}.pdf", i);
        cfg.render_pdf(&s); 
    });


    Command::new("pdfunite")
        .args((0..10).map(|v| format!("math{}.pdf", v)).collect::<Vec<_>>())
        .arg("math.pdf")
        .spawn()
        .expect("generate math.pdf");
    //let s = format!("math.png");
    //cfg.render_png(s); 
}
