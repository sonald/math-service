use actix_web::{server, Result, Error, Responder, http, HttpRequest, HttpResponse, App, fs::NamedFile};
use actix_web::{HttpMessage, AsyncResponder};
use actix_web::error::ErrorNotFound;

use log::*;
use std::cell::Cell;
use dotenv::dotenv;

//use serde::Deserialize;
#[macro_use] extern crate serde_derive;

use futures::future::{Future, ok};

use mathgen::paint::*;

struct MathState {
    requests: Cell<i32>,
}

#[derive(Deserialize)]
struct GenerateFormData {
    title: String,
    level: i32,
    range: i32,
    kind: String
}

impl MathState {
    fn new() -> MathState {
        info!("create new state");

        MathState {
            requests: Cell::new(0)
        }
    }

    fn incr(&self) {
        debug!("incr: {}", self.requests.get());
        self.requests.set(self.requests.get() + 1);
    }
}

fn index(_: &HttpRequest<MathState>) -> Result<NamedFile> {
    info!("get index");

    if cfg!(feature = "service") {
        Ok(NamedFile::open("/web/api.sonald.me/index.html")?)
    } else if cfg!(feature = "local") {
        Ok(NamedFile::open("./index.html")?)
    } else {
        Err(ErrorNotFound("format is not supported"))
    }
}

fn index2(req: &HttpRequest<MathState>) -> impl Responder {
    info!("get index");

    req.state().incr();

    let body = format!("<h1> Math Garden </h1>
    <div>
        requests : {}
    </div> ", req.state().requests.get());

    let mut resp = HttpResponse::with_body(http::StatusCode::OK, body);
    resp.headers_mut().insert(http::header::CONTENT_TYPE, "text/html".parse().unwrap());

    resp
}

fn handle_generate(req: &HttpRequest<MathState>) -> Box<Future<Item=HttpResponse, Error=Error>> {
    info!("handle_generate");

    req.urlencoded::<GenerateFormData>()
        .from_err()
        .and_then(|fd: GenerateFormData| {
            info!("form: (title = {}, level = {}, range = {}, kind = {})",
            fd.title, fd.level, fd.range, fd.kind);
            let mut cfg = Configuration::basic();
            cfg.result_range = 0..fd.range;
            cfg.title = fd.title;
            cfg.level = fd.level;

            let (body, ct) = match fd.kind.as_ref() {
                "pdf" => (cfg.render_pdf_to_stream(), "application/pdf"),
                _ => (cfg.render_png_to_stream(), "image/png")
            };

            ok(HttpResponse::Ok().content_type(ct).body(body))


        })
        .responder()
}

fn generate_math(_: &HttpRequest<MathState>) -> impl Responder {
    let mut cfg = Configuration::basic();

    let body = cfg.render_pdf_to_stream(); 

    HttpResponse::Ok()
        .content_type("application/pdf")
        .body(body)
}

fn generate_math_png(_: &HttpRequest<MathState>) -> impl Responder {
    let mut cfg = Configuration::basic();

    let body = cfg.render_png_to_stream(); 

    info!("generate_math_png, read {}", body.len());

    HttpResponse::Ok()
        .content_type("image/png")
        .body(body)
}

fn main() {
    dotenv().ok();

    env_logger::init();

    let serv = server::new(|| App::with_state(MathState::new())
                .prefix("/apps/math")
                .resource("/", |r| {
                    r.method(http::Method::POST).f(handle_generate);
                    r.method(http::Method::GET).f(index)
                })
                .resource("/pdf", |r| r.method(http::Method::GET).f(generate_math))
                .resource("/png", |r| r.method(http::Method::GET).f(generate_math_png)));

    if cfg!(feature = "service") {
        serv.bind("127.0.0.1:8080").unwrap().run();
    } else if cfg!(feature = "local") {
        serv.bind("0.0.0.0:8080").unwrap().run();
    }

}
