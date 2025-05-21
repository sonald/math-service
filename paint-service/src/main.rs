use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Error, Responder};
use actix_files::NamedFile;
use std::path::PathBuf;

#[macro_use] extern crate diesel;
#[macro_use] extern crate log;

extern crate dotenv;
extern crate rand;
extern crate r2d2;
// extern crate r2d2_diesel; // Removed as using diesel's built-in r2d2 feature
extern crate paint_math;
extern crate mathgen;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager as DieselConnectionManager; // Using diesel's r2d2 ConnectionManager

use paint_math::paint::PrimitiveMathGen;
use paint_math::paint::MathPainter; // Added import for MathPainter
use serde::Deserialize;

mod schema;
mod models;

#[derive(Deserialize)]
struct GenerateParams {
    title: String,
    level: i32,
}

pub struct MathState {
    pool: r2d2::Pool<DieselConnectionManager<PgConnection>>, // Updated ConnectionManager
}

impl MathState {
    pub fn new(pool: r2d2::Pool<DieselConnectionManager<PgConnection>>) -> Self { // Updated ConnectionManager
        MathState {
            pool: pool,
        }
    }
}

async fn index(_req: HttpRequest) -> Result<NamedFile, Error> {
    let path = PathBuf::from("static/index.html");
    Ok(NamedFile::open(path)?)
}

async fn index2(req: HttpRequest) -> impl Responder { 
    info!("index2: {:?}", req);
    HttpResponse::Ok().body("hello from index2")
}

async fn handle_generate(
    data: web::Data<MathState>, 
    params: web::Query<GenerateParams>
) -> Result<HttpResponse, Error> {
    let title = params.title.clone();
    let level = params.level;
    let pool = data.pool.clone();

    let result: Result<Vec<u8>, _> = web::block(move || {
        let _db_conn = pool.get().map_err(|e| {
            error!("Failed to get DB connection from pool: {}",e);
            // () // Error type for web::block needs to be consistent
        })?; // Added ? to propagate error, ensure error type matches block's requirements

        let mut gen = PrimitiveMathGen::new();
        gen.level = level;
        let mut painter = MathPainter::new(gen); 
        painter.title = title;
        let pdf_data = painter.render_pdf_to_stream();
        Ok(pdf_data) as Result<Vec<u8>, ()> // Error type for web::block needs to be simple or map to one
    }).await.map_err(|e| {
        error!("Blocking error: {}", e);
        actix_web::error::ErrorInternalServerError("Blocking error") // Ensure this error type matches function signature
    })?;

    match result {
        Ok(pdf_data) => {
            Ok(HttpResponse::Ok()
                .content_type("application/pdf")
                .body(pdf_data))
        }
        Err(_) => {
             Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

async fn generate_math(data: web::Data<MathState>) -> impl Responder { 
    let _pool = data.pool.clone(); 
    let mut gen = PrimitiveMathGen::new();
    let mut painter = MathPainter::new(gen); 
    let pdf_data = painter.render_pdf_to_stream();
    HttpResponse::Ok()
        .content_type("application/pdf")
        .body(pdf_data)
}

async fn generate_math_png(data: web::Data<MathState>) -> impl Responder { 
    let _pool = data.pool.clone(); 
    let mut gen = PrimitiveMathGen::new();
    let mut painter = MathPainter::new(gen); 
    let png_data = painter.render_png_to_stream();
    HttpResponse::Ok()
        .content_type("image/png")
        .body(png_data)
}

#[actix_web::main] 
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info,paint_service=info");
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = DieselConnectionManager::<PgConnection>::new(database_url); // Updated ConnectionManager
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(MathState::new(pool.clone()))) 
            .wrap(actix_web::middleware::Logger::default())
            .service(web::resource("/generate_math_params").route(web::get().to(handle_generate)))
            .service(web::resource("/generate_math").route(web::get().to(generate_math)))
            .service(web::resource("/generate_math_png").route(web::get().to(generate_math_png)))
            .service(web::resource("/index2.html").route(web::get().to(index2)))
            .service(web::resource("/").route(web::get().to(index))) 
            .service(actix_files::Files::new("/", "static").show_files_listing()) 
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}
