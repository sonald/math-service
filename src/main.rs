use actix_web::{server, Responder, http, HttpRequest, HttpResponse, App};
use std::fs::File;
use std::io::prelude::*;
use std::vec::Vec;

use log::*;
use std::cell::Cell;
use dotenv::dotenv;

use mathgen::paint::*;

struct MathState {
    requests: Cell<i32>,
}

impl MathState {
    fn new() -> MathState {
        debug!("create new state");

        MathState {
            requests: Cell::new(0)
        }
    }

    fn incr(&self) {
        debug!("incr: {}", self.requests.get());
        self.requests.set(self.requests.get() + 1);
    }
}

fn index(req: &HttpRequest<MathState>) -> impl Responder {
    debug!("get index");

    req.state().incr();

    let body = format!("<h1> Math Garden </h1>
    <div>
        requests : {}
    </div> ", req.state().requests.get());

    let mut resp = HttpResponse::with_body(http::StatusCode::OK, body);
    resp.headers_mut().insert(http::header::CONTENT_TYPE, "text/html".parse().unwrap());

    resp
}

fn generate_math(_: &HttpRequest<MathState>) -> impl Responder {
    let mut body = Vec::new();

    let mut f = File::open("math.pdf").expect("open failed");
    f.read_to_end(&mut body).ok();
    debug!("generate_math_pdf, read {}", body.len());

    HttpResponse::Ok()
        .content_type("application/pdf")
        .body(body)
}

fn generate_math_png(_: &HttpRequest<MathState>) -> impl Responder {
    let mut cfg = Configuration {
        validator: ValidatorForMySon {has_mul_or_div: false},
        title: "四则混合练习题".to_string(),
        level: 2
    };

    let body = cfg.render_png_to_stream(); 

    debug!("generate_math_png, read {}", body.len());

    HttpResponse::Ok()
        .content_type("image/png")
        .body(body)
}

fn main() {
    dotenv().ok();

    env_logger::init();

    server::new(|| App::with_state(MathState::new())
                .prefix("/apps/math")
                .resource("/", |r| r.method(http::Method::GET).f(index))
                .resource("/pdf", |r| r.method(http::Method::GET).f(generate_math))
                .resource("/png", |r| r.method(http::Method::GET).f(generate_math_png)))
        //.bind("127.0.0.1:8080")
        .bind("0.0.0.0:8080")
        .unwrap()
        .run();

}
