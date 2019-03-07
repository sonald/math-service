use actix_web::{server, HttpRequest, HttpResponse, App};

fn index(_: &HttpRequest) -> &'static str {
    "<h1> Math Garden </h1>"
}

fn main() {
    env_logger::init();

    server::new(|| App::new().resource("r", |r| r.f(index)))
        .bind("127.0.0.1:8080")
        .unwrap()
        .run();

}
